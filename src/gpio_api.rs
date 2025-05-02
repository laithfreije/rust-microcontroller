
const NUM_PINS: usize = 30;

use rp2040_pac::io_bank0::gpio::gpio_ctrl::FUNCSEL_A::SIO as SIOFuncSel;
use rp2040_pac::{Peripherals, RESETS, SIO};

pub struct GPIO
{
    sio: SIO
}

#[allow(unused)]
impl GPIO
{
    fn write_reset_registers(resets: RESETS)
    {
        resets.reset().write(|w| unsafe {w.bits(0)});

        while(resets.reset_done().read().bits() != 0xFFFFFFFF){}
    }

    pub fn new() -> Self
    {
        let mut peripherals = Peripherals::take().unwrap();

        let resets = peripherals.RESETS;

        peripherals.SIO.gpio_oe().reset();
        peripherals.SIO.gpio_out().reset();

        resets.reset().modify(|_, w| w.pads_bank0().clear_bit());
        while resets.reset_done().read().pads_bank0().bit_is_clear() {}

        resets.reset().modify(|_, w| w.io_bank0().clear_bit());
        while resets.reset_done().read().io_bank0().bit_is_clear() {}

        for i in 0..NUM_PINS
        {
            peripherals.IO_BANK0.gpio(25).gpio_ctrl().modify(|_, w| w.funcsel().variant(SIOFuncSel));
        }

        GPIO { sio: peripherals.SIO }
    }

    pub fn set_high(&mut self, pin_num: usize)
    {
        self.sio.gpio_out_set().write(|w| unsafe {w.bits(1 << pin_num as u32)});
    }

    pub fn set_low(&mut self, pin_num: usize)
    {
        self.sio.gpio_out_clr().write(|w| unsafe {w.bits(1 << pin_num as u32)});
    }

    pub fn set_output(&mut self, pin_num: usize)
    {
        self.sio.gpio_oe_set().write(|w| unsafe {w.bits(1 << pin_num as u32)});
    }

    pub fn set_input(&mut self, pin_num: usize)
    {
        self.sio.gpio_oe_clr().write(|w| unsafe {w.bits(1 << pin_num as u32)});
    }

    pub fn read(&mut self, pin_num: usize) -> bool
    {
        todo!()
    }
}