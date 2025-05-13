//! UART Module
//!
//! This module provides UART (Universal Asynchronous Receiver/Transmitter) functionality
//! with interrupt-driven input handling
use rp2040_pac::{RESETS, UART0, interrupt};

use crate::constants::MAX_LINE_LENGTH;
use core::cell::RefCell;
use cortex_m::interrupt::{Mutex, free};
use heapless::spsc::Queue;

pub mod terminal;

/// Default UART baud rate
const UART_BAUD_RATE: u32 = 115200;

/// Global static queue for storing UART input received from ISR
/// Uses a mutex-protected RefCell for safe concurrent access
static INPUT_QUEUE: Mutex<RefCell<Queue<u8, MAX_LINE_LENGTH>>> =
    Mutex::new(RefCell::new(Queue::new()));

/// Represents word length configurations for UART communication
#[allow(unused)]
enum UartWordLength {
    Five = 0b00,
    Six = 0b01,
    Seven = 0b10,
    Eight = 0b11,
}

/// UART0 interrupt handler
///
/// Processes received characters and stores them in the input queue.
#[interrupt]
fn UART0_IRQ() {
    let uart = unsafe { &*UART0::ptr() };
    let masked_irq_status = uart.uartmis().read();

    let rx_interrupt_set = masked_irq_status.rxmis().bit_is_set();
    let rx_timeout_interrupt_set = masked_irq_status.rtmis().bit_is_set();
    if rx_interrupt_set || rx_timeout_interrupt_set {
        let mut is_rx_fifo_empty = uart.uartfr().read().rxfe().bit_is_set();
        while !is_rx_fifo_empty {
            let data = uart.uartdr().read().data().bits();

            // Enter interrupt-free section
            free(|cs| {
                let mut queue = INPUT_QUEUE.borrow(cs).borrow_mut();
                let _ = queue.enqueue(data);
            });

            is_rx_fifo_empty = uart.uartfr().read().rxfe().bit_is_set();
        }
    }

    uart.uarticr().write(|w| unsafe { w.bits(0xFFFF) });
}

/// Trait defining the interface for serial port operations
trait SerialPort {
    /// Sends a single byte through the serial port
    fn putc(&mut self, c: u8);

    /// Prints a byte slice to the serial port
    fn print(&mut self, s: &[u8]);

    /// Enables or disables the FIFO buffer
    fn set_fifo_enable(&mut self, enable: bool);

    /// Configures the number of stop bits (one or two)
    fn use_two_stop_bits(&mut self, use_two_stop_bits: bool);

    /// Enables or disables parity bit
    fn set_parity(&mut self, parity: bool);

    /// Configures the baud rate
    fn config_baud_rate(&mut self);

    /// Sets the word length for UART communication
    fn config_word_length(&mut self, word_length: UartWordLength);

    /// Configures UART parameters (word length, FIFO, stop bits, parity)
    fn config_parameters(&mut self);

    /// Configures UART interrupts
    fn config_interrupts(&mut self);

    /// Enables or disables the UART peripheral
    fn set_peripheral_enable(&mut self, enable: bool);

    /// Retrieves input from the UART buffer
    fn get_input(&mut self) -> heapless::Vec<u8, MAX_LINE_LENGTH>;
}
pub struct Uart {
    /// The UART0 peripheral instance
    uart_peripheral: UART0,

    /// The peripheral clock frequency in Hz
    peripheral_clock_freq: u32,
}

/// UART peripheral wrapper struct
impl Uart {
    /// Creates a new UART instance with the specified parameters
    ///
    /// # Arguments
    /// * `uart_peripheral` - The UART0 peripheral instance
    /// * `peripheral_clock_freq` - The peripheral clock frequency in Hz
    /// * `resets` - Mutable reference to the RESETS peripheral for resetting UART
    pub fn new(uart_peripheral: UART0, peripheral_clock_freq: u32, resets: &mut RESETS) -> Self {
        let mut uart = Uart {
            uart_peripheral,
            peripheral_clock_freq,
        };

        uart.reset_peripheral(resets);
        uart.set_peripheral_enable(false);
        uart.config_baud_rate();
        uart.config_parameters();
        uart.set_peripheral_enable(true);
        uart.config_interrupts();

        uart
    }

