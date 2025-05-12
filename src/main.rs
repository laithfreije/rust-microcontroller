//! # Rust Microcontroller
//! This is a base application that runs on the Raspberry Pi Pico H microcontroller.
#![no_std]
#![no_main]

mod clock;

mod gpio_api;

use crate::gpio_api::Gpio;
// Import the rp2040_hal crate
use crate::clock::ClockAPI;
use rp2040_hal::{Clock, Watchdog, entry};
use rp2040_pac::Peripherals;

// The onboard LED on the pico H microcontroller is pin 25
const ONBOARD_LED_NUM: usize = 25;

// Blink the LED every 1 second
const LED_BLINK_DELAY_MS: u32 = 1000;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

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
