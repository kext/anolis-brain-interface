use embassy_nrf::{timer, peripherals, pac, Peripheral, PeripheralRef, into_ref, interrupt};
use embassy_nrf::ppi::{Ppi, AnyConfigurableChannel, Event, Task};
use embassy_nrf::gpio::{AnyPin, Port, Pin};
use embassy_nrf::interrupt::typelevel::Interrupt;
use embassy_sync::channel::{Channel, ReceiveFuture};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use core::ptr::NonNull;
use core::marker::PhantomData;
use core::cell::RefCell;
use core::arch::asm;
use alloc::vec::Vec;
use critical_section::Mutex;

pub struct RHD2216<'d> {
    // We need two timers.
    // Timer 1 generates the SPI transaction interval.
    timer1: timer::Timer<'d, peripherals::TIMER1>,
    // Timer 2 counts the SPI transactions.
    timer2: timer::Timer<'d, peripherals::TIMER2>,
    // We need two PPI channels.
    // Channel 1 triggers the SPI transaction.
    ppi1: Ppi<'d, AnyConfigurableChannel, 1, 1>,
    // Channel 2 increments the counter after a transaction.
    ppi2: Ppi<'d, AnyConfigurableChannel, 1, 1>,
    // The pins can be any pins.
    cs: AnyPin,
    clk: AnyPin,
    mosi: AnyPin,
    miso: AnyPin,
    // The SPI bus must be SPI3 because it is the only one with support for the CS pin.
    _spi: PeripheralRef<'d, peripherals::SPI3>,
}

fn spi_registers() -> &'static pac::spim3::RegisterBlock {
    unsafe { &*pac::SPIM3::ptr() }
}

fn timer1_registers() -> &'static pac::timer2::RegisterBlock {
    unsafe { &*pac::TIMER1::ptr() }
}

fn timer2_registers() -> &'static pac::timer2::RegisterBlock {
    unsafe { &*pac::TIMER2::ptr() }
}

// Functions to generate commands as u16.
// Most of these intentionally use LE byte order with swapped bytes.
const fn convert_channel(c: u8) -> u16 { (c as u16).to_le() }
const fn read_register(r: u8) -> u16 { (r as u16 | 192).to_le() }
const fn write_register(r: u8, d: u8) -> u16 { (((d as u16) << 8) | (r as u16) | 128).to_le() }
const fn start_calibration() -> u16 { 0b01010101u16 }
const fn dummy_command() -> u16 { read_register(40) }

#[derive(PartialEq)]
enum State {
    Off, // The ADC is stopped.
    Starting, // The ADC is executing the calibration sequence.
    Rx1, // Receiving into buffer rx1.
    Rx2, // Receiving into buffer rx2.
}

const CHANNEL_COUNT: usize = 16;
const FRAMES_PER_BUFFER: usize = 100;
const STRIDE: usize = 20;
const BUFFER_SIZE: usize = FRAMES_PER_BUFFER * STRIDE;
const OVERFLOW: usize = BUFFER_SIZE / 10;
const TOTAL_BUFFER: usize = BUFFER_SIZE + OVERFLOW;
const OFFSET: u32 = BUFFER_SIZE as u32 * 2;

struct SpiBuffers {
    tx: [u16; TOTAL_BUFFER],
    rx1: [u16; TOTAL_BUFFER],
    rx2: [u16; TOTAL_BUFFER],
    state: State,
}

static SPI_BUFFERS: Mutex<RefCell<SpiBuffers>> = Mutex::new(RefCell::new(SpiBuffers {
    tx: [0u16; TOTAL_BUFFER],
    rx1: [0u16; TOTAL_BUFFER],
    rx2: [0u16; TOTAL_BUFFER],
    state: State::Off,
}));
static CHANNEL: Channel<CriticalSectionRawMutex, Vec<u16>, 4> = Channel::new();

