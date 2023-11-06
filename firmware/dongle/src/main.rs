#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use data_channel::{L2capError, BoxPacket};
use defmt::{info, warn, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Output, OutputDrive, Level};
use embassy_nrf::interrupt;
use embedded_alloc::Heap;
use nrf_softdevice::ble::l2cap::L2cap;
use nrf_softdevice::ble::{Address, AddressType, PhySet, Connection};
use nrf_softdevice::ble::central::{ScanConfig, self, ConnectConfig};
use nrf_softdevice::{raw, Softdevice};

// global logger
use defmt_rtt as _;
// time driver
use embassy_nrf as _;
use panic_probe as _;

#[global_allocator]
static HEAP: Heap = Heap::empty();

mod adv_data;

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

type MyPacket = BoxPacket<512>;

async fn handle_connection(l2cap: &L2cap<MyPacket>, connection: &Connection) -> Result<(), L2capError<MyPacket>> {
    let config = nrf_softdevice::ble::l2cap::Config { credits: 8 };
    let channel = l2cap.setup(connection, &config, data_channel::PSM).await?;
    let mut _counter = 0u8;
    loop {
        let packet = channel.rx().await?;
        let data = packet.as_bytes();
        /*if data[0] != counter {
            info!("Dropped packet");
        }
        counter = data[0].wrapping_add(1);*/
        info!("{:?}", data);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");

    // Initialise allocator.
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024 * 64;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    // Get the peripherals access crate.
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
            att_mtu: 114
        }),
        conn_gattc: Some(raw::ble_gattc_conn_cfg_t {
            write_cmd_tx_queue_size: 0,
        }),
        conn_gatts: Some(raw::ble_gatts_conn_cfg_t {
            hvn_tx_queue_size: 0
        }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t { attr_tab_size: 1024 }),
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
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(raw::BLE_GATTS_VLOC_STACK as u8),
        }),
        conn_l2cap: Some(raw::ble_l2cap_conn_cfg_t {
            ch_count: 1,
            rx_mps: 256,
            tx_mps: 256,
            rx_queue_size: 10,
            tx_queue_size: 10,
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);
    let l2cap = L2cap::init(&sd);

    unwrap!(spawner.spawn(softdevice_task(sd)));

    let mut led = Output::new(p.P0_24, Level::Low, OutputDrive::Standard);
    led.set_high();

    loop {
        let mut config = ScanConfig::default();
        let addr = central::scan(sd, &config, |adv_report| {
            if adv_data::supports_data_service(adv_report) {
                Some(adv_report.peer_addr.addr)
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
                if handle_connection(&l2cap, &connection).await.is_err() {
                    warn!("Error in handle_connection");
                }
                let _ = connection.disconnect();
            },
            Err(_) => warn!("Connection failed"),
        }
    }
}