    /// Resets the UART peripheral
    ///
    /// # Arguments
    /// * `resets` - The RESETS peripheral
    fn reset_peripheral(&mut self, resets: &mut RESETS) {
        resets.reset().modify(|_, w| w.uart0().clear_bit());
        while resets.reset_done().read().uart0().bit_is_clear() {}
    }
}
impl SerialPort for Uart {
    fn config_baud_rate(&mut self) {
        /*
           From section 4.2.3.2.1 of rp2040-datasheet:
           The baud rate divisor is a 22-bit number consisting of a 16-bit integer and
           a 6-bit fractional part.

           UARTCLK / (16 * BaudRate) = BRD(integer) + BRD(fractional)

           You can calculate the 6-bit number by taking the fractional part of the
           required baud rate divisor and multiplying it by 64 (that is, 2^n,
           where n is the width of the UARTFBRD Register) and adding
           0.5 to account for rounding errors
           m = integer(BRD(fractional) * 2^n + 0.5), n=6
        */

        // Set integer part
        let baud_rate_divisor_integer =
            (self.peripheral_clock_freq as f32 / (16f32 * UART_BAUD_RATE as f32)) as u32;

        // Integer part
        self.uart_peripheral
            .uartibrd()
            .write(|w| unsafe { w.bits(baud_rate_divisor_integer) });

        // Calculate fractional part (round to nearest)
        let peripheral_clock_freq_float = self.peripheral_clock_freq as f32;
        let uart_baud_rate_float = UART_BAUD_RATE as f32;

        // Calculate fractional part of the divisor
        let fraction = (peripheral_clock_freq_float / (16f32 * uart_baud_rate_float))
            - (baud_rate_divisor_integer as f32);

        let uart_rounding_factor = 0.5f32;

        // Calculate the 6-bit register value representing the fractional part
        let fraction_bits = ((fraction * 64f32) + uart_rounding_factor) as u32;
        self.uart_peripheral
            .uartfbrd()
            .write(|w| unsafe { w.bits(fraction_bits) });

        // Ensure changes take effect
        let lcr = self.uart_peripheral.uartlcr_h().read().bits();
        self.uart_peripheral
            .uartlcr_h()
            .write(|w| unsafe { w.bits(lcr) });
    }

    fn set_fifo_enable(&mut self, enable: bool) {
        if enable {
            self.uart_peripheral
                .uartlcr_h()
                .modify(|_, w| w.fen().set_bit());
        } else {
            self.uart_peripheral
                .uartlcr_h()
                .modify(|_, w| w.fen().clear_bit());
        }
    }

    fn use_two_stop_bits(&mut self, use_two_stop_bits: bool) {
        if use_two_stop_bits {
            self.uart_peripheral
                .uartlcr_h()
                .modify(|_, w| w.stp2().set_bit());
        } else {
            self.uart_peripheral
                .uartlcr_h()
                .modify(|_, w| w.stp2().clear_bit());
        }
    }

    fn set_parity(&mut self, parity: bool) {
        if parity {
            self.uart_peripheral
                .uartlcr_h()
                .modify(|_, w| w.pen().set_bit());
        } else {
            self.uart_peripheral
                .uartlcr_h()
                .modify(|_, w| w.pen().clear_bit());
        }
    }

    fn config_word_length(&mut self, word_length: UartWordLength) {
        self.uart_peripheral
            .uartlcr_h()
            .modify(|_, w| unsafe { w.wlen().bits(word_length as u8) });
    }

    fn config_parameters(&mut self) {
        self.config_word_length(UartWordLength::Eight);
        self.set_fifo_enable(true);
        self.use_two_stop_bits(false);
        self.set_parity(false);
    }

    fn get_input(&mut self) -> heapless::Vec<u8, MAX_LINE_LENGTH> {
        let mut buffer = heapless::Vec::new();

        // Enter interrupt-free section
        free(|cs| {
            let mut queue = INPUT_QUEUE.borrow(cs).borrow_mut();
            while let Some(byte) = queue.dequeue() {
                let _ = buffer.push(byte);
            }
        });
        buffer
    }

    fn config_interrupts(&mut self) {
        // Clear all interrupts
        self.uart_peripheral
            .uarticr()
            .write(|w| unsafe { w.bits(0xFFFF) });

        // Enable RX interrupt
        self.uart_peripheral.uartimsc().modify(|_, w| {
            w.rxim() // RX interrupt mask
                .set_bit() // Enable RX interrupt
        });

        // Set RX Timeout Interrupt Mask
        self.uart_peripheral
            .uartimsc()
            .modify(|_, w| w.rtim().set_bit());

        // Enable UART interrupt in NVIC
        unsafe {
            // Enable the UART0 IRQ
            rp2040_pac::NVIC::unmask(rp2040_pac::Interrupt::UART0_IRQ);
        }
    }

    fn set_peripheral_enable(&mut self, enable: bool) {
        match enable {
            true => {
                self.uart_peripheral.uartcr().write(|w| {
                    w.uarten().set_bit();
                    w.txe().set_bit();
                    w.rxe().set_bit()
                });
            }
            false => {
                self.uart_peripheral.uartcr().write(|w| {
                    w.uarten().clear_bit();
                    w.txe().clear_bit();
                    w.rxe().clear_bit()
                });
            }
        }
    }

    fn putc(&mut self, c: u8) {
        // Wait until TX FIFO is not full
        while self.uart_peripheral.uartfr().read().txff().bit_is_set() {}
        // Write the character to the data register
        self.uart_peripheral
            .uartdr()
            .write(|w| unsafe { w.data().bits(c) });
    }

    fn print(&mut self, s: &[u8]) {
        for &byte in s {
            self.putc(byte);
        }
    }
}
