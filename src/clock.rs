//! Clock management module for the Raspberry Pi Pico.
//!
//! This module provides functionality for initializing and managing the system clocks
//! on the RP2040 microcontroller.

use rp_pico::XOSC_CRYSTAL_FREQ;
use rp2040_hal::clocks::{ClocksManager, init_clocks_and_plls};
use rp2040_hal::{Watchdog, pac};

/// Manages the clock configuration for the RP2040 microcontroller.
///
/// This struct encapsulates the clock manager and provides methods for
/// clock initialization and management.
#[allow(unused)]
pub struct ClockAPI {
    /// The clock manager instance that controls all system clocks
    pub clocks: ClocksManager,
}

#[allow(unused)]
impl ClockAPI {
    /// Creates a new instance of the clock manager with initialized system clocks.
    ///
    /// # Arguments
    ///
    /// * `xosc` - The external oscillator peripheral
    /// * `clocks` - The clocks peripheral
    /// * `pll_sys` - The system PLL peripheral
    /// * `pll_usb` - The USB PLL peripheral
    /// * `resets` - The reset controller
    /// * `watchdog` - The watchdog peripheral
    ///
    /// # Returns
    ///
    /// Returns a new `ClockAPI` instance with initialized clocks
    ///
    /// # Panics
    ///
    /// Panics if clock initialization fails
    pub fn new(
        xosc: pac::XOSC,
        clocks: pac::CLOCKS,
        pll_sys: pac::PLL_SYS,
        pll_usb: pac::PLL_USB,
        resets: &mut pac::RESETS,
        watchdog: &mut Watchdog,
    ) -> Self {
        let clocks = init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            xosc,
            clocks,
            pll_sys,
            pll_usb,
            resets,
            watchdog,
        )
        .ok()
        .unwrap();

        Self { clocks }
    }
}
