// Key normalization for terminal ESC sequences (subset), inspired by Toy 6.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCode {
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    F(u8),
    Alt(u8), // Alt + ASCII byte
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEvent {
    Byte(u8),
    Key(KeyCode),
}

enum EscState {
    None,
    Esc,
    Csi(Vec<u8>),
    EfO, // ESC O ...
}

pub struct KeyDecoder {
    state: EscState,
}

impl Default for KeyDecoder { fn default() -> Self { Self{ state: EscState::None } } }
impl KeyDecoder { pub fn new() -> Self { Self::default() } }

impl KeyDecoder {
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<KeyEvent> {
        let mut out = Vec::new();
        for &b in bytes {
            match &mut self.state {
                EscState::None => {
                    if b == 0x1B { self.state = EscState::Esc; }
                    else { out.push(KeyEvent::Byte(b)); }
                }
                EscState::Esc => {
                    if b == b'[' { self.state = EscState::Csi(Vec::new()); }
                    else if b == b'O' { self.state = EscState::EfO; }
                    // Alt-<letter>
                    else if (b as char).is_ascii_alphabetic() {
                        out.push(KeyEvent::Key(KeyCode::Alt(b.to_ascii_lowercase())));
                        self.state = EscState::None;
                    } else { out.push(KeyEvent::Key(KeyCode::Escape)); self.state = EscState::None; }
                }
                EscState::Csi(buf) => {
                    // Collect until a final byte in @A-Z~ range
                    if b.is_ascii_alphabetic() {
                        // Final letter
                        match b {
                            b'A' => out.push(KeyEvent::Key(KeyCode::ArrowUp)),
                            b'B' => out.push(KeyEvent::Key(KeyCode::ArrowDown)),
                            b'C' => out.push(KeyEvent::Key(KeyCode::ArrowRight)),
                            b'D' => out.push(KeyEvent::Key(KeyCode::ArrowLeft)),
                            b'H' => out.push(KeyEvent::Key(KeyCode::Home)),
                            b'F' => out.push(KeyEvent::Key(KeyCode::End)),
                            _ => { /* ignore unknown */ }
                        }
                        self.state = EscState::None;
                    } else if b == b'~' {
                        // Tilde-terminated sequences like [5~, [6~, [2~, [3~
                        let seq = std::str::from_utf8(&buf[..]).unwrap_or("");
                        match seq {
                            "2" => out.push(KeyEvent::Key(KeyCode::Insert)),
                            "3" => out.push(KeyEvent::Key(KeyCode::Delete)),
                            "5" => out.push(KeyEvent::Key(KeyCode::PageUp)),
                            "6" => out.push(KeyEvent::Key(KeyCode::PageDown)),
                            _ => {}
                        }
                        self.state = EscState::None;
                    } else {
                        buf.push(b);
                    }
                }
                EscState::EfO => {
                    // ESC O P..S : F1..F4 on some terms
                    match b {
                        b'P' => out.push(KeyEvent::Key(KeyCode::F(1))),
                        b'Q' => out.push(KeyEvent::Key(KeyCode::F(2))),
                        b'R' => out.push(KeyEvent::Key(KeyCode::F(3))),
                        b'S' => out.push(KeyEvent::Key(KeyCode::F(4))),
                        _ => {}
                    }
                    self.state = EscState::None;
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_keys_and_paging() {
        let mut d = KeyDecoder::new();
        let ev = d.feed(b"\x1b[A\x1b[B\x1b[C\x1b[D\x1b[5~\x1b[6~\x1b[2~\x1b[3~");
        assert!(matches!(ev[0], KeyEvent::Key(KeyCode::ArrowUp)));
        assert!(matches!(ev[1], KeyEvent::Key(KeyCode::ArrowDown)));
        assert!(matches!(ev[2], KeyEvent::Key(KeyCode::ArrowRight)));
        assert!(matches!(ev[3], KeyEvent::Key(KeyCode::ArrowLeft)));
        assert!(matches!(ev[4], KeyEvent::Key(KeyCode::PageUp)));
        assert!(matches!(ev[5], KeyEvent::Key(KeyCode::PageDown)));
        assert!(matches!(ev[6], KeyEvent::Key(KeyCode::Insert)));
        assert!(matches!(ev[7], KeyEvent::Key(KeyCode::Delete)));
    }

    #[test]
    fn alt_letter_and_fkeys() {
        let mut d = KeyDecoder::new();
        let ev = d.feed(b"\x1ba\x1bOP\x1bOQ\x1bOR\x1bOS");
        assert!(ev.iter().any(|e| matches!(e, KeyEvent::Key(KeyCode::Alt(b'a')))));
        assert!(ev.iter().any(|e| matches!(e, KeyEvent::Key(KeyCode::F(1)))));
        assert!(ev.iter().any(|e| matches!(e, KeyEvent::Key(KeyCode::F(2)))));
        assert!(ev.iter().any(|e| matches!(e, KeyEvent::Key(KeyCode::F(3)))));
        assert!(ev.iter().any(|e| matches!(e, KeyEvent::Key(KeyCode::F(4)))));
    }

    #[test]
    fn fragmentation_across_chunks() {
        let mut d = KeyDecoder::new();
        let mut out = Vec::new();
        out.extend(d.feed(b"\x1b"));
        out.extend(d.feed(b"[5"));
        out.extend(d.feed(b"~"));
        assert!(matches!(out[0], KeyEvent::Key(KeyCode::PageUp)));
    }
}
