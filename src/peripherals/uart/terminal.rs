//! Terminal module providing line editing capabilities for UART-based CLI.
//!
//! This module implements a terminal with support for:
//! - Basic cursor movement (left/right arrows)
//! - Text insertion and deletion
//! - Color-coded prompts
//! - ANSI escape sequence handling
//! - CLI banner display

use crate::constants::MAX_LINE_LENGTH;
use crate::peripherals::uart::terminal::EscapeState::{BracketReceived, NotReceived, Received};
use crate::peripherals::uart::{SerialPort, Uart};
use rp2040_pac::{RESETS, UART0};

/// ASCII control codes used in terminal operations
enum ASCIICode {
    Backspace = 0x08,
    Newline = 0x0A,
    CarriageReturn = 0x0D,
    Escape = 0x1B,
    Space = 0x20,
    ArrowRight = 0x43,
    ArrowLeft = 0x44,
    LeftBracket = 0x5B,
    Delete = 0x7F,
}

/// ANSI terminal control sequences
enum ASCIIControl {
    /// Clears from cursor to end of line
    ClearToEndOfLine,

    /// Clears entire screen
    ClearScreen,

    /// Moves cursor to top-left position (1,1)
    MoveCursorToTop,

    /// Resets all text formatting
    ClearFormatting,
}

impl ASCIIControl {
    /// Converts the control sequence to its byte representation
    fn as_bytes(&self) -> &[u8] {
        match self {
            ASCIIControl::ClearToEndOfLine => b"0K",
            ASCIIControl::ClearScreen => b"2J",
            ASCIIControl::MoveCursorToTop => b"H",
            ASCIIControl::ClearFormatting => b"0m",
        }
    }
}

/// Represents the state of escape sequence processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EscapeState {
    /// No escape sequence in progress
    NotReceived,
    /// ESC character received
    Received,
    /// ESC [ sequence received
    BracketReceived,
}

/// Main terminal editor structure for handling terminal input/output
pub struct Terminal {
    /// Current cursor position in the line
    cursor: usize,

    /// Current state of escape sequence processing
    escape_state: EscapeState,

    /// UART peripheral instance
    uart: Uart,

    /// Current line buffer
    current_line: heapless::Vec<u8, MAX_LINE_LENGTH>,

    /// Color used for the prompt
    prompt_color: TerminalTextColor,

    /// Banner text displayed at startup
    cli_banner: &'static [u8],

    /// Prompt text displayed at the start of each line
    cli_prompt: &'static [u8],
}

/// Available colors for terminal text
#[derive(Debug)]
#[allow(unused)]
pub enum TerminalTextColor {
    Red,
    Green,
    Blue,
}

impl TerminalTextColor {
    /// Converts the color to its ANSI escape sequence
    fn as_bytes(&self) -> &[u8] {
        match self {
            TerminalTextColor::Red => b"31m",
            TerminalTextColor::Green => b"32m",
            TerminalTextColor::Blue => b"34m",
        }
    }
}

impl Terminal {
    /// Creates a new Terminal instance
    ///
    /// # Arguments
    ///
    /// * `uart_peripheral` - The UART0 peripheral to use
    /// * `uart_clock_freq` - The UART peripheral clock frequency in Hz
    /// * `resets` - Mutable reference to the RESETS peripheral
    /// * `prompt_color` - Color to use for the prompt
    /// * `cli_banner` - Banner text displayed at startup
    /// * `cli_prompt` - Prompt text displayed before each line
    ///
    /// # Returns
    ///
    /// A new Terminal instance with initialized terminal
    pub fn new(
        uart_peripheral: UART0,
        uart_clock_freq: u32,
        resets: &mut RESETS,
        prompt_color: TerminalTextColor,
        cli_banner: &'static [u8],
        cli_prompt: &'static [u8],
    ) -> Self {
        let uart = Uart::new(uart_peripheral, uart_clock_freq, resets);
        let current_line: heapless::Vec<u8, MAX_LINE_LENGTH> = heapless::Vec::new();
        let mut editor = Terminal {
            cursor: 0,
            escape_state: NotReceived,
            uart,
            current_line,
            prompt_color,
            cli_banner,
            cli_prompt,
        };

        editor.clear_screen();
        editor.print_banner();
        editor.print_prompt();
        editor
    }

    /// Prints the prompt at the beginning of the line
    fn print_prompt(&mut self) {
        let prompt = self.cli_prompt;
        self.print(prompt, true);
    }

    /// Prints the CLI banner at startup
    fn print_banner(&mut self) {
        let banner = self.cli_banner;
        self.print(banner, true);
    }

    /// Prints the escape sequence ESC + [
    fn print_escape_sequence(&mut self) {
        self.uart.putc(ASCIICode::Escape as u8);
        self.uart.putc(ASCIICode::LeftBracket as u8);
    }

    /// Prints a control sequence ESC + [ + control_sequence
    fn print_control_sequence(&mut self, control_sequence: &[u8]) {
        self.print_escape_sequence();
        self.uart.print(control_sequence);
    }

    /// Clears the screen
    pub fn clear_screen(&mut self) {
        self.print_control_sequence(ASCIIControl::ClearScreen.as_bytes());
        self.print_control_sequence(ASCIIControl::MoveCursorToTop.as_bytes());
        self.cursor = 0;
        self.current_line.clear();
    }

