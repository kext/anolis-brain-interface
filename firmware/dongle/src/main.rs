#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod adv_data;
pub mod webusb;

use data_channel::BoxPacket;
use defmt::{info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_nrf::{
    gpio::{Level, Output, OutputDrive},
    interrupt::{self, InterruptExt},
    usb::{vbus_detect::VbusDetect, Driver},
};
use embassy_time::Instant;
use embassy_usb::{driver::EndpointError, msos, Builder, UsbDevice};
use embedded_alloc::Heap;
use nrf_softdevice::ble::{
    central::{self, ConnectConfig, ScanConfig},
    l2cap::{L2cap, RxError, SetupError},
    Address, AddressType, Connection, PhySet, TxPower,
};
use nrf_softdevice::{raw, Softdevice};
use static_cell::make_static;
use webusb::WebUsb;

// global logger
use defmt_rtt as _;
// time driver
use embassy_nrf as _;
use panic_probe as _;

#[global_allocator]
static HEAP: Heap = Heap::empty();

embassy_nrf::bind_interrupts!(struct Irqs {
    USBD => embassy_nrf::usb::InterruptHandler<embassy_nrf::peripherals::USBD>;
});

/// Simple class for bus powered devices where VBUS is always available.
struct VbusAlways {}
impl VbusDetect for VbusAlways {
    fn is_usb_detected(&self) -> bool {
        true
    }
    async fn wait_power_ready(&mut self) -> Result<(), ()> {
        Ok(())
    }
}

type MyDriver = Driver<'static, embassy_nrf::peripherals::USBD, VbusAlways>;

/// Task for the Softdevice.
#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, MyDriver>) {
    device.run().await;
}

fn start_usb(spawner: &Spawner, usbd: embassy_nrf::peripherals::USBD) -> WebUsb<'static, MyDriver> {
    // Create the driver, from the HAL.
    let driver = Driver::new(usbd, Irqs, VbusAlways {});
    interrupt::USBD.set_priority(interrupt::Priority::P2);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xbf50, 0x0b70);
    config.manufacturer = Some("VES");
    config.product = Some("Brain Interface Dongle");
    config.serial_number = None;
    config.device_release = 1;

    config.device_class = 0xFF;
    config.device_sub_class = 0x00;
    config.device_protocol = 0x00;
    config.composite_with_iads = false;

    // Create embassy-usb DeviceBuilder using the driver and config.
    let mut builder = Builder::new(
        driver,
        config,
        &mut make_static!([0; 64])[..],
        &mut make_static!([0; 64])[..],
        &mut make_static!([0; 64])[..],
        &mut make_static!([0; 192])[..],
        &mut make_static!([0; 128])[..],
    );

    builder.msos_descriptor(embassy_usb::msos::windows_version::WIN8_1, 32);
    builder.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
    builder.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
        "DeviceInterfaceGUIDs",
        msos::PropertyData::RegMultiSz(&["{8FE0B5B1-A901-4CBE-967E-3920E99F1A01}"]),
    ));

    // Create classes on the builder.
    let class = WebUsb::new(&mut builder, 64, None);

    // Build the builder.
    let usb = builder.build();

    unwrap!(spawner.spawn(usb_task(usb)));
    class
}

type MyPacket = BoxPacket<2048>;

struct ConnectionError {}
impl From<SetupError> for ConnectionError {
    fn from(_value: SetupError) -> Self {
        Self {}
    }
}
impl From<RxError> for ConnectionError {
    fn from(_value: RxError) -> Self {
        Self {}
    }
}
impl From<EndpointError> for ConnectionError {
    fn from(_value: EndpointError) -> Self {
        Self {}
    }
}

async fn handle_connection(
    l2cap: &L2cap<MyPacket>,
    connection: &Connection,
    usb_sender: &mut webusb::Sender<'static, MyDriver>,
) -> Result<(), ConnectionError> {
    let config = nrf_softdevice::ble::l2cap::Config { credits: 20 };
    let channel = l2cap.setup(connection, &config, data_channel::PSM).await?;
    let mut bytes = 0;
    let mut counter = 0;
    let mut start = None;
    loop {
        let packet = channel.rx().await?;
        let data = packet.as_bytes();
        usb_sender.write(data).await?;
        bytes += data.len();
        counter += 1;
        if counter == 100 {
            counter = 0;
            match start {
                None => {
                    start = Some(Instant::now());
                    bytes = 0;
                }
                Some(t0) => {
                    let t = Instant::now();
                    let d = t - t0;
                    info!("{}", bytes as u64 * 8000 / d.as_millis());
                    start = Some(t);
                    bytes = 0;
                }
            }
        }
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
            p_value: b"Dongle" as *const u8 as _,
            current_len: 6,
            max_len: 6,
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
            rx_queue_size: 20,
            tx_queue_size: 3,
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);
    let l2cap = L2cap::init(sd);

    unwrap!(spawner.spawn(softdevice_task(sd)));

    info!("Setting up USB");
    let mut usb = start_usb(&spawner, p.USBD);
    usb.wait_connection().await;
    let (mut usb_sender, mut usb_receiver) = usb.split();
    info!("Waiting for USB");
    let mut data = [0u8; 20];
    let _ = usb_receiver.read(&mut data).await;

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
        })
        .await
        .unwrap();
        info!("Found {:?}", addr);
        let whitelist = [&Address::new(AddressType::RandomStatic, addr)];
        config.timeout = 200;
        config.whitelist = Some(&whitelist[..]);
        config.phys = PhySet::M2;
        config.tx_power = TxPower::Plus8dBm;
        match central::connect(
            sd,
            &ConnectConfig {
                att_mtu: Some(247),
                scan_config: config,
                conn_params: raw::ble_gap_conn_params_t {
                    conn_sup_timeout: 100,
                    min_conn_interval: 40,
                    max_conn_interval: 40,
                    slave_latency: 5,
                },
            },
        )
        .await
        {
            Ok(mut connection) => {
                info!("Connected");
                if connection.phy_update(PhySet::M2, PhySet::M2).is_err() {
                    warn!("Could not upgrade to 2M PHY");
                }
                info!("MTU {}", connection.att_mtu());
                if handle_connection(&l2cap, &connection, &mut usb_sender)
                    .await
                    .is_err()
                {
                    warn!("Error in handle_connection");
                }
                let _ = connection.disconnect();
            }
            Err(_) => warn!("Connection failed"),
        }
    }
}
