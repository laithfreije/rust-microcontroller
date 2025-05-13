//! # Raspberry Pi Pico Microcontroller
//!
//! This application runs on the Raspberry Pi Pico RP2040 Microcontroller
//!
#![no_std]
#![no_main]

mod clock;
mod gpio_api;

use crate::clock::ClockAPI;
use crate::gpio_api::Gpio;
use rp2040_hal::{Clock, Watchdog, entry};
use rp2040_pac::Peripherals;

/// GPIO pin number for the onboard LED
const ONBOARD_LED_NUM: usize = 25;

/// Delay in milliseconds between LED state changes
const LED_BLINK_DELAY_MS: u32 = 1000;

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
        &mut peripherals.IO_BANK0,
    );

    // Set the onboard LED to output mode
    pins.set_output(ONBOARD_LED_NUM);

    // Create a delay provider using the system clock
    let mut delay = cortex_m::delay::Delay::new(
        cortex_m::Peripherals::take().unwrap().SYST,
        clocks.clocks.system_clock.freq().to_Hz(),
    );

    loop {
        pins.set_high(ONBOARD_LED_NUM);
        delay.delay_ms(LED_BLINK_DELAY_MS);
        pins.set_low(ONBOARD_LED_NUM);
        delay.delay_ms(LED_BLINK_DELAY_MS);
    }
}
