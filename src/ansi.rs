#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnsiEvent {
    Text(u8),
    SetColor(u8),
}

fn inverse_color(idx: u8) -> u8 {
    match idx & 0x07 {
        0 => 0,
        1 => 4,
        2 => 2,
        3 => 6,
        4 => 1,
        5 => 5,
        6 => 3,
        _ => 7,
    }
}

#[derive(Default)]
pub struct AnsiConverter {
    buf: Vec<u8>,
    in_csi: bool,
    cur_fg: u8,
    cur_bg: u8,
    bold: bool,
}

impl AnsiConverter {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            in_csi: false,
            cur_fg: 7,
            cur_bg: 0,
            bold: false,
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) -> Vec<AnsiEvent> {
        let mut out = Vec::new();
        let mut i = 0usize;
        while i < bytes.len() {
            let b = bytes[i];
            if !self.in_csi {
                if b == 0x1B {
                    self.in_csi = true;
                    self.buf.clear();
                    i += 1;
                    continue;
                }
                out.push(AnsiEvent::Text(b));
                i += 1;
                continue;
            } else {
                if self.buf.is_empty() {
                    if b != b'[' {
                        self.in_csi = false;
                        continue;
                    } else {
                        self.buf.push(b);
                        i += 1;
                        continue;
                    }
                } else {
                    self.buf.push(b);
                    i += 1;
                    // CSI sequences end with any alphabetic character (A-Z, a-z)
                    if b.is_ascii_alphabetic() {
                        // Only process 'm' (color codes), ignore others (cursor positioning, etc)
                        if b == b'm' {
                            let params_str =
                                std::str::from_utf8(&self.buf[1..self.buf.len() - 1]).unwrap_or("");
                            let mut new_fg = self.cur_fg;
                            let mut new_bg = self.cur_bg;
                            let mut new_bold = self.bold;
                            for part in params_str.split(';').filter(|s| !s.is_empty()) {
                                if let Ok(n) = part.parse::<u32>() {
                                    match n {
                                        0 => {
                                            new_bold = false;
                                            new_fg = 7;
                                            new_bg = 0;
                                        }
                                        1 => {
                                            new_bold = true;
                                        }
                                        30..=37 => {
                                            new_fg = inverse_color((n as u8) - 30);
                                        }
                                        90..=97 => {
                                            new_fg = inverse_color((n as u8) - 90);
                                            new_bold = true;
                                        }
                                        40..=47 => {
                                            new_bg = inverse_color((n as u8) - 40);
                                        }
                                        100..=107 => {
                                            new_bg = inverse_color((n as u8) - 100);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            self.cur_fg = new_fg;
                            self.cur_bg = new_bg;
                            self.bold = new_bold;
                            let mut color: u8 = (self.cur_bg << 4) | (self.cur_fg & 0x0F);
                            if self.bold {
                                color |= 1 << 7;
                            }
                            out.push(AnsiEvent::SetColor(color));
                        }
                        // Exit CSI mode for any alphabetic character (H, J, K, m, etc)
                        self.in_csi = false;
                        self.buf.clear();
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telnet::TelnetParser;

    #[test]
    fn basic_and_fragmented_color() {
        let mut ac = AnsiConverter::new();
        let mut ev = ac.feed(b"A ");
        ev.extend(ac.feed(&[0x1B]));
        ev.extend(ac.feed(b"[31m"));
        ev.extend(ac.feed(b"B"));
        assert!(matches!(ev[0], AnsiEvent::Text(b'A')));
        assert!(matches!(ev[1], AnsiEvent::Text(b' ')));
        let c = match ev[2] {
            AnsiEvent::SetColor(c) => c,
            _ => 0,
        };
        assert_eq!(c & 0x0F, 4);
        assert_eq!((c & 0x70) >> 4, 0);
        assert!(matches!(ev[3], AnsiEvent::Text(b'B')));
    }

    #[test]
    fn multiple_sequences_reset_and_bright() {
        let mut ac = AnsiConverter::new();
        let ev = ac.feed(b"\x1b[1;44;33mZ\x1b[0m");
        if let AnsiEvent::SetColor(col) = ev[0] {
            assert_ne!(col & 0x80, 0);
            assert_eq!(((col & 0x70) >> 4), 1);
            assert_eq!(col & 0x0F, 6);
        } else {
            panic!()
        }
        assert!(matches!(ev[1], AnsiEvent::Text(b'Z')));
        if let AnsiEvent::SetColor(col) = ev[2] {
            assert_eq!(col & 0x0F, 7);
            assert_eq!(((col & 0x70) >> 4), 0);
            assert_eq!(col & 0x80, 0);
        } else {
            panic!()
        }
        // bright fg sets bold
        let ev2 = ac.feed(b"\x1b[91m");
        if let AnsiEvent::SetColor(c) = ev2[0] {
            assert_ne!(c & 0x80, 0);
            assert_eq!(c & 0x0F, 4);
        }
    }

    #[test]
    fn telnet_then_ansi_pipeline() {
        let mut t = TelnetParser::new();
        t.feed(b"A");
        t.feed(&[0x1B]);
        t.feed(b"[32mB");
        let app = t.take_app_out();
        let mut ac = AnsiConverter::new();
        let ev = ac.feed(&app);
        assert!(matches!(ev[0], AnsiEvent::Text(b'A')));
        if let AnsiEvent::SetColor(col) = ev[1] {
            assert_eq!(col & 0x0F, 2);
        } else {
            panic!()
        }
        assert!(matches!(ev[2], AnsiEvent::Text(b'B')));
    }
}
