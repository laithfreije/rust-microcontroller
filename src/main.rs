//! # Rust Microcontroller
//!
#![no_std]
#![no_main]

mod clock;

use cortex_m::delay::Delay;
use embedded_hal::digital::v2::OutputPin;

use crate::clock::ClockAPI;
use rp2040_hal::gpio::Pins;
// Import the rp2040_hal crate
use rp2040_hal::{entry, pac, Clock, Sio, Watchdog};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// The entry point of our application
#[entry]
fn _start() -> ! {
    // Grab the singleton objects for the peripherals

    let mut peripherals = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(peripherals.WATCHDOG);

    let clock_api: ClockAPI = ClockAPI::new(peripherals.XOSC, peripherals.CLOCKS,
                                            peripherals.PLL_SYS, peripherals.PLL_USB,
                                            &mut peripherals.RESETS, &mut watchdog);

    let core = pac::CorePeripherals::take().unwrap();

    // Initialize the SIO (Single-cycle I/O) block
    let sio = Sio::new(peripherals.SIO);
    
    // Set up the pins in appropriate mode
    let pins = Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );
    
    // Get the GPIO pin for the on-board LED (usually GP25)
    // Note: The Pico H has the LED on GPIO 25
    let mut led_pin = pins.gpio25.into_push_pull_output();
    
    // Create a delay object based on the ARM Cortex-M systick timer
    let mut delay = Delay::new(core.SYST, clock_api.clocks.system_clock.freq().to_Hz());
    
    // Main loop where we blink the LED
    loop {
        // Turn on the LED
        led_pin.set_high().unwrap();
        // Wait for 500ms
        delay.delay_ms(500);
        
        // Turn off the LED
        led_pin.set_low().unwrap();
        // Wait for 500ms
        delay.delay_ms(500);
    }
}