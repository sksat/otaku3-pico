#![no_std]
#![no_main]

use dht20::Dht20;
// Hardware Abstraction Layer
use rp_pico::hal;
// periheral access crate
use hal::pac;

// on panic, just halt
use panic_halt as _;

// traits
use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use hal::clocks::Clock;
use hal::fugit::RateExtU32;
use hal::uart::*;

// Entry point to our bare-metal application.
// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
// as soon as all global variables are initialised.
#[hal::entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut timer = hal::timer::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart_pins = (
        // UART TX (characters sent from RP2040) on pin 1 (GPIO0)
        pins.gpio0.into_function(),
        // UART RX (characters received by RP2040) on pin 2 (GPIO1)
        pins.gpio1.into_function(),
    );

    let mut uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    let sda_pin: hal::gpio::Pin<_, hal::gpio::FunctionI2C, _> = pins.gpio16.reconfigure();
    let scl_pin: hal::gpio::Pin<_, hal::gpio::FunctionI2C, _> = pins.gpio17.reconfigure();

    let i2c = hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    let mut sensor = Dht20::new(i2c, 0x38, timer);

    loop {
        let data = sensor.read().unwrap();
        writeln!(uart, "temperature = {}\r", data.temp).unwrap();
        writeln!(uart, "humidity = {}\r", data.hum).unwrap();
        timer.delay_ms(1000);
    }
}