impl SpiBuffers {
    fn tx_address(&self) -> u32 {
        &self.tx as *const _ as u32
    }
    fn rx1_address(&self) -> u32 {
        &self.rx1 as *const _ as u32
    }
    fn rx2_address(&self) -> u32 {
        &self.rx2 as *const _ as u32
    }
    fn fill_startup_commands(b: &mut [u16]) {
        for i in 0..b.len() {
            b[i] = match i {
                // Write all the registers
                10 => write_register(0, 0b11011110),
                11 => write_register(1, 8),
                12 => write_register(2, 32),
                13 => write_register(3, 0),
                14 => write_register(4, 0),
                15 => write_register(5, 0),
                16 => write_register(6, 0),
                17 => write_register(7, 0),
                // Upper Cutoff: 3kHz
                18 => write_register(8, 3),
                19 => write_register(9, 1),
                20 => write_register(10, 13),
                21 => write_register(11, 1),
                // Lower Cutoff: 1Hz
                22 => write_register(12, 44),
                23 => write_register(13, 6),
                // Channel Mask
                24 => write_register(14, (((1 << CHANNEL_COUNT) - 1) & 255) as u8),
                25 => write_register(15, ((((1 << CHANNEL_COUNT) - 1) >> 8) & 255) as u8),
                26 => write_register(16, ((((1 << CHANNEL_COUNT) - 1) >> 16) & 255) as u8),
                27 => write_register(17, ((((1 << CHANNEL_COUNT) - 1) >> 24) & 255) as u8),
                // Leave at least 100µs before calibration starts
                200 => start_calibration(),
                _ => dummy_command(),
            }
        }
    }
    fn fill_readout_commands(b: &mut [u16]) {
        for i in 0..b.len() {
            let n = i % STRIDE;
            b[i] = if n < CHANNEL_COUNT {
                convert_channel(n as u8)
            } else {
                dummy_command()
            }
        }
    }
    unsafe fn setup(&mut self) {
        let r = spi_registers();
        if self.state != State::Off {
            panic!("Trying to start RHD while it is already running.");
        }
        Self::fill_startup_commands(&mut self.tx[0..BUFFER_SIZE]);
        Self::fill_readout_commands(&mut self.tx[BUFFER_SIZE..]);
        self.state = State::Starting;
        r.txd.ptr.write(|w| unsafe { w.bits(self.tx_address()) });
        r.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(2) });
        r.txd.list.write(|w| w.list().array_list());
        r.rxd.ptr.write(|w| unsafe { w.bits(self.rx1_address()) });
        r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(2) });
        r.rxd.list.write(|w| w.list().array_list());
    }
    unsafe fn update(&mut self) {
        let r = spi_registers();
        match self.state {
            State::Off => {
                // Should only happen if stop was called while interrupt was
                // already signalled but not served yet.
            },
            State::Starting => {
                self.state = State::Rx1;
                Self::fill_readout_commands(&mut self.tx[0..BUFFER_SIZE]);
                adjust_pointer(r.txd.ptr.as_ptr(), 0u32.wrapping_sub(OFFSET));
                let n = adjust_pointer(r.rxd.ptr.as_ptr(), 0u32.wrapping_sub(OFFSET))
                    .wrapping_sub(self.rx1_address()) as usize;
                // Copy overflow
                for i in 0..n {
                    self.rx1[i] = self.rx1[BUFFER_SIZE + i];
                }
            },
            State::Rx1 => {
                self.state = State::Rx2;
                adjust_pointer(r.txd.ptr.as_ptr(), 0u32.wrapping_sub(OFFSET));
                let offset = self.rx2_address()
                    .wrapping_sub(self.rx1_address())
                    .wrapping_sub(OFFSET);
                let n = adjust_pointer(r.rxd.ptr.as_ptr(), offset)
                    .wrapping_sub(self.rx2_address()) as usize;
                // Copy overflow
                for i in 0..n {
                    self.rx2[i] = self.rx1[BUFFER_SIZE + i];
                }
                // Generate frame
                let mut frame = Vec::<u16>::with_capacity(FRAMES_PER_BUFFER * CHANNEL_COUNT);
                for f in 0..FRAMES_PER_BUFFER {
                    for c in 0..CHANNEL_COUNT {
                        frame.push(self.rx1[f * STRIDE + c + 2].to_be());
                    }
                }
                CHANNEL.try_send(frame).unwrap();
            },
            State::Rx2 => {
                self.state = State::Rx1;
                adjust_pointer(r.txd.ptr.as_ptr(), 0u32.wrapping_sub(OFFSET));
                let offset = self.rx1_address()
                    .wrapping_sub(self.rx2_address())
                    .wrapping_sub(OFFSET);
                let n = adjust_pointer(r.rxd.ptr.as_ptr(), offset)
                    .wrapping_sub(self.rx1_address()) as usize;
                // Copy overflow
                for i in 0..n {
                    self.rx1[i] = self.rx2[BUFFER_SIZE + i];
                }
                // Generate frame
                let mut frame = Vec::<u16>::with_capacity(FRAMES_PER_BUFFER * CHANNEL_COUNT);
                for f in 0..FRAMES_PER_BUFFER {
                    for c in 0..CHANNEL_COUNT {
                        frame.push(self.rx2[f * STRIDE + c + 2].to_be());
                    }
                }
                CHANNEL.try_send(frame).unwrap();
            },
        }
    }
}

/// Interrupt handler.
pub struct InterruptHandler {
    _phantom: PhantomData<peripherals::TIMER2>,
}

/// Adjust a pointer register in an atomic fashion.
/// Returns the value the register has been set to.
/// This is used to access the DMA pointers of the SPI interface.
/// They should only be modified if there is currently no transaction
/// happening and the timer is not close to overflowing.
unsafe fn adjust_pointer(x: *mut u32, n: u32) -> u32 {
    let r = timer1_registers();
    let capture = r.tasks_capture[3].as_ptr() as u32;
    let cc = r.cc[3].as_ptr() as u32;
    let v: u32;
    asm!(
        "2:",
        // Load the pointer
        "ldrex {v}, [{x}]",
        // Capture the timer value
        "mov {t}, #1",
        "str {t}, [{capture}]",
        "ldr {t}, [{cc}]",
        // Check for range
        "cmp {t}, #25", // Lower bound
        "blt 2b",
        "cmp {t}, #60", // Upper bound
        "bgt 2b",
        // Adjust and store the pointer
        "add {v}, {n}",
        "strex {t}, {v}, [{x}]",
        "cmp {t}, #0",
        "bne 2b",
        x = in(reg) x,
        n = in(reg) n,
        capture = in(reg) capture,
        cc = in(reg) cc,
        v = out(reg) v,
        t = out(reg) _
    );
    v
}

