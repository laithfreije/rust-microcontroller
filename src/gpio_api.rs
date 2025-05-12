const NUM_PINS: usize = 30;

use rp2040_pac::io_bank0::gpio::gpio_ctrl::FUNCSEL_A::SIO as SIOFuncSel;
use rp2040_pac::{IO_BANK0, RESETS, SIO};

pub struct Gpio {
    sio: SIO,
}

#[allow(unused)]
impl Gpio {
    fn write_reset_registers(resets: &mut RESETS) {
        resets.reset().write(|w| unsafe { w.bits(0) });
        while resets.reset_done().read().bits() != 0xFFFFFFFF {}
    }

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

    pub fn set_high(&mut self, pin_num: usize) {
        self.sio
            .gpio_out_set()
            .write(|w| unsafe { w.bits(1 << pin_num as u32) });
    }

    pub fn set_low(&mut self, pin_num: usize) {
        self.sio
            .gpio_out_clr()
            .write(|w| unsafe { w.bits(1 << pin_num as u32) });
    }

    pub fn set_output(&mut self, pin_num: usize) {
        self.sio
            .gpio_oe_set()
            .write(|w| unsafe { w.bits(1 << pin_num as u32) });
    }

    pub fn set_input(&mut self, pin_num: usize) {
        self.sio
            .gpio_oe_clr()
            .write(|w| unsafe { w.bits(1 << pin_num as u32) });
    }

    pub fn read(&mut self, pin_num: usize) -> bool {
        todo!()
    }
}
