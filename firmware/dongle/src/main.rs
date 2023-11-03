#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, warn, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Output, OutputDrive, Level};
use embassy_nrf::interrupt;
use nrf_softdevice::ble::{Address, AddressType, PhySet};
use nrf_softdevice::ble::central::{ScanConfig, self, ConnectConfig};
use nrf_softdevice::{raw, Softdevice};

// global logger
use defmt_rtt as _;
// time driver
use embassy_nrf as _;
use panic_probe as _;

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");

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
            central_role_count: 1,
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

    let sd = Softdevice::enable(&config);

    unwrap!(spawner.spawn(softdevice_task(sd)));

    let mut led = Output::new(p.P0_24, Level::Low, OutputDrive::Standard);
    led.set_high();

    let uuid = &[7, 0x3c, 0x53, 0x4c, 0xb6, 0xf0, 0x86, 0x02, 0xa1, 0x85, 0x42, 0x47, 0x83, 0x42, 0x4b, 0xb7, 0xed];

    loop {
        let mut config = ScanConfig::default();
        let addr = central::scan(sd, &config, |param| {
            if AdvDataIterator::new(get_advertisement_data(param)).find(|d| *d == uuid).is_some() {
                Some(param.peer_addr.addr)
            } else {
                None
            }
        }).await.unwrap();
        info!("Found {:?}", addr);
        let whitelist = [
            &Address::new(AddressType::RandomStatic, addr),
        ];
        config.timeout = 200;
        config.whitelist = Some(&whitelist[..]);
        config.phys = PhySet::M2;
        match central::connect(sd, &ConnectConfig {
            scan_config: config,
            conn_params: raw::ble_gap_conn_params_t {
                conn_sup_timeout: 100,
                min_conn_interval: 8,
                max_conn_interval: 8,
                slave_latency: 5,
            },
        }).await {
            Ok(connection) => {
                info!("Connected");
                info!("{}", connection.att_mtu());
                connection.disconnect().unwrap();
            },
            Err(_) => warn!("Connection failed"),
        }
    }
}

/// Iterator over advertisement data.
/// Advertisement data is a list of bytes
struct AdvDataIterator<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> AdvDataIterator<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
        }
    }
}

impl<'a> Iterator for AdvDataIterator<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<&'a [u8]> {
        if self.pos >= self.data.len() {
            return None;
        }
        let len = self.data[self.pos] as usize;
        if self.pos + len + 1 <= self.data.len() {
            let r = Some(&self.data[self.pos + 1 .. self.pos + len + 1]);
            self.pos += len + 1;
            r
        } else {
            None
        }
    }
}

/// Extract the advertisement data from a raw advertisement report.
fn get_advertisement_data(param: &raw::ble_gap_evt_adv_report_t) -> &[u8] {
    if param.data.len > 0 {
        // SAFETY: The advertisement data lives as long as the report.
        unsafe {
            core::slice::from_raw_parts(param.data.p_data, param.data.len as usize)
        }
    } else {
        &[]
    }
}