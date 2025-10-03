use crate::ansi::{AnsiConverter, AnsiEvent};
use crate::mccp::Decompressor;
use crate::scrollback::Scrollback;
use crate::telnet::TelnetParser;

pub struct Session<D: Decompressor> {
    decomp: D,
    telnet: TelnetParser,
    ansi: AnsiConverter,
    pub scrollback: Scrollback,
    cur_color: u8,
    line_buf: Vec<(u8, u8)>, // (char, color) pairs like C++ SET_COLOR stream
    prompt_events: usize,
}

impl<D: Decompressor> Session<D> {
    pub fn new(decomp: D, width: usize, height: usize, lines: usize) -> Self {
        Self {
            decomp,
            telnet: TelnetParser::new(),
            ansi: AnsiConverter::new(),
            scrollback: Scrollback::new(width, height, lines),
            cur_color: 0x07,
            line_buf: Vec::new(),
            prompt_events: 0,
        }
    }

    pub fn feed(&mut self, chunk: &[u8]) {
        self.decomp.receive(chunk);
        while self.decomp.pending() {
            let out = self.decomp.take_output();
            self.telnet.feed(&out);
            let prompt_count = self.telnet.drain_prompt_events();
            self.prompt_events += prompt_count;
            let app = self.telnet.take_app_out();
            for ev in self.ansi.feed(&app) {
                match ev {
                    AnsiEvent::SetColor(c) => self.cur_color = c,
                    AnsiEvent::Text(b'\n') => {
                        self.scrollback.print_line_colored(&self.line_buf);
                        self.line_buf.clear();
                    }
                    AnsiEvent::Text(b'\r') => { /* discard \r like C++ Session.cc:541 */ }
                    AnsiEvent::Text(b) => self.line_buf.push((b, self.cur_color)),
                }
            }
            // Flush line_buf on prompt events (GA/EOR) - prompts don't have trailing newlines
            if prompt_count > 0 && !self.line_buf.is_empty() {
                self.scrollback.print_line_colored(&self.line_buf);
                self.line_buf.clear();
            }
        }
    }

    pub fn drain_prompt_events(&mut self) -> usize {
        let n = self.prompt_events;
        self.prompt_events = 0;
        n
    }

    /// Get current incomplete line (not yet terminated by newline or prompt event)
    pub fn current_line(&self) -> Vec<u8> {
        self.line_buf.iter().map(|(ch, _)| *ch).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mccp::PassthroughDecomp;

    #[test]
    fn session_pipeline_basic() {
        let mut ses = Session::new(PassthroughDecomp::new(), 5, 2, 20);
        ses.feed(b"Hello\nWorld\n");
        let v = ses.scrollback.viewport_slice();
        let text: Vec<u8> = v.iter().map(|a| (a & 0xFF) as u8).collect();
        assert_eq!(&text[0..5], b"Hello");
        assert_eq!(&text[5..10], b"World");
    }

    #[test]
    fn nodeka_menu_colors() {
        // Real Nodeka output with mid-line color changes
        // Line format: [red bg spaces][reset][white text][reset][red bg spaces]\n
        let nodeka_line =
            b"\x1b[41m \x1b[0m \x1b[1;37mWelcome to Nodeka\x1b[0m: \x1b[41m \x1b[0m\n\r";

        let mut ses = Session::new(PassthroughDecomp::new(), 80, 3, 100);
        ses.feed(nodeka_line);

        // Get the stored line
        let v = ses.scrollback.viewport_slice();

        // Extract text (should have "Welcome to Nodeka")
        let text: String = v[0..80].iter().map(|a| (a & 0xFF) as u8 as char).collect();

        assert!(
            text.contains("Welcome to Nodeka"),
            "Text should contain 'Welcome to Nodeka', got: {:?}",
            text
        );

        // Check that "Welcome" part has white color (0x87 or 0x07), NOT black-on-black (0x00)
        let welcome_start = text.find('W').expect("Should find 'W'");
        let welcome_color = (v[welcome_start] >> 8) as u8;

        assert_ne!(
            welcome_color & 0x0F,
            0x00,
            "Text color should NOT be black (0x00), got: 0x{:02x}",
            welcome_color
        );

        // NOW test the conversion to ANSI - this is what get_buffer uses
        let ansi_output = crate::screen::attrib_row_to_ansi(&v[0..80]);
        assert!(
            ansi_output.contains("Welcome to Nodeka"),
            "ANSI output should contain 'Welcome to Nodeka', got: {:?}",
            ansi_output
        );
    }
}
