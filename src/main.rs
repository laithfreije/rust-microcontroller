//! # Raspberry Pi Pico Microcontroller
//!
//! This application runs on the Raspberry Pi Pico RP2040 Microcontroller
//!
#![no_std]
#![no_main]

mod cli;
mod clocks;
mod constants;
mod peripherals;

use crate::cli::Cli;
use crate::clocks::ClockAPI;
use crate::peripherals::gpio::Gpio;
use rp2040_hal::{Watchdog, entry};
use rp2040_pac::Peripherals;

/// GPIO pin number for the onboard LED
const ONBOARD_LED_NUM: usize = 25;

/// Panic handler that loops indefinitely
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Main entry point for the application
///
/// Initializes the system and enters the main loop
#[entry]
fn _start() -> ! {
    unsafe { cortex_m::interrupt::enable() };

    // This object is used to access peripherals such as GPIO and reset registers
    let mut peripherals = Peripherals::take().unwrap();

    // Create a new watchdog timer
    let mut watchdog = Watchdog::new(peripherals.WATCHDOG);

    // Initialize the system clocks and watchdog
    let clocks = ClockAPI::new(
        peripherals.XOSC,
        peripherals.CLOCKS,
        peripherals.PLL_SYS,
        peripherals.PLL_USB,
        &mut peripherals.RESETS,
        &mut watchdog,
    );

    // Initialize GPIO pins
    let mut pins = Gpio::new(
        peripherals.SIO,
        &mut peripherals.RESETS,
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
    );

    // Set the onboard LED to output mode
    pins.set_output(ONBOARD_LED_NUM);
    pins.set_function(0, 0b010);
    pins.set_function(1, 0b010);

    let mut cli: Cli = Cli::new(
        peripherals.UART0,
        &mut peripherals.RESETS,
        clocks.uart_clock_freq(),
    );

    pins.set_high(ONBOARD_LED_NUM);

    loop {
        cli.process_input();
    }
}
