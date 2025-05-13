//! GPIO (General Purpose Input/Output) control module.
//!
//! This module provides a safe interface for controlling GPIO pins
//! on the RP2040 microcontroller.

use rp2040_pac::io_bank0::gpio::gpio_ctrl::FUNCSEL_A::SIO as SIOFuncSel;
use rp2040_pac::{IO_BANK0, RESETS, SIO};

/// Maximum number of GPIO pins available on the RP2040
const NUM_PINS: usize = 30;

/// Manages GPIO operations for the RP2040 microcontroller.
///
/// Provides methods for configuring and controlling GPIO pins,
/// including setting pin directions and reading/writing pin states.
pub struct Gpio {
    /// The SIO (Single-cycle Input/Output) peripheral
    sio: SIO,
}

#[allow(unused)]
impl Gpio {
    /// Writes to reset registers and waits for completion.
    ///
    /// # Arguments
    ///
    /// * `resets` - The reset controller peripheral
    fn write_reset_registers(resets: &mut RESETS) {
        resets.reset().write(|w| unsafe { w.bits(0) });
        while resets.reset_done().read().bits() != 0xFFFFFFFF {}
    }

    /// Creates a new GPIO manager instance.
    ///
    /// Initializes the GPIO system by:
    /// - Resetting the SIO peripheral
    /// - Configuring pad controls
    /// - Setting up IO bank functionality
    ///
    /// # Arguments
    ///
    /// * `sio` - The SIO peripheral
    /// * `resets` - The reset controller
    /// * `io_bank0` - The IO bank peripheral
    ///
    /// # Returns
    ///
    /// A new `Gpio` instance
    pub fn new(sio: SIO, resets: &mut RESETS, io_bank0: &mut IO_BANK0) -> Self {
        // Initialize SIO
        sio.gpio_oe().reset();
        sio.gpio_out().reset();

        // Reset pads_bank0
        resets.reset().modify(|_, w| w.pads_bank0().clear_bit());
        while resets.reset_done().read().pads_bank0().bit_is_clear() {}

        // Reset io_bank0
        resets.reset().modify(|_, w| w.io_bank0().clear_bit());
        while resets.reset_done().read().io_bank0().bit_is_clear() {}

        // Configure GPIO functions
        for i in 0..NUM_PINS {
            io_bank0
                .gpio(i)
                .gpio_ctrl()
                .modify(|_, w| w.funcsel().variant(SIOFuncSel));
        }

        Gpio { sio }
    }

    /// Sets a GPIO pin to high state.
    ///
    /// # Arguments
    ///
    /// * `pin_num` - The GPIO pin number (0-29)
    pub fn set_high(&mut self, pin_num: usize) {
        self.sio
            .gpio_out_set()
            .write(|w| unsafe { w.bits(1 << pin_num as u32) });
    }

    /// Sets a GPIO pin to low state.
    ///
    /// # Arguments
    ///
    /// * `pin_num` - The GPIO pin number (0-29)
    pub fn set_low(&mut self, pin_num: usize) {
        self.sio
            .gpio_out_clr()
            .write(|w| unsafe { w.bits(1 << pin_num as u32) });
    }

    /// Configures a GPIO pin as an output.
    ///
    /// # Arguments
    ///
    /// * `pin_num` - The GPIO pin number (0-29)
    pub fn set_output(&mut self, pin_num: usize) {
        self.sio
            .gpio_oe_set()
            .write(|w| unsafe { w.bits(1 << pin_num as u32) });
    }

    /// Configures a GPIO pin as an input.
    ///
    /// # Arguments
    ///
    /// * `pin_num` - The GPIO pin number (0-29)
    pub fn set_input(&mut self, pin_num: usize) {
        self.sio
            .gpio_oe_clr()
            .write(|w| unsafe { w.bits(1 << pin_num as u32) });
    }

    /// Reads the current state of a GPIO pin.
    ///
    /// # Arguments
    ///
    /// * `pin_num` - The GPIO pin number (0-29)
    ///
    /// # Returns
    ///
    /// Returns `true` if the pin is high, `false` if the pin is low
    pub fn read(&mut self, pin_num: usize) -> bool {
        todo!()
    }
}
