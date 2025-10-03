use crate::mccp::Decompressor;
use crate::telnet::TelnetParser;
use crate::ansi::{AnsiConverter, AnsiEvent};
use crate::scrollback::Scrollback;

pub struct Session<D: Decompressor> {
    decomp: D,
    telnet: TelnetParser,
    ansi: AnsiConverter,
    pub scrollback: Scrollback,
    cur_color: u8,
    line_buf: Vec<u8>,
    prompt_events: usize,
}

impl<D: Decompressor> Session<D> {
    pub fn new(decomp: D, width: usize, height: usize, lines: usize) -> Self {
        Self { decomp, telnet: TelnetParser::new(), ansi: AnsiConverter::new(), scrollback: Scrollback::new(width, height, lines), cur_color: 0x07, line_buf: Vec::new(), prompt_events: 0 }
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
                    AnsiEvent::Text(b'\n') => { self.scrollback.print_line(&self.line_buf, self.cur_color); self.line_buf.clear(); }
                    AnsiEvent::Text(b) => self.line_buf.push(b),
                }
            }
            // Flush line_buf on prompt events (GA/EOR) - prompts don't have trailing newlines
            if prompt_count > 0 && !self.line_buf.is_empty() {
                self.scrollback.print_line(&self.line_buf, self.cur_color);
                self.line_buf.clear();
            }
        }
    }

    pub fn drain_prompt_events(&mut self) -> usize { let n = self.prompt_events; self.prompt_events = 0; n }

    /// Get current incomplete line (not yet terminated by newline or prompt event)
    pub fn current_line(&self) -> &[u8] {
        &self.line_buf
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
}

