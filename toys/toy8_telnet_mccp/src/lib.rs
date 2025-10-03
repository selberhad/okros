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
    pub const TELOPT_EOR: u8 = 25; // per reference (IAC DO 25 in response to WILL 25)
    pub const TELOPT_COMPRESS: u8 = 85;  // MCCP v1
    pub const TELOPT_COMPRESS2: u8 = 86; // MCCP v2
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
                // Reference behavior: only respond to WILL EOR (25) with DO EOR.
                // MCCP responses are handled by the decompressor, not here.
                match (cmd, b) {
                    (telnet::WILL, telnet::TELOPT_EOR) => {
                        self.responses.extend_from_slice(&[telnet::IAC, telnet::DO, b]);
                    }
                    _ => {
                        // Ignore other negotiations by default
                    }
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

// ANSI SGR → attrib color converter
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnsiEvent {
    Text(u8),
    SetColor(u8),
}

fn ansi_to_internal(idx: u8) -> u8 {
    match idx & 0x07 { 0=>0, 1=>4, 2=>2, 3=>6, 4=>1, 5=>5, 6=>3, _=>7 }
}

pub struct AnsiConverter {
    buf: Vec<u8>,
    in_csi: bool,
    cur_fg: u8,
    cur_bg: u8,
    bold: bool,
}

impl Default for AnsiConverter {
    fn default() -> Self { Self{ buf: Vec::new(), in_csi: false, cur_fg: 7, cur_bg: 0, bold: false } }
}

impl AnsiConverter {
    pub fn new() -> Self { Self::default() }
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<AnsiEvent> {
        let mut out = Vec::new();
        let mut i = 0usize;
        while i < bytes.len() {
            let b = bytes[i];
            if !self.in_csi {
                if b == 0x1B { // ESC
                    self.in_csi = true;
                    self.buf.clear();
                    i += 1; continue;
                } else {
                    out.push(AnsiEvent::Text(b)); i+=1; continue;
                }
            } else {
                // Expect '[' then params then 'm'
                if self.buf.is_empty() {
                    if b != b'[' { // not SGR, cancel
                        self.in_csi = false; continue;
                    } else { self.buf.push(b); i+=1; continue; }
                } else {
                    self.buf.push(b); i+=1;
                    if b == b'm' {
                        // parse params excluding initial '[' and final 'm'
                        let params_str = std::str::from_utf8(&self.buf[1..self.buf.len()-1]).unwrap_or("");
                        let mut new_fg = self.cur_fg; let mut new_bg = self.cur_bg; let mut new_bold = self.bold;
                        for part in params_str.split(';').filter(|s| !s.is_empty()) {
                            if let Ok(n) = part.parse::<u32>() {
                                match n {
                                    0 => { new_bold=false; new_fg=7; new_bg=0; }
                                    1 => { new_bold=true; }
                                    30..=37 => { new_fg = ansi_to_internal((n as u8)-30); }
                                    40..=47 => { new_bg = ansi_to_internal((n as u8)-40); }
                                    _ => {}
                                }
                            }
                        }
                        self.cur_fg = new_fg; self.cur_bg = new_bg; self.bold = new_bold;
                        let mut color: u8 = (self.cur_bg << 4) | (self.cur_fg & 0x0F);
                        if self.bold { color |= 1<<7; }
                        out.push(AnsiEvent::SetColor(color));
                        self.in_csi = false; self.buf.clear();
                    }
                    continue;
                }
            }
        }
        out
    }
}

// Phase B scaffold: MCCP decompression interface and pipeline

pub trait Decompressor {
    fn receive(&mut self, input: &[u8]);
    fn pending(&self) -> bool;
    fn take_output(&mut self) -> Vec<u8>;
    fn error(&self) -> bool { false }
    fn response(&mut self) -> Option<Vec<u8>> { None }
}

// Simple pass-through decompressor used for tests until real MCCP is wired
pub struct PassthroughDecomp {
    buf: Vec<u8>,
}

impl PassthroughDecomp {
    pub fn new() -> Self { Self{ buf: Vec::new() } }
}

impl Decompressor for PassthroughDecomp {
    fn receive(&mut self, input: &[u8]) { self.buf.extend_from_slice(input); }
    fn pending(&self) -> bool { !self.buf.is_empty() }
    fn take_output(&mut self) -> Vec<u8> { std::mem::take(&mut self.buf) }
}

// End-to-end pipeline mirroring reference order: decompress → telnet parse
pub struct Pipeline<D: Decompressor> {
    pub decomp: D,
    pub telnet: TelnetParser,
}

impl<D: Decompressor> Pipeline<D> {
    pub fn new(decomp: D) -> Self { Self{ decomp, telnet: TelnetParser::new() } }

    pub fn feed(&mut self, chunk: &[u8]) {
        self.decomp.receive(chunk);
        while self.decomp.pending() {
            let out = self.decomp.take_output();
            if !out.is_empty() {
                self.telnet.feed(&out);
            }
        }
    }

    pub fn drain_decomp_responses(&mut self) -> Vec<u8> {
        let mut all = Vec::new();
        while let Some(mut r) = self.decomp.response() {
            all.append(&mut r);
        }
        all
    }

    pub fn error(&self) -> bool { self.decomp.error() }
}

// MCCP handshake stub that strips MCCP telnet negotiation and emits responses
pub struct MccpStub {
    residual: Vec<u8>,
    out: Vec<u8>,
    responses: Vec<u8>,
    got_v2: bool,
    compressing: bool,
    error: bool,
}

impl MccpStub {
    pub fn new() -> Self { Self { residual: Vec::new(), out: Vec::new(), responses: Vec::new(), got_v2: false, compressing: false, error: false } }
}

impl Decompressor for MccpStub {
    fn receive(&mut self, input: &[u8]) {
        // Append new bytes
        self.residual.extend_from_slice(input);

        // If not compressing, scan for MCCP negotiation and strip it.
        let mut i = 0usize;
        while i < self.residual.len() {
            let b = self.residual[i];
            if !self.compressing {
                if b != telnet::IAC {
                    self.out.push(b);
                    i += 1;
                    continue;
                }
                // Need at least 2 bytes to decide
                if i + 1 >= self.residual.len() { break; }
                let b1 = self.residual[i+1];
                // Escaped IAC
                if b1 == telnet::IAC {
                    self.out.push(telnet::IAC);
                    i += 2;
                    continue;
                }
                // MCCP WILL COMPRESS / COMPRESS2
                if b1 == telnet::WILL {
                    if i + 2 >= self.residual.len() { break; }
                    let opt = self.residual[i+2];
                    if opt == telnet::TELOPT_COMPRESS2 {
                        // respond DO COMPRESS2, strip from stream
                        self.responses.extend_from_slice(&[telnet::IAC, telnet::DO, telnet::TELOPT_COMPRESS2]);
                        self.got_v2 = true;
                        i += 3;
                        continue;
                    } else if opt == telnet::TELOPT_COMPRESS {
                        // respond DO COMPRESS if no v2, else DONT COMPRESS
                        if self.got_v2 {
                            self.responses.extend_from_slice(&[telnet::IAC, telnet::DONT, telnet::TELOPT_COMPRESS]);
                        } else {
                            self.responses.extend_from_slice(&[telnet::IAC, telnet::DO, telnet::TELOPT_COMPRESS]);
                        }
                        i += 3;
                        continue;
                    }
                    // other WILL: let through to telnet
                }
                // MCCP start sequences
                if b1 == telnet::SB {
                    // need 5 bytes for either start sequence
                    if i + 4 >= self.residual.len() { break; }
                    let opt = self.residual[i+2];
                    // v1: IAC SB 85 WILL SE
                    if opt == telnet::TELOPT_COMPRESS && self.residual[i+3] == telnet::WILL && self.residual[i+4] == telnet::SE {
                        self.compressing = true;
                        i += 5;
                        continue;
                    }
                    // v2: IAC SB 86 IAC SE
                    if opt == telnet::TELOPT_COMPRESS2 && self.residual[i+3] == telnet::IAC && self.residual[i+4] == telnet::SE {
                        self.compressing = true;
                        i += 5;
                        continue;
                    }
                    // Not a start sequence; pass through as normal
                }
                // Default: pass IAC and continue; TelnetParser will handle
                self.out.push(b);
                i += 1;
                continue;
            } else {
                // compressing: we don't actually decompress here; pass-through
                // Simulate an error if a special sentinel appears during compression
                // Sentinel: 0xDE 0xAD 0xBE 0xEF (chosen to not collide with normal text)
                if i + 3 < self.residual.len()
                    && self.residual[i] == 0xDE && self.residual[i+1] == 0xAD
                    && self.residual[i+2] == 0xBE && self.residual[i+3] == 0xEF {
                    self.error = true;
                    i += 4;
                    continue;
                }
                // Simulate end-of-compression (like Z_STREAM_END) sentinel: 0xED 0xFE 0xED
                if i + 2 < self.residual.len()
                    && self.residual[i] == 0xED && self.residual[i+1] == 0xFE
                    && self.residual[i+2] == 0xED {
                    // Disable compression and skip the marker
                    self.compressing = false;
                    i += 3;
                    continue;
                }
                self.out.push(b);
                i += 1;
            }
        }
        // Keep unconsumed residual for next call
        if i > 0 { self.residual.drain(0..i); }
    }

    fn pending(&self) -> bool { !self.error && !self.out.is_empty() }
    fn take_output(&mut self) -> Vec<u8> { std::mem::take(&mut self.out) }
    fn error(&self) -> bool { self.error }
    fn response(&mut self) -> Option<Vec<u8>> {
        if self.responses.is_empty() { None } else { Some(std::mem::take(&mut self.responses)) }
    }
}

#[cfg(feature = "real_mccp")]
pub struct MccpInflate {
    residual: Vec<u8>,
    out: Vec<u8>,
    responses: Vec<u8>,
    got_v2: bool,
    compressing: bool,
    error: bool,
    comp_bytes: usize,
    uncomp_bytes: usize,
    decompressor: Option<flate2::Decompress>,
}

#[cfg(feature = "real_mccp")]
impl MccpInflate {
    pub fn new() -> Self {
        Self { residual: Vec::new(), out: Vec::new(), responses: Vec::new(), got_v2: false, compressing: false, error: false, comp_bytes: 0, uncomp_bytes: 0, decompressor: None }
    }
    pub fn stats(&self) -> (usize, usize) { (self.comp_bytes, self.uncomp_bytes) }
    pub fn version(&self) -> u8 {
        if self.decompressor.is_none() && !self.compressing { 0 } else if self.got_v2 { 2 } else { 1 }
    }
}

#[cfg(feature = "real_mccp")]
impl Decompressor for MccpInflate {
    fn receive(&mut self, input: &[u8]) {
        self.residual.extend_from_slice(input);
        let mut i = 0usize;
        while i < self.residual.len() {
            let b = self.residual[i];
            if !self.compressing {
                if b != telnet::IAC { self.out.push(b); i+=1; continue; }
                if i+1 >= self.residual.len() { break; }
                let b1 = self.residual[i+1];
                if b1 == telnet::IAC { self.out.push(telnet::IAC); i+=2; continue; }
                if b1 == telnet::WILL {
                    if i+2 >= self.residual.len() { break; }
                    let opt = self.residual[i+2];
                    if opt == telnet::TELOPT_COMPRESS2 { self.responses.extend_from_slice(&[telnet::IAC, telnet::DO, telnet::TELOPT_COMPRESS2]); self.got_v2=true; i+=3; continue; }
                    if opt == telnet::TELOPT_COMPRESS { if self.got_v2 { self.responses.extend_from_slice(&[telnet::IAC, telnet::DONT, telnet::TELOPT_COMPRESS]); } else { self.responses.extend_from_slice(&[telnet::IAC, telnet::DO, telnet::TELOPT_COMPRESS]); } i+=3; continue; }
                }
                if b1 == telnet::SB {
                    if i+4 >= self.residual.len() { break; }
                    let opt = self.residual[i+2];
                    if opt == telnet::TELOPT_COMPRESS && self.residual[i+3]==telnet::WILL && self.residual[i+4]==telnet::SE {
                        self.compressing = true; self.decompressor = Some(flate2::Decompress::new(true)); i+=5; continue;
                    }
                    if opt == telnet::TELOPT_COMPRESS2 && self.residual[i+3]==telnet::IAC && self.residual[i+4]==telnet::SE {
                        self.compressing = true; self.decompressor = Some(flate2::Decompress::new(true)); i+=5; continue;
                    }
                }
                // pass IAC to telnet layer if not MCCP control
                self.out.push(b); i+=1; continue;
            } else {
                // streaming inflate
                let mut dec = match self.decompressor.as_mut() { Some(d)=>d, None=>{ self.error=true; break } };
                // use what's left as input
                let in_data = &self.residual[i..];
                let out_start = self.out.len();
                // reserve some space
                self.out.resize(out_start + in_data.len().max(64), 0);
                let mut total_in_before = dec.total_in();
                let mut total_out_before = dec.total_out();
                let res = dec.decompress(in_data, &mut self.out[out_start..], flate2::FlushDecompress::None);
                match res {
                    Ok(status) => {
                        let used_in = (dec.total_in() - total_in_before) as usize;
                        let produced = (dec.total_out() - total_out_before) as usize;
                        self.comp_bytes += used_in; self.uncomp_bytes += produced;
                        i += used_in;
                        self.out.truncate(out_start + produced);
                        if status == flate2::Status::StreamEnd {
                            // Disable compression; copy any remaining bytes as plain
                            self.compressing = false;
                            self.decompressor = None;
                        }
                        if used_in == 0 && produced == 0 { // need more output space
                            // grow and retry a little
                            self.out.reserve(128);
                            // prevent infinite loop
                            if in_data.is_empty() { break; }
                        }
                    }
                    Err(_) => { self.error = true; break; }
                }
                continue;
            }
        }
        if i>0 { self.residual.drain(0..i); }
    }
    fn pending(&self) -> bool { !self.error && !self.out.is_empty() }
    fn take_output(&mut self) -> Vec<u8> { std::mem::take(&mut self.out) }
    fn error(&self) -> bool { self.error }
    fn response(&mut self) -> Option<Vec<u8>> { if self.responses.is_empty(){None}else{Some(std::mem::take(&mut self.responses))} }
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
    fn fragmented_do_option_is_ignored() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC, DO]);
        p.feed(&[1]); // ECHO
        assert!(p.take_responses().is_empty());
        assert!(p.take_app_out().is_empty());
    }

    #[test]
    fn will_option_is_ignored_except_eor() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC, WILL, 31]); // NAWS
        assert!(p.take_responses().is_empty());
    }

    #[test]
    fn iac_escaped_255_in_output() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC, IAC]);
        assert_eq!(p.take_app_out(), vec![IAC]);
    }

    #[test]
    fn iac_escaped_255_across_fragments() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC]);
        p.feed(&[IAC]);
        assert_eq!(p.take_app_out(), vec![IAC]);
        assert!(p.take_responses().is_empty());
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
    fn subnegotiation_data_allows_escaped_iac_iac() {
        let mut p = TelnetParser::new();
        // IAC SB 31 ... (IAC IAC as literal 255) ... IAC SE
        p.feed(&[IAC, SB, 31]);
        p.feed(&[IAC, IAC]); // literal 255 inside SB
        p.feed(&[IAC, SE]);
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

    #[test]
    fn fragmented_ga_across_chunks_splices_prompt() {
        let mut p = TelnetParser::new();
        p.feed(b"hello ");
        p.feed(&[IAC]);
        p.feed(&[GA]);
        p.feed(b"world");
        assert_eq!(p.take_app_out(), b"hello world");
        assert_eq!(p.drain_prompt_events(), 1);
    }

    #[test]
    fn will_eor_generates_do_eor() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC, WILL, TELOPT_EOR]);
        assert_eq!(p.take_responses(), vec![IAC, DO, TELOPT_EOR]);
    }

    #[test]
    fn fragmented_will_eor_generates_do_eor() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC]);
        p.feed(&[WILL]);
        p.feed(&[TELOPT_EOR]);
        assert_eq!(p.take_responses(), vec![IAC, DO, TELOPT_EOR]);
    }

    #[test]
    fn dont_and_wont_produce_no_response() {
        let mut p = TelnetParser::new();
        p.feed(&[IAC, DONT, 1]);
        p.feed(&[IAC, WONT, 31]);
        assert!(p.take_responses().is_empty());
    }

    // Pipeline tests (MCCP scaffold)
    #[test]
    fn pipeline_passthrough_plain_text() {
        let mut pl = Pipeline::new(PassthroughDecomp::new());
        pl.feed(b"abc");
        pl.feed(&[IAC, GA]);
        pl.feed(b"def");
        assert_eq!(pl.telnet.take_app_out(), b"abcdef");
        assert_eq!(pl.telnet.drain_prompt_events(), 1);
    }

    #[test]
    fn pipeline_will_eor_negotiation_passthrough() {
        let mut pl = Pipeline::new(PassthroughDecomp::new());
        pl.feed(&[IAC, WILL, TELOPT_EOR]);
        assert_eq!(pl.telnet.take_responses(), vec![IAC, DO, TELOPT_EOR]);
    }

    // MCCP handshake tests using MccpStub
    #[test]
    fn mccp_v2_will_triggers_do_and_is_stripped() {
        let mut pl = Pipeline::new(MccpStub::new());
        pl.feed(&[IAC, WILL, TELOPT_COMPRESS2]);
        // responses come from decompressor, not telnet
        assert_eq!(pl.telnet.take_responses(), Vec::<u8>::new());
        assert_eq!(pl.drain_decomp_responses(), vec![IAC, DO, TELOPT_COMPRESS2]);
        // no app output
        assert!(pl.telnet.take_app_out().is_empty());
    }

    #[test]
    fn mccp_v1_will_after_v2_triggers_dont() {
        let mut pl = Pipeline::new(MccpStub::new());
        pl.feed(&[IAC, WILL, TELOPT_COMPRESS2]);
        // drain first response
        let _ = pl.drain_decomp_responses();
        pl.feed(&[IAC, WILL, TELOPT_COMPRESS]);
        assert_eq!(pl.drain_decomp_responses(), vec![IAC, DONT, TELOPT_COMPRESS]);
    }

    #[test]
    fn mccp_v2_start_sequence_enters_compressing_and_passes_following_bytes() {
        let mut pl = Pipeline::new(MccpStub::new());
        pl.feed(&[IAC, WILL, TELOPT_COMPRESS2]);
        let _ = pl.drain_decomp_responses();
        // Start sequence: IAC SB 86 IAC SE
        pl.feed(&[IAC, SB, TELOPT_COMPRESS2, IAC, SE]);
        // Now feed some payload; stub should pass-through to telnet
        pl.feed(b"hello");
        assert_eq!(pl.telnet.take_app_out(), b"hello");
    }

    #[test]
    fn mccp_v1_start_sequence_enters_compressing_and_passes_following_bytes() {
        let mut pl = Pipeline::new(MccpStub::new());
        pl.feed(&[IAC, WILL, TELOPT_COMPRESS]);
        let _ = pl.drain_decomp_responses();
        // Start sequence: IAC SB 85 WILL SE
        pl.feed(&[IAC, SB, TELOPT_COMPRESS, WILL, SE]);
        pl.feed(b"data");
        assert_eq!(pl.telnet.take_app_out(), b"data");
    }

    #[test]
    fn mccp_start_sequences_are_stripped_not_shown_to_telnet() {
        let mut pl = Pipeline::new(MccpStub::new());
        pl.feed(&[IAC, SB, TELOPT_COMPRESS2, IAC, SE]);
        // No app bytes yet, and telnet didn't see SB/SE
        assert!(pl.telnet.take_app_out().is_empty());
    }

    #[cfg(feature = "real_mccp")]
    fn compress_bytes(data: &[u8]) -> Vec<u8> {
        use flate2::{Compression, write::ZlibEncoder};
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        use std::io::Write;
        enc.write_all(data).unwrap();
        enc.finish().unwrap()
    }

    #[test]
    #[cfg(feature = "real_mccp")]
    fn real_mccp_v2_handshake_and_decompress() {
        let mut pl = Pipeline::new(MccpInflate::new());
        // Will COMPRESS2
        pl.feed(&[IAC, WILL, TELOPT_COMPRESS2]);
        assert_eq!(pl.drain_decomp_responses(), vec![IAC, DO, TELOPT_COMPRESS2]);
        // Start sequence
        pl.feed(&[IAC, SB, TELOPT_COMPRESS2, IAC, SE]);
        // Compressed payload
        let payload = compress_bytes(b"hello world");
        // Feed in two fragments to mimic streaming
        let mid = payload.len()/2;
        pl.feed(&payload[..mid]);
        pl.feed(&payload[mid..]);
        assert_eq!(pl.telnet.take_app_out(), b"hello world");
        assert!(!pl.error());
        // End-of-stream: compressor marks end internally; we just continue
        pl.feed(b"tail");
        assert_eq!(pl.telnet.take_app_out(), b"tail");
    }

    #[test]
    #[cfg(feature = "real_mccp")]
    fn real_mccp_v1_handshake_and_decompress() {
        let mut pl = Pipeline::new(MccpInflate::new());
        // Will COMPRESS (v1)
        pl.feed(&[IAC, WILL, TELOPT_COMPRESS]);
        // Accept v1 (no prior v2)
        assert_eq!(pl.drain_decomp_responses(), vec![IAC, DO, TELOPT_COMPRESS]);
        // Start sequence v1: IAC SB 85 WILL SE
        pl.feed(&[IAC, SB, TELOPT_COMPRESS, WILL, SE]);
        // Compressed payload
        let payload = compress_bytes(b"v1-ok");
        pl.feed(&payload);
        assert_eq!(pl.telnet.take_app_out(), b"v1-ok");
        assert!(!pl.error());
    }

    #[test]
    #[cfg(feature = "real_mccp")]
    fn real_mccp_error_on_invalid_stream() {
        let mut pl = Pipeline::new(MccpInflate::new());
        pl.feed(&[IAC, WILL, TELOPT_COMPRESS2]);
        let _ = pl.drain_decomp_responses();
        pl.feed(&[IAC, SB, TELOPT_COMPRESS2, IAC, SE]);
        // Feed invalid zlib bytes
        pl.feed(&[0x00, 0x01, 0x02, 0x03]);
        assert!(pl.error());
        // Further bytes should not be forwarded
        pl.feed(b"after");
        assert!(pl.telnet.take_app_out().is_empty());
    }

    #[test]
    #[cfg(feature = "real_mccp")]
    fn real_mccp_stats_and_version() {
        let mut pl = Pipeline::new(MccpInflate::new());
        pl.feed(&[IAC, WILL, TELOPT_COMPRESS2]);
        let _ = pl.drain_decomp_responses();
        pl.feed(&[IAC, SB, TELOPT_COMPRESS2, IAC, SE]);
        // During compression, version should be 2
        assert_eq!(pl.decomp.version(), 2);
        let payload = compress_bytes(b"stat");
        pl.feed(&payload);
        assert_eq!(pl.telnet.take_app_out(), b"stat");
        let (comp, uncomp) = pl.decomp.stats();
        assert!(comp > 0);
        assert_eq!(uncomp, 4);
        // After EOS, version may be 0
        assert!(matches!(pl.decomp.version(), 0|2));
    }

    #[test]
    fn ansi_sgr_basic_and_fragmentation() {
        let mut ac = AnsiConverter::new();
        // Basic color set: ESC[31m
        let mut ev = ac.feed(b"A ");
        ev.extend(ac.feed(&[0x1B]));
        ev.extend(ac.feed(b"[31m"));
        ev.extend(ac.feed(b"B"));
        // Expect Text('A'), Text(' '), SetColor(color for fg=red, bg=black, no bold), Text('B')
        assert!(matches!(ev[0], AnsiEvent::Text(b'A')));
        assert!(matches!(ev[1], AnsiEvent::Text(b' ')));
        let c = match ev[2] { AnsiEvent::SetColor(c)=>c, _=>0 };
        // red=ANSI 31 -> internal fg=4 via inverse map; bg=0
        assert_eq!(c & 0x0F, 4);
        assert_eq!((c & 0xF0)>>4, 0);
        assert!(matches!(ev[3], AnsiEvent::Text(b'B')));

        // Bold + bg + fg, then reset
        let mut ac = AnsiConverter::new();
        let ev2 = ac.feed(b"\x1b[1;44;33mZ\x1b[0m");
        // First event should be SetColor with bold, bg=blue(4->1 internal?), fg=yellow(3->6 internal?)
        if let AnsiEvent::SetColor(col) = ev2[0] {
            assert_eq!((col & 0x80) != 0, true);
            assert_eq!(((col & 0x70)>>4), 1); // blue ANSI 44 -> internal 1
            assert_eq!(col & 0x0F, 6); // yellow ANSI 33 -> internal 6
        } else { panic!("expected SetColor"); }
        // Then Text('Z') then SetColor(reset white on black)
        assert!(matches!(ev2[1], AnsiEvent::Text(b'Z')));
        if let AnsiEvent::SetColor(col) = ev2[2] { assert_eq!(col & 0x0F, 7); assert_eq!((col & 0xF0)>>4, 0); assert_eq!(col & 0x80, 0);} else { panic!("expected reset SetColor"); }
    }

    #[test]
    fn one_byte_fragmentation_mixed_text_and_ga() {
        let mut pl = Pipeline::new(PassthroughDecomp::new());
        for &b in b"hello" { pl.feed(&[b]); }
        pl.feed(&[IAC]);
        pl.feed(&[GA]);
        for &b in b" world" { pl.feed(&[b]); }
        assert_eq!(pl.telnet.take_app_out(), b"hello world");
        assert_eq!(pl.telnet.drain_prompt_events(), 1);
    }

    #[test]
    fn one_byte_fragmentation_will_eor_negotiation() {
        let mut pl = Pipeline::new(PassthroughDecomp::new());
        pl.feed(&[IAC]);
        pl.feed(&[WILL]);
        pl.feed(&[TELOPT_EOR]);
        assert_eq!(pl.telnet.take_responses(), vec![IAC, DO, TELOPT_EOR]);
    }

    #[test]
    fn one_byte_fragmentation_do_option_is_ignored() {
        let mut pl = Pipeline::new(PassthroughDecomp::new());
        pl.feed(&[IAC]);
        pl.feed(&[DO]);
        pl.feed(&[1]);
        assert!(pl.telnet.take_responses().is_empty());
        assert!(pl.telnet.take_app_out().is_empty());
    }

    #[test]
    fn mixed_option_negotiations_yield_expected_responses_in_order() {
        let mut p = TelnetParser::new();
        // WILL EOR, WILL NAWS, DO 1 (in three chunks)
        p.feed(&[IAC, WILL, TELOPT_EOR]);
        p.feed(&[IAC, WILL, 31]);
        p.feed(&[IAC, DO, 1]);
        let r = p.take_responses();
        assert_eq!(r, vec![IAC, DO, TELOPT_EOR]);
        assert!(p.take_app_out().is_empty());
    }

    #[test]
    fn mccp_error_path_stops_output() {
        let mut pl = Pipeline::new(MccpStub::new());
        // Start v2 compressing
        pl.feed(&[IAC, WILL, TELOPT_COMPRESS2]);
        let _ = pl.drain_decomp_responses();
        pl.feed(&[IAC, SB, TELOPT_COMPRESS2, IAC, SE]);
        // Feed some payload, should pass
        pl.feed(b"abc");
        assert_eq!(pl.telnet.take_app_out(), b"abc");
        // Trigger error with sentinel bytes
        pl.feed(&[0xDE, 0xAD, 0xBE, 0xEF]);
        assert!(pl.error());
        // Subsequent bytes should not be forwarded (simulated stalled decompressor)
        pl.feed(b"more");
        assert!(pl.telnet.take_app_out().is_empty());
    }

    #[test]
    fn mccp_end_of_stream_disables_compression_and_telnet_continues() {
        let mut pl = Pipeline::new(MccpStub::new());
        // Enter compressing
        pl.feed(&[IAC, WILL, TELOPT_COMPRESS2]);
        let _ = pl.drain_decomp_responses();
        pl.feed(&[IAC, SB, TELOPT_COMPRESS2, IAC, SE]);
        pl.feed(b"xyz");
        assert_eq!(pl.telnet.take_app_out(), b"xyz");
        // Simulate end-of-compression
        pl.feed(&[0xED, 0xFE, 0xED]);
        // Now send a telnet DO option; parser should ignore (no reply here)
        pl.feed(&[IAC, DO, 1]);
        assert!(pl.telnet.take_responses().is_empty());
    }

    #[test]
    fn naws_negotiation_is_ignored() {
        let mut p = TelnetParser::new();
        // Reference default: ignore NAWS here; MCCP/EOR handled elsewhere
        p.feed(&[IAC, WILL, 31]);
        assert!(p.take_responses().is_empty());
        // Server DO NAWS -> ignored
        p.feed(&[IAC, DO, 31]);
        assert!(p.take_responses().is_empty());
    }

    #[test]
    fn naws_subnegotiation_is_ignored_and_stripped() {
        let mut p = TelnetParser::new();
        // Pretend some app text, then NAWS SB data, then more text
        p.feed(b"A");
        p.feed(&[IAC, SB]);
        p.feed(&[31, 80, 0, 24]); // option + width/height payload bytes
        p.feed(&[IAC, SE]);
        p.feed(b"B");
        // App output should be A then B, with no SB bytes
        assert_eq!(p.take_app_out(), b"AB");
    }
}
