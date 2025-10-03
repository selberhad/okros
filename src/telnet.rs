pub mod telnet {
    pub const IAC: u8 = 255;
    pub const DONT: u8 = 254;
    pub const DO: u8 = 253;
    pub const WONT: u8 = 252;
    pub const WILL: u8 = 251;
    pub const SB: u8 = 250;
    pub const GA: u8 = 249;
    pub const SE: u8 = 240;
    pub const EOR: u8 = 239;
    pub const TELOPT_EOR: u8 = 25;
}

pub struct TelnetParser {
    iac_seen: bool,
    cmd_pending: Option<u8>,
    sb_active: bool,
    app_out: Vec<u8>,
    responses: Vec<u8>,
    prompt_count: usize,
}

impl TelnetParser {
    pub fn new() -> Self { Self{ iac_seen:false, cmd_pending:None, sb_active:false, app_out:Vec::new(), responses:Vec::new(), prompt_count:0 } }
    pub fn feed(&mut self, chunk: &[u8]) {
        use telnet::*;
        let mut i=0; while i<chunk.len() { let b=chunk[i]; i+=1;
            if self.sb_active {
                if !self.iac_seen { if b==IAC { self.iac_seen=true; } } else { if b==SE { self.sb_active=false; self.iac_seen=false; } else if b==IAC { self.iac_seen=false; } else { self.iac_seen=false; } }
                continue;
            }
            if self.iac_seen {
                self.iac_seen=false;
                match b { IAC=>self.app_out.push(IAC), GA|EOR=>{ self.prompt_count+=1; }, SB=>{ self.sb_active=true; }, DO|DONT|WILL|WONT=>{ self.cmd_pending=Some(b); }, _=>{} }
                continue;
            }
            if let Some(cmd)=self.cmd_pending.take() { // process option byte b
                if cmd==WILL && b==TELOPT_EOR { self.responses.extend_from_slice(&[IAC, DO, b]); }
                continue;
            }
            if b==IAC { self.iac_seen=true; continue; }
            self.app_out.push(b);
        }}
    pub fn take_app_out(&mut self)->Vec<u8>{ std::mem::take(&mut self.app_out) }
    pub fn take_responses(&mut self)->Vec<u8>{ std::mem::take(&mut self.responses) }
    pub fn drain_prompt_events(&mut self)->usize{ let n=self.prompt_count; self.prompt_count=0; n }
}

#[cfg(test)]
mod tests { use super::*; use telnet::*;
    #[test] fn plain_text_passthrough(){ let mut p=TelnetParser::new(); p.feed(b"hello"); assert_eq!(p.take_app_out(), b"hello"); assert!(p.take_responses().is_empty()); }
    #[test] fn eor_reply_only(){ let mut p=TelnetParser::new(); p.feed(&[IAC,WILL,TELOPT_EOR]); assert_eq!(p.take_responses(), vec![IAC,DO,TELOPT_EOR]); }
    #[test] fn fragmented_will_eor(){ let mut p=TelnetParser::new(); p.feed(&[IAC]); p.feed(&[WILL]); p.feed(&[TELOPT_EOR]); assert_eq!(p.take_responses(), vec![IAC,DO,TELOPT_EOR]); }
    #[test] fn do_and_wont_ignored(){ let mut p=TelnetParser::new(); p.feed(&[IAC,DO,1]); p.feed(&[IAC,WONT,31]); assert!(p.take_responses().is_empty()); }
    #[test] fn iac_escaped_255_in_output(){ let mut p=TelnetParser::new(); p.feed(&[IAC,IAC]); assert_eq!(p.take_app_out(), vec![IAC]); }
    #[test] fn ga_and_eor_prompt_events(){ let mut p=TelnetParser::new(); p.feed(b"abc"); p.feed(&[IAC,GA]); p.feed(b"def"); assert_eq!(p.take_app_out(), b"abcdef"); assert_eq!(p.drain_prompt_events(),1); p.feed(&[IAC,EOR]); assert_eq!(p.drain_prompt_events(),1); }
    #[test] fn fragmented_ga_splices_prompt(){ let mut p=TelnetParser::new(); p.feed(b"hello "); p.feed(&[IAC]); p.feed(&[GA]); p.feed(b"world"); assert_eq!(p.take_app_out(), b"hello world"); assert_eq!(p.drain_prompt_events(),1); }
    #[test] fn sb_ignored(){ let mut p=TelnetParser::new(); p.feed(&[IAC,SB,1, IAC,SE]); assert!(p.take_app_out().is_empty()); }
    #[test] fn sb_allows_iac_iac_literal(){ let mut p=TelnetParser::new(); p.feed(&[IAC,SB,31]); p.feed(&[IAC,IAC]); p.feed(&[IAC,SE]); assert!(p.take_app_out().is_empty()); }
}
