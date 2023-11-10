#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use data_channel::{BoxPacket, L2capError};
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{AnyPin, Level, Output, OutputDrive},
    interrupt, peripherals,
    uarte::{self, UarteTx},
};
use embassy_time::{Duration, Timer};
use embedded_alloc::Heap;
use nrf_softdevice::{
    ble::{
        l2cap::{L2cap, Packet},
        peripheral::{self, ConnectableAdvertisement},
        Connection, Phy, TxPower,
    },
    raw, Softdevice,
};

// global logger
use defmt_rtt as _;
// time driver
use embassy_nrf as _;
use panic_probe as _;

mod rhd2216;
use rhd2216::RHD2216;

#[global_allocator]
static HEAP: Heap = Heap::empty();

bind_interrupts!(struct Irqs {
    TIMER2 => rhd2216::InterruptHandler;
    UARTE1 => uarte::InterruptHandler<peripherals::UARTE1>;
});

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[embassy_executor::task]
async fn blink_task(pin1: AnyPin, pin2: AnyPin, pin3: AnyPin, off: bool) -> ! {
    let mut led_1 = Output::new(pin1, Level::from(off), OutputDrive::Standard);
    let mut led_2 = Output::new(pin2, Level::from(off), OutputDrive::Standard);
    let mut led_3 = Output::new(pin3, Level::from(off), OutputDrive::Standard);
    loop {
        for i in 0..4 {
            led_1.set_level(Level::from((i == 4) ^ off));
            led_2.set_level(Level::from((i == 4) ^ off));
            led_3.set_level(Level::from((i == 4) ^ off));
            Timer::after(Duration::from_millis(500)).await;
        }
    }
}

#[rustfmt::skip]
const ADVERTISEMENT_DATA: &'static [u8] = &[
    // Flags
    2, 1, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
    // Complete List of 128-bit Service Class UUIDs
    // edb74b42-8347-4285-a102-86f0b64c533c
    17, 7, 0x3c, 0x53, 0x4c, 0xb6, 0xf0, 0x86, 0x02, 0xa1, 0x85, 0x42, 0x47, 0x83, 0x42, 0x4b, 0xb7, 0xed,
];

pub fn advertisement() -> ConnectableAdvertisement<'static> {
    ConnectableAdvertisement::ScannableUndirected {
        adv_data: ADVERTISEMENT_DATA,
        scan_data: &[],
    }
}

type MyPacket = BoxPacket<2048>;

async fn send_rhd_data(
    rhd: &mut RHD2216<'_>,
    l2cap: &L2cap<MyPacket>,
    connection: &Connection,
) -> Result<(), L2capError<MyPacket>> {
    let config = nrf_softdevice::ble::l2cap::Config { credits: 3 };
    let channel = l2cap.listen(connection, &config, data_channel::PSM).await?;
    info!("Starting");
    rhd.start();
    let mut counter = 0u8;
    loop {
        let d = rhd.read().await;
        let mut packet = MyPacket::new();
        packet.append(&[counter, d.channels as u8]);
        counter = counter.wrapping_add(1);
        for v in &d.frames {
            if packet.len() > MyPacket::MTU - 2 {
                if let Err(e) = channel.tx(packet).await {
                    rhd.stop();
                    return Err(e.into());
                }
                packet = MyPacket::new();
                packet.append(&[counter]);
                counter = counter.wrapping_add(1);
            }
            packet.append(&v.to_le_bytes());
        }
        if let Err(e) = channel.tx(packet).await {
            rhd.stop();
            return Err(e.into());
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");

    // Initialise allocator
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024 * 128;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    // First we get the peripherals access crate.
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = interrupt::Priority::P3;
    config.time_interrupt_priority = interrupt::Priority::P3;
    let p = embassy_nrf::init(config);

    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_SYNTH as u8,
            rc_ctiv: 0,
            rc_temp_ctiv: 0,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 1,
            event_length: 40,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }),
        conn_gattc: Some(raw::ble_gattc_conn_cfg_t {
            write_cmd_tx_queue_size: 0,
        }),
        conn_gatts: Some(raw::ble_gatts_conn_cfg_t {
            hvn_tx_queue_size: 0,
        }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: 1024,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 5,
            central_role_count: 15,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: b"Brain Interface" as *const u8 as _,
            current_len: 15,
            max_len: 15,
            write_perm: raw::ble_gap_conn_sec_mode_t {
                _bitfield_1: raw::ble_gap_conn_sec_mode_t::new_bitfield_1(1, 1),
            },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        conn_l2cap: Some(raw::ble_l2cap_conn_cfg_t {
            ch_count: 1,
            rx_mps: 256,
            tx_mps: 256,
            rx_queue_size: 3,
            tx_queue_size: 20,
        }),
        ..Default::default()
    };

    let _mode: AnyPin = p.P0_19.into();
    let _busy: AnyPin = p.P0_22.into();
    let _b1: AnyPin = p.P0_09.into();
    let _b2: AnyPin = p.P0_10.into();
    let _b3: AnyPin = p.P0_23.into();
    let _b4: AnyPin = p.P1_00.into();
    let _b5: AnyPin = p.P0_21.into();
    let _b6: AnyPin = p.P0_07.into();

    let _led1: AnyPin = p.P0_00.into();
    let _led2: AnyPin = p.P0_01.into();
    let _led3: AnyPin = _busy;

    let _rhd_cs: AnyPin = _b1;
    let _rhd_clk: AnyPin = _b2;
    let _rhd_mosi: AnyPin = _b3;
    let _rhd_miso: AnyPin = _b4;

    let sd = Softdevice::enable(&config);
    let l2cap = L2cap::init(&sd);

    unwrap!(spawner.spawn(softdevice_task(sd)));
    unwrap!(spawner.spawn(blink_task(_led1, _led2, _led3, false)));

    // Enable UARTE, otherwise SPI3 will not work.
    let mut uart = UarteTx::new(p.UARTE1, Irqs, _mode, Default::default());
    let _ = uart.write(&[1, 2, 3]).await;

    let mut rhd = RHD2216::new(
        Irqs,
        p.SPI3,
        p.TIMER1,
        p.TIMER2,
        p.PPI_CH0.into(),
        p.PPI_CH1.into(),
        _rhd_cs,
        _rhd_clk,
        _rhd_mosi,
        _rhd_miso,
    );

    loop {
        info!("Waiting for connection");
        let mut config = peripheral::Config::default();
        config.tx_power = TxPower::ZerodBm;
        config.secondary_phy = Phy::M2;

        if let Ok(connection) = peripheral::advertise_connectable(sd, advertisement(), &config).await {
            info!("advertising done! I have a connection.");
            if let Err(_) = send_rhd_data(&mut rhd, &l2cap, &connection).await {
                info!("Stopped");
            }
        }
    }
}
