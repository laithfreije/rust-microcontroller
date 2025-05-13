use crate::constants::MAX_LINE_LENGTH;
use crate::peripherals::uart::terminal::EscapeState::{
    EscapeAndBracketReceived, EscapeNotReceived, EscapeReceived,
};
use crate::peripherals::uart::{SerialPort, Uart};
use rp2040_pac::{RESETS, UART0};

enum ASCIICode {
    Backspace = 0x08,
    Newline = 0x0A,
    CarriageReturn = 0x0D,
    Escape = 0x1B,
    Space = 0x20,
    ArrowRight = 0x43,
    ArrowLeft = 0x44,
    Bracket = 0x5B,
    Delete = 0x7F,
}

enum ASCIIControl {
    ClearToEndOfLine,
    ClearScreen,
    MoveCursorToTop,
    ClearFormatting,
}

impl ASCIIControl {
    fn as_bytes(&self) -> &[u8] {
        match self {
            ASCIIControl::ClearToEndOfLine => b"0K",
            ASCIIControl::ClearScreen => b"2J",
            ASCIIControl::MoveCursorToTop => b"H",
            ASCIIControl::ClearFormatting => b"0m",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EscapeState {
    EscapeNotReceived,
    EscapeReceived,
    EscapeAndBracketReceived,
}

pub struct LineEditor {
    cursor: usize,
    escape_state: EscapeState,
    uart: Uart,

    current_line: heapless::Vec<u8, MAX_LINE_LENGTH>,
    prompt_color: LineEditorColor,
    cli_banner: &'static [u8],
    cli_prompt: &'static [u8],
}

#[derive(Debug)]
#[allow(unused)]
pub enum LineEditorColor {
    Red,
    Green,
    Blue,
}

impl LineEditorColor {
    fn as_bytes(&self) -> &[u8] {
        match self {
            LineEditorColor::Red => b"31m",
            LineEditorColor::Green => b"32m",
            LineEditorColor::Blue => b"34m",
        }
    }
}

impl LineEditor {
    pub fn new(
        uart_peripheral: UART0,
        uart_clock_freq: u32,
        resets: &mut RESETS,
        prompt_color: LineEditorColor,
        cli_banner: &'static [u8],
        cli_prompt: &'static [u8],
    ) -> Self {
        let uart = Uart::new(uart_peripheral, uart_clock_freq, resets);
        let current_line: heapless::Vec<u8, MAX_LINE_LENGTH> = heapless::Vec::new();
        let mut editor = LineEditor {
            cursor: 0,
            escape_state: EscapeNotReceived,
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

    fn print_prompt(&mut self) {
        let prompt = self.cli_prompt;
        self.print(prompt, true);
    }

    fn print_banner(&mut self) {
        let banner = self.cli_banner;
        self.print(banner, true);
    }

    fn print_escape_sequence(&mut self) {
        self.uart.putc(ASCIICode::Escape as u8);
        self.uart.putc(ASCIICode::Bracket as u8);
    }
    fn print_control_sequence(&mut self, control_sequence: &[u8]) {
        self.print_escape_sequence();

        self.uart.print(control_sequence);
    }

    pub fn clear_screen(&mut self) {
        self.print_control_sequence(ASCIIControl::ClearScreen.as_bytes());

        self.print_control_sequence(ASCIIControl::MoveCursorToTop.as_bytes());
        self.cursor = 0;
        self.current_line.clear();
    }

    fn apply_prompt_color(&mut self) {
        self.print_escape_sequence();
        let prompt_color = self.prompt_color.as_bytes();
        self.uart.print(prompt_color);
    }

    fn clear_formatting(&mut self) {
        self.print_control_sequence(ASCIIControl::ClearFormatting.as_bytes());
    }

    pub fn print(&mut self, s: &[u8], apply_prompt_color: bool) {
        match apply_prompt_color {
            true => {
                self.apply_prompt_color();
            }

            false => {}
        }

        self.uart.print(s);
        self.clear_formatting();
    }

    pub fn process_input(&mut self) {
        let mut bytes = self.uart.get_input();
        self.process_bytes(&mut bytes);
    }

    fn move_cursor_left(&mut self) {
        match self.cursor {
            x if x > 0 => {
                self.uart.putc(ASCIICode::Escape as u8);
                self.uart.putc(ASCIICode::Bracket as u8);
                self.uart.putc(ASCIICode::ArrowLeft as u8);
                self.cursor -= 1;
            }

            _ => {}
        }
    }
    fn move_cursor_right(&mut self) {
        match self.cursor {
            x if x < self.current_line.len() => {
                self.uart.putc(ASCIICode::Escape as u8);
                self.uart.putc(ASCIICode::Bracket as u8);
                self.uart.putc(ASCIICode::ArrowRight as u8);
                self.cursor += 1;
            }

            _ => {}
        }
    }

    fn clear_line(&mut self) {
        self.print_control_sequence(ASCIIControl::ClearToEndOfLine.as_bytes());
        self.uart.putc(ASCIICode::CarriageReturn as u8);
        self.cursor = 0;

        // Print the prompt again
        self.print_prompt();
    }

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

    fn newline(&mut self) {
        self.uart.putc(ASCIICode::CarriageReturn as u8);
        self.uart.putc(ASCIICode::Newline as u8);
        self.print_prompt();
        self.cursor = 0;

        // @todo: Trigger command processing
    }

    fn backspace(&mut self) {
        match self.cursor {
            x if x <= 0 => {
                return;
            }
            _ => {
                self.current_line.remove(self.cursor - 1);
                self.move_cursor_left();
                self.rewrite_line();
            }
        }
    }

    fn space(&mut self) {
        match self.current_line.len() {
            x if x >= MAX_LINE_LENGTH => {
                return;
            }
            _ => {
                self.current_line
                    .insert(self.cursor, ASCIICode::Space as u8)
                    .unwrap();
                self.move_cursor_right();
                self.rewrite_line();
            }
        }
    }

    fn insert_character(&mut self, data: u8) {
        match self.current_line.len() {
            x if x >= MAX_LINE_LENGTH => {
                return;
            }
            _ => {
                self.current_line.insert(self.cursor, data).unwrap();
                self.uart.putc(data);
                self.cursor += 1;
            }
        }
    }

    pub fn process_bytes(&mut self, buffer: &mut heapless::Vec<u8, MAX_LINE_LENGTH>) {
        for &data in buffer.iter() {
            match self.escape_state {
                EscapeNotReceived => match data {
                    x if x == ASCIICode::Escape as u8 => {
                        self.escape_state = EscapeReceived; // '\x1b' (ESC)
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

                EscapeReceived => match data {
                    x if x == ASCIICode::Bracket as u8 => {
                        self.escape_state = EscapeAndBracketReceived;
                    }
                    _ => {
                        self.escape_state = EscapeNotReceived;
                    }
                },

                EscapeAndBracketReceived => {
                    match data {
                        x if x == ASCIICode::ArrowLeft as u8 => {
                            self.move_cursor_left();
                        }
                        x if x == ASCIICode::ArrowRight as u8 => {
                            self.move_cursor_right();
                        }

                        _ => {}
                    }
                    self.escape_state = EscapeNotReceived;
                }
            }
        }
    }
}
