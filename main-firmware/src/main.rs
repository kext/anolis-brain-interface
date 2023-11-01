#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

extern crate alloc;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::{interrupt, bind_interrupts};
use embassy_nrf::gpio::{Level, Output, OutputDrive, AnyPin};
use embassy_time::{Duration, Timer};
use embedded_alloc::Heap;
use futures::future::join;
use nrf_softdevice::{raw, Softdevice};
use nrf_softdevice::ble::{Connection, peripheral, gatt_server};
use nrf_softdevice::ble::gatt_server::NotifyValueError;

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
            led_1.set_level(Level::from((i == 0) ^ off));
            led_2.set_level(Level::from((i == 1) ^ off));
            led_3.set_level(Level::from((i == 2) ^ off));
            Timer::after(Duration::from_millis(500)).await;
        }
    }
}

#[nrf_softdevice::gatt_service(uuid = "edb74b42-8347-4285-a102-86f0b64c533c")]
struct RhdService {
    #[characteristic(uuid = "feb7f8e1-c457-4993-b0a0-92dd89a9547c", read, notify)]
    liveview: [u8; rhd2216::CHANNEL_COUNT * 4 + 1],
}

#[nrf_softdevice::gatt_server]
struct Server {
    service: RhdService,
}

async fn rhd_task<'a>(rhd: &'a mut RHD2216<'_>, server: &'a Server, connection: &'a Connection) {
    rhd.start();
    let mut counter = 0u8;
    let mut lv = [0u8; rhd2216::CHANNEL_COUNT * 4 + 1];
    loop {
        let d = rhd.read().await;
        lv[0] = counter;
        counter = counter.wrapping_add(1);
        for i in 0..d.channels {
            let mut min = u16::MAX;
            let mut max = u16::MIN;
            for j in (i..d.frames.len()).step_by(d.channels) {
                let v = d.frames[j];
                if v < min { min = v; }
                if v > max { max = v; }
            }
            lv[(i * 4 + 1)..(i * 4 + 3)].copy_from_slice(&min.to_le_bytes());
            lv[(i * 4 + 3)..(i * 4 + 5)].copy_from_slice(&max.to_le_bytes());
        }
        if let Err(NotifyValueError::Disconnected) = server.service.liveview_notify(connection, &lv) {
            break;
        }
    }
    info!("Stopping");
    rhd.stop();
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");

    // Initialise allocator
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024 * 64;
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
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 1,
            event_length: 40,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t {
            att_mtu: 247,
        }),
        conn_gatts: Some(raw::ble_gatts_conn_cfg_t {
            hvn_tx_queue_size: 10,
        }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_DEFAULT,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 1,
            central_role_count: 0,
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
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(raw::BLE_GATTS_VLOC_STACK as u8),
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
    let server = unwrap!(Server::new(sd));

    unwrap!(spawner.spawn(softdevice_task(sd)));
    unwrap!(spawner.spawn(blink_task(_led1, _led2, _led3, false)));

    let mut rhd = RHD2216::new(
        Irqs,
        p.SPI3, p.TIMER1, p.TIMER2, p.PPI_CH0.into(), p.PPI_CH1.into(),
        _rhd_cs, _rhd_clk, _rhd_mosi, _rhd_miso,
    );

    #[rustfmt::skip]
    let adv_data = &[
        0x02, 0x01, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, 0x03, 0x09, 0x18,
        0x0a, 0x09, b'I', b'n', b't', b'e', b'r', b'f', b'a', b'c', b'e',
    ];
    #[rustfmt::skip]
    let scan_data = &[
        0x03, 0x03, 0x09, 0x18,
    ];

    loop {
        info!("Waiting for connection");
        let config = peripheral::Config::default();

        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected { adv_data, scan_data };
        let conn = unwrap!(peripheral::advertise_connectable(sd, adv, &config).await);
        info!("advertising done! I have a connection.");

        let rhd_future = rhd_task(&mut rhd, &server, &conn);
        let gatt_future = gatt_server::run(&conn, &server, |e| match e {
            ServerEvent::Service(e) => match e {
                RhdServiceEvent::LiveviewCccdWrite { notifications } => {
                    info!("Notifications {}", notifications);
                },
            }
        });

        join(rhd_future, gatt_future).await;
    }
}
