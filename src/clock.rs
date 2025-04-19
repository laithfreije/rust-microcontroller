// Import necessary libraries
use rp2040_hal::clocks::{init_clocks_and_plls, ClocksManager};
use rp2040_hal::{pac, Watchdog};
use rp_pico::XOSC_CRYSTAL_FREQ;

#[allow(unused)]
/// A struct for managing clocks in the Raspberry PI Pico H.
pub struct ClockAPI{
    // Define necessary clock variables here
    pub clocks: ClocksManager,
}

#[allow(unused)]
impl ClockAPI {
    pub fn new(
        xosc: pac::XOSC,
        clocks: pac::CLOCKS,
        pll_sys: pac::PLL_SYS,
        pll_usb: pac::PLL_USB,
        mut resets: &mut pac::RESETS, // Only this needs to be a reference
        watchdog: &mut Watchdog,  // And this
    ) -> Self {
        let clocks = init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            xosc,
            clocks,
            pll_sys,
            pll_usb,
            &mut resets,
            watchdog,
        ).ok().unwrap();

        Self { clocks }
    }
}