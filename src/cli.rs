//! Command Line Interface (CLI) module
//!
//! This module provides a high-level interface for a UART-based command line interface,
//! featuring a customized shell prompt and banner. It wraps the lower-level terminal
//! functionality into a user-friendly CLI interface.
use crate::peripherals::uart::terminal::{Terminal, TerminalTextColor};

use rp2040_pac::{RESETS, UART0};

/// ASCII art banner displayed when the CLI starts
///
/// Displays a decorative box containing the shell name and description
const CLI_BANNER: &[u8] = concat!(
    "╔══════════════════════════╗\r\n",
    "║        PICO SHELL        ║\r\n",
    "║  Embedded UART Console   ║\r\n",
    "╚══════════════════════════╝\r\n",
)
.as_bytes();

/// Command prompt string displayed before each input line
const CLI_PROMPT: &[u8] = b"[PICO]$ ";

/// Main CLI structure that handles the command-line interface
///
/// Provides a high-level interface for interacting with the UART console,
/// managing the terminal and command processing.
pub struct Cli {
    /// The underlying terminal instance
    line_editor: Terminal,
}

impl Cli {
    /// Creates a new CLI instance
    ///
    /// # Arguments
    ///
    /// * `uart_peripheral` - The UART0 peripheral to use for communication
    /// * `resets` - Reference to the RESETS peripheral for initialization
    /// * `uart_clock_freq` - The UART peripheral clock frequency in Hz
    ///
    /// # Returns
    ///
    /// A new `Cli` instance configured with the default banner and blue prompt
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut cli = Cli::new(uart0, &mut resets, clocks.uart_clock_freq());
    /// ```
    pub fn new(uart_peripheral: UART0, resets: &mut RESETS, uart_clock_freq: u32) -> Self {
        let line_editor = Terminal::new(
            uart_peripheral,
            uart_clock_freq,
            resets,
            TerminalTextColor::Blue,
            CLI_BANNER,
            CLI_PROMPT,
        );
        Cli { line_editor }
    }

    /// Prints text to the CLI
    ///
    /// # Arguments
    ///
    /// * `s` - Byte slice containing the text to print
    #[allow(unused)]
    pub fn print(&mut self, s: &[u8]) {
        self.line_editor.print(s, false);
    }

    /// Processes any pending input from the UART
    ///
    /// This method should be called regularly (e.g., in the main loop) to handle
    /// incoming characters and update the CLI state.
    pub fn process_input(&mut self) {
        self.line_editor.process_input();
    }
}
