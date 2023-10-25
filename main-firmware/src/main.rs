#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

#![macro_use]

extern crate alloc;

use defmt_rtt as _; // global logger
use embassy_nrf as _; // time driver
use panic_probe as _;

use core::mem;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::{interrupt, bind_interrupts};
use embassy_nrf::gpio::{Level, Output, OutputDrive, AnyPin};
use embassy_time::{Duration, Timer};
use nrf_softdevice::{raw, Softdevice};
use embedded_alloc::Heap;

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
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_DEFAULT.into(),
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: raw::BLE_GAP_ADV_SET_COUNT_DEFAULT as u8,
            periph_role_count: raw::BLE_GAP_ROLE_COUNT_PERIPH_DEFAULT as u8,
            central_role_count: 3,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: b"HelloRust" as *const u8 as _,
            current_len: 9,
            max_len: 9,
            write_perm: unsafe { mem::zeroed() },
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

    unwrap!(spawner.spawn(softdevice_task(sd)));

    unwrap!(spawner.spawn(blink_task(_led1, _led2, _led3, false)));

    let mut rhd = RHD2216::new(
        Irqs,
        p.SPI3, p.TIMER1, p.TIMER2, p.PPI_CH0.into(), p.PPI_CH1.into(),
        _rhd_cs, _rhd_clk, _rhd_mosi, _rhd_miso
    );
    rhd.start();

    let mut count = 0usize;
    loop {
        for _ in 0..100 {
            let v = rhd.read().await;
            count += v.len();
        }
        info!("{} samples received", count);
        //Timer::after(Duration::from_millis(250)).await;
        //rhd.stop();
    }
}