    /// Applies the color passed at initialisation to the prompt text
    fn apply_prompt_color(&mut self) {
        self.print_escape_sequence();
        let prompt_color = self.prompt_color.as_bytes();
        self.uart.print(prompt_color);
    }

    /// Clears all text formatting
    fn clear_formatting(&mut self) {
        self.print_control_sequence(ASCIIControl::ClearFormatting.as_bytes());
    }

    /// Prints text to the terminal
    ///
    /// # Arguments
    ///
    /// * `s` - Text to print
    /// * `apply_prompt_color` - Whether to apply the configured prompt color to the text
    pub fn print(&mut self, s: &[u8], apply_prompt_color: bool) {
        if apply_prompt_color {
            self.apply_prompt_color();
        }

        self.uart.print(s);
        self.clear_formatting();
    }

    /// Processes input from the UART
    ///
    /// This method should be called regularly to handle incoming characters
    /// and update the terminal state.
    pub fn process_input(&mut self) {
        let mut bytes = self.uart.get_input();
        self.process_bytes(&mut bytes);
    }

    /// Moves the cursor one character to the right
    fn move_cursor_left(&mut self) {
        match self.cursor {
            x if x > 0 => {
                self.uart.putc(ASCIICode::Escape as u8);
                self.uart.putc(ASCIICode::LeftBracket as u8);
                self.uart.putc(ASCIICode::ArrowLeft as u8);
                self.cursor -= 1;
            }

            _ => {}
        }
    }

    /// Moves the cursor one character to the right
    fn move_cursor_right(&mut self) {
        match self.cursor {
            x if x < self.current_line.len() => {
                self.uart.putc(ASCIICode::Escape as u8);
                self.uart.putc(ASCIICode::LeftBracket as u8);
                self.uart.putc(ASCIICode::ArrowRight as u8);
                self.cursor += 1;
            }

            _ => {}
        }
    }

    /// Erases all characters in the current line and moves the cursor to the beginning of the line
    fn clear_line(&mut self) {
        self.print_control_sequence(ASCIIControl::ClearToEndOfLine.as_bytes());

        // The carriage return after the ClearToEndOfLine sequence resets the cursor position
        self.uart.putc(ASCIICode::CarriageReturn as u8);
        self.cursor = 0;

        // Print the prompt again
        self.print_prompt();
    }

    /// Rewrites characters from the current cursor position to the end of the line
    fn rewrite_line(&mut self) {
        let original_cursor = self.cursor;

        // Clear the line
        self.clear_line();

        // Rewrite the contents
        for &byte in self.current_line.iter().skip(self.cursor) {
            self.cursor += 1;
            self.uart.putc(byte);
        }

        self.cursor = self.current_line.len();
        while self.cursor > original_cursor {
            self.move_cursor_left();
        }
    }

    /// Moves the cursor to the beginning of the next line
    fn newline(&mut self) {
        self.uart.putc(ASCIICode::CarriageReturn as u8);
        self.uart.putc(ASCIICode::Newline as u8);
        self.print_prompt();
        self.cursor = 0;

        // @todo: Trigger command processing
    }

    /// Deletes the previous character and moves the cursor left
    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.current_line.remove(self.cursor - 1);
            self.move_cursor_left();
            self.rewrite_line();
        }
    }

    /// Inserts a space at the current character position
    fn space(&mut self) {
        if self.current_line.len() < MAX_LINE_LENGTH {
            self.current_line
                .insert(self.cursor, ASCIICode::Space as u8)
                .unwrap();
            self.move_cursor_right();
            self.rewrite_line();
        }
    }

    /// Inserts a character at the current cursor position
    fn insert_character(&mut self, data: u8) {
        if self.current_line.len() < MAX_LINE_LENGTH {
            self.current_line.insert(self.cursor, data).unwrap();
            self.uart.putc(data);
            self.cursor += 1;
        }
    }

    /// Processes a buffer of input bytes
    ///
    /// # Arguments
    ///
    /// * `buffer` - Buffer containing input bytes to process
    pub fn process_bytes(&mut self, buffer: &mut heapless::Vec<u8, MAX_LINE_LENGTH>) {
        for &data in buffer.iter() {
            match self.escape_state {
                NotReceived => match data {
                    x if x == ASCIICode::Escape as u8 => {
                        self.escape_state = Received; // '\x1b' (ESC)
                    }

                    x if x == ASCIICode::CarriageReturn as u8 || x == ASCIICode::Newline as u8 => {
                        self.newline();
                        self.current_line.clear();
                    }

                    x if x == ASCIICode::Backspace as u8 || x == ASCIICode::Delete as u8 => {
                        self.backspace();
                    }

                    x if x == ASCIICode::Space as u8 => {
                        self.space();
                    }

                    0x21..=0x7E => {
                        self.insert_character(data);
                    }

                    _ => {}
                },

                Received => match data {
                    x if x == ASCIICode::LeftBracket as u8 => {
                        self.escape_state = BracketReceived;
                    }
                    _ => {
                        self.escape_state = NotReceived;
                    }
                },

                BracketReceived => {
                    match data {
                        x if x == ASCIICode::ArrowLeft as u8 => {
                            self.move_cursor_left();
                        }
                        x if x == ASCIICode::ArrowRight as u8 => {
                            self.move_cursor_right();
                        }

                        _ => {}
                    }
                    self.escape_state = NotReceived;
                }
            }
        }
    }
}