fn spi_start_task() -> Task<'static> {
    let r = spi_registers();
    unsafe {
        Task::new_unchecked(NonNull::new_unchecked(r.tasks_start.as_ptr()))
    }
}

fn spi_end_event() -> Event<'static> {
    let r = spi_registers();
    unsafe {
        Event::new_unchecked(NonNull::new_unchecked(r.events_end.as_ptr()))
    }
}

impl interrupt::typelevel::Handler<interrupt::typelevel::TIMER2> for InterruptHandler {
    unsafe fn on_interrupt() {
        let mut pin = embassy_nrf::gpio::Output::new(
            embassy_nrf::peripherals::P0_21::steal(),
            embassy_nrf::gpio::Level::High,
            embassy_nrf::gpio::OutputDrive::Standard,
        );
        let r = timer2_registers();

        if r.events_compare[0].read().bits() != 0 {
            r.events_compare[0].write(|w| w.events_compare().clear_bit());
            critical_section::with(|cs| {
                SPI_BUFFERS.borrow_ref_mut(cs).update();
            });
        }

        pin.set_low();
    }
}

fn timer2_enable_cc0_isr() {
    interrupt::typelevel::TIMER2::set_priority(interrupt::Priority::P2);
    unsafe { interrupt::typelevel::TIMER2::enable(); }
    let r = timer2_registers();
    r.intenset.write(|w| w.compare0().set());
}

impl<'d> RHD2216<'d> {
    pub fn new(
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::TIMER2, InterruptHandler> + 'd,
        spi: peripherals::SPI3,
        timer1: peripherals::TIMER1,
        timer2: peripherals::TIMER2,
        ppi1: AnyConfigurableChannel,
        ppi2: AnyConfigurableChannel,
        cs: AnyPin,
        clk: AnyPin,
        mosi: AnyPin,
        miso: AnyPin
    ) -> Self {
        into_ref!(spi);
        let timer1 = timer::Timer::new(timer1);
        let timer2 = timer::Timer::new_counter(timer2);
        let ppi1 = Ppi::new_one_to_one(ppi1, timer1.cc(0).event_compare(), spi_start_task());
        let ppi2 = Ppi::new_one_to_one(ppi2, spi_end_event(), timer2.task_count());
        timer1.cc(0).short_compare_clear();
        timer2.cc(0).short_compare_clear();
        Self {
            timer1: timer1,
            timer2: timer2,
            ppi1: ppi1,
            ppi2: ppi2,
            cs: cs,
            clk: clk,
            mosi: mosi,
            miso: miso,
            _spi: spi,
        }
    }

    fn spi_setup(&mut self) {
        let r = spi_registers();
        r.psel.csn.write(|w| unsafe {
            w
            .pin().bits(self.cs.pin())
            .port().bit(self.cs.port() == Port::Port1)
            .connect().connected()
        });
        r.csnpol.write(|w| w.csnpol().low());
        r.psel.sck.write(|w| unsafe {
            w
            .pin().bits(self.clk.pin())
            .port().bit(self.clk.port() == Port::Port1)
            .connect().connected()
        });
        r.psel.mosi.write(|w| unsafe {
            w
            .pin().bits(self.mosi.pin())
            .port().bit(self.mosi.port() == Port::Port1)
            .connect().connected()
        });
        r.psel.miso.write(|w| unsafe {
            w
            .pin().bits(self.miso.pin())
            .port().bit(self.miso.port() == Port::Port1)
            .connect().connected()
        });
        r.frequency.write(|w| w.frequency().m16());
        r.enable.write(|w| w.enable().enabled());
    }

    pub fn start(&mut self) {
        critical_section::with(|cs| unsafe {
            SPI_BUFFERS.borrow_ref_mut(cs).setup();
        });
        self.spi_setup();
        self.timer1.cc(0).write(80);
        self.timer2.cc(0).write(2000);
        timer2_enable_cc0_isr();
        self.ppi1.enable();
        self.ppi2.enable();
        self.timer1.set_frequency(timer::Frequency::F16MHz);
        self.timer1.start();
    }

    #[allow(dead_code)]
    pub fn stop(&mut self) {
        critical_section::with(|cs| {
            let mut x = SPI_BUFFERS.borrow_ref_mut(cs);
            x.state = State::Off;
            self.timer1.stop();
        });
    }

    pub fn read(&mut self) -> ReceiveFuture<'_, CriticalSectionRawMutex, Vec<u16>, 4> {
        CHANNEL.receive()
    }
}
