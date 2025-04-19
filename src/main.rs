//! # Rust Microcontroller
//! This is a base application that runs on the Raspberry Pi Pico H microcontroller.
#![no_std]
#![no_main]

mod clock;

mod gpio_api;

use crate::gpio_api::GPIO;
// Import the rp2040_hal crate
use rp2040_hal::entry;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// The entry point of our application
#[entry]
fn _start() -> ! {

    let mut pins = GPIO::new();

    pins.set_output(25);
    pins.set_high(25);
    loop {

    }
}