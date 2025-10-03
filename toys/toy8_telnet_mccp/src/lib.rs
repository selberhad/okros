// Toy 8: Telnet parser (Phase A: no MCCP)

pub mod telnet {
    pub const IAC: u8 = 255;
    pub const DONT: u8 = 254;
    pub const DO: u8 = 253;
    pub const WONT: u8 = 252;
    pub const WILL: u8 = 251;
    pub const SB: u8 = 250;
    pub const GA: u8 = 249;
    pub const SE: u8 = 240;
    pub const EOR: u8 = 239; // End Of Record
}

pub struct TelnetParser {
    // IAC parsing state
    iac_seen: bool,
    cmd_pending: Option<u8>,
    opt_pending: Option<u8>,
    sb_active: bool,
    sb_buf: Vec<u8>,

    // outputs
    app_out: Vec<u8>,
    responses: Vec<u8>,
    prompt_count: usize,
}

impl TelnetParser {
    pub fn new() -> Self {
        Self {
            iac_seen: false,
            cmd_pending: None,
            opt_pending: None,
            sb_active: false,
            sb_buf: Vec::new(),
            app_out: Vec::new(),
            responses: Vec::new(),
            prompt_count: 0,
        }
    }

    pub fn feed(&mut self, chunk: &[u8]) {
        use telnet::*;
        let mut i = 0;
        while i < chunk.len() {
            let b = chunk[i];
            i += 1;

            if self.sb_active {
                // Subnegotiation until IAC SE
                if !self.iac_seen {
                    if b == IAC { self.iac_seen = true; } else { self.sb_buf.push(b); }
                } else {
                    // last seen IAC inside SB
                    if b == SE { // end subneg
                        self.sb_active = false;
                        self.iac_seen = false;
                        self.sb_buf.clear(); // ignore content
                    } else if b == IAC {
                        // IAC IAC within SB => literal 255 in subneg data
                        self.sb_buf.push(IAC);
                        self.iac_seen = false;
                    } else {
                        // Unexpected, reset
                        self.iac_seen = false;
                    }
                }
                continue;
            }

            if self.iac_seen {
                // We have IAC previously
                self.iac_seen = false;
                match b {
                    telnet::IAC => {
                        // Escaped 255 -> literal 255 in app output
                        self.app_out.push(telnet::IAC);
                    }
                    telnet::GA | telnet::EOR => {
                        // Prompt boundary
                        self.prompt_count += 1;
                    }
                    telnet::SB => {
                        self.sb_active = true;
                        self.sb_buf.clear();
                    }
                    telnet::DO | telnet::DONT | telnet::WILL | telnet::WONT => {
                        self.cmd_pending = Some(b);
                    }
                    _ => {
                        // Ignore other commands
                    }
                }
                continue;
            }

            if let Some(cmd) = self.cmd_pending {
                // Expecting an option byte
                self.opt_pending = Some(b);
                // Default policy: refuse all options (WONT for DO, DONT for WILL)
                match cmd {
                    telnet::DO => {
                        self.responses.extend_from_slice(&[telnet::IAC, telnet::WONT, b]);
                    }
                    telnet::WILL => {
                        self.responses.extend_from_slice(&[telnet::IAC, telnet::DONT, b]);
                    }
                    telnet::DONT | telnet::WONT => {
                        // Acknowledge passively: no response needed for simplicity
                    }
                    _ => {}
                }
                self.cmd_pending = None;
                self.opt_pending = None;
                continue;
            }

            if b == telnet::IAC {
                self.iac_seen = true;
                continue;
            }

            // Normal application byte
            self.app_out.push(b);
        }
    }

    pub fn take_app_out(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.app_out)
    }
    pub fn take_responses(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.responses)
    }
    pub fn drain_prompt_events(&mut self) -> usize {
        let n = self.prompt_count; self.prompt_count = 0; n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use telnet::*;

    #[test]
    fn plain_text_passthrough() {
        let mut p = TelnetParser::new();
        p.feed(b"hello");
        assert_eq!(p.take_app_out(), b"hello");
        assert!(p.take_responses().is_empty());
    }

    #[test]
    fn fragmented_do_option_generates_wont() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC, DO]);
        p.feed(&[1]); // ECHO
        assert_eq!(p.take_responses(), vec![IAC, WONT, 1]);
        assert!(p.take_app_out().is_empty());
    }

    #[test]
    fn will_option_generates_dont() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC, WILL, 31]); // NAWS
        assert_eq!(p.take_responses(), vec![IAC, DONT, 31]);
    }

    #[test]
    fn iac_escaped_255_in_output() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC, IAC]);
        assert_eq!(p.take_app_out(), vec![IAC]);
    }

    #[test]
    fn subnegotiation_ignored_until_se_across_fragments() {
        let mut p = TelnetParser::new();
        // IAC SB 1 ... IAC SE fragmented
        p.feed(&[IAC, SB]);
        p.feed(&[1, 0x41, 0x42]);
        p.feed(&[IAC]);
        p.feed(&[SE]);
        // No app output or responses from SB in phase A
        assert!(p.take_app_out().is_empty());
        assert!(p.take_responses().is_empty());
    }

    #[test]
    fn ga_and_eor_trigger_prompt_events() {
        let mut p = TelnetParser::new();
        p.feed(b"abc");
        p.feed(&[IAC, GA]);
        p.feed(b"def");
        assert_eq!(p.take_app_out(), b"abcdef");
        assert_eq!(p.drain_prompt_events(), 1);
        p.feed(&[IAC, EOR]);
        assert_eq!(p.drain_prompt_events(), 1);
    }
}

