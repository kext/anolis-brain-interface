//! The firmware for the dongle.
//!
//! Scans for a brain interface and connects to it.
//! The data is then send over USB to the connected PC.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod adv_data;
pub mod webusb;

use core::cell::RefCell;

use critical_section::Mutex;
use data_channel::BoxPacket;
use defmt::{info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_nrf::{
    gpio::{Level, Output, OutputDrive},
    interrupt::{self, InterruptExt},
    usb::{vbus_detect::VbusDetect, Driver},
};
use embassy_time::{Duration, Instant, Timer};
use embassy_usb::{driver::EndpointError, msos, Builder, UsbDevice};
use embedded_alloc::Heap;
use nrf_softdevice::ble::{
    central::{self, ConnectConfig, ScanConfig},
    l2cap::{self, L2cap, RxError, SetupError},
    Address, AddressType, PhySet, TxPower,
};
use nrf_softdevice::{raw, Softdevice};
use static_cell::make_static;
use webusb::WebUsb;

// global logger
use defmt_rtt as _;
// time driver
use embassy_nrf as _;
use panic_probe as _;

/// Use the embedded alloc heap to enable [`BoxPacket`].
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

/// Driver for [embassy_usb].
type MyDriver = Driver<'static, embassy_nrf::peripherals::USBD, VbusAlways>;

/// Task for the Softdevice.
#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

/// Task for the USB interface.
#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, MyDriver>) {
    device.run().await;
}

/// Enable the USB interface.
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

/// Alias for the packet type to have one place to change the size.
type MyPacket = BoxPacket<2048>;

/// Shared state for communication between the tasks.
struct State {
    /// Time of the last activity on the USB.
    last_usb_activity: Option<Instant>,
}
impl State {
    /// Create a new state.
    const fn new() -> Self {
        Self {
            last_usb_activity: None,
        }
    }
}

/// Global shared state.
static STATE: Mutex<RefCell<State>> = Mutex::new(RefCell::new(State::new()));
/// USB timeout.
const USB_TIMEOUT: Duration = Duration::from_secs(1);

fn usb_active() -> bool {
    critical_section::with(|cs| {
        let state = STATE.borrow_ref(cs);
        state
            .last_usb_activity
            .map_or(false, |t| t + USB_TIMEOUT > Instant::now())
    })
}

#[embassy_executor::task]
async fn usb_read_task(mut receiver: webusb::Receiver<'static, MyDriver>) -> ! {
    loop {
        let mut data = [0u8; 20];
        let _ = receiver.read(&mut data).await;
        critical_section::with(|cs| {
            let mut state = STATE.borrow_ref_mut(cs);
            state.last_usb_activity.replace(Instant::now());
        });
    }
}

/// Unified error type for [`handle_connection`].
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

/// Receive data from the L2CAP channel and forward it to the USB interface.
async fn handle_connection(
    data_channel: l2cap::Channel<MyPacket>,
    command_channel: l2cap::Channel<MyPacket>,
    usb_sender: &mut webusb::Sender<'static, MyDriver>,
) -> Result<(), ConnectionError> {
    let mut stopped = false;
    loop {
        if !usb_active() && !stopped {
            command_channel
                .tx(MyPacket::new().ok_or(ConnectionError {})?)
                .await
                .map_err(|_| ConnectionError {})?;
            stopped = true;
        }
        let packet = data_channel.rx().await?;
        usb_sender.write(&packet).await?;
    }
}

/// The main task.
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
            periph_role_count: 1,
            central_role_count: 1,
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
            ch_count: 2,
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
    let (mut usb_sender, usb_receiver) = usb.split();
    info!("Waiting for USB");

    unwrap!(spawner.spawn(usb_read_task(usb_receiver)));

    let mut led = Output::new(p.P0_24, Level::Low, OutputDrive::Standard);
    led.set_high();

    loop {
        while !usb_active() {
            Timer::after_millis(100).await;
        }
        info!("Connecting ...");
        let mut config = ScanConfig::default();
        config.timeout = 200;
        let addr = match central::scan(sd, &config, |adv_report| {
            if adv_data::supports_data_service(adv_report) {
                Some(adv_report.peer_addr.addr)
            } else {
                None
            }
        })
        .await
        {
            Ok(addr) => addr,
            Err(_) => continue,
        };
        info!("Found {:?}", addr);
        let whitelist = [&Address::new(AddressType::RandomStatic, addr)];
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
                    slave_latency: 0,
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
                let config = nrf_softdevice::ble::l2cap::Config { credits: 20 };
                let data_channel = l2cap.setup(&connection, &config, 1).await;
                let command_channel = l2cap.setup(&connection, &config, 2).await;
                if let (Ok(data_channel), Ok(command_channel)) = (data_channel, command_channel) {
                    if handle_connection(data_channel, command_channel, &mut usb_sender)
                        .await
                        .is_err()
                    {
                        info!("Connection ended");
                    }
                }
            }
            Err(_) => warn!("Connection failed"),
        }
    }
}
