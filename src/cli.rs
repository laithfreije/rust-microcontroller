use crate::peripherals::uart::terminal::{LineEditor, LineEditorColor};

use rp2040_pac::{RESETS, UART0};

const CLI_BANNER: &[u8] = concat!(
    "╔══════════════════════════╗\r\n",
    "║        PICO SHELL        ║\r\n",
    "║  Embedded UART Console   ║\r\n",
    "╚══════════════════════════╝\r\n",
)
.as_bytes();

const CLI_PROMPT: &[u8] = b"[PICO]$ ";

pub struct Cli {
    line_editor: LineEditor,
}

impl Cli {
    pub fn new(uart_peripheral: UART0, resets: &mut RESETS, uart_clock_freq: u32) -> Self {
        let line_editor = LineEditor::new(
            uart_peripheral,
            uart_clock_freq,
            resets,
            LineEditorColor::Blue,
            CLI_BANNER,
            CLI_PROMPT,
        );
        Cli { line_editor }
    }

    #[allow(unused)]
    pub fn print(&mut self, s: &[u8]) {
        self.line_editor.print(s, false);
    }

    pub fn process_input(&mut self) {
        self.line_editor.process_input();
    }
}
