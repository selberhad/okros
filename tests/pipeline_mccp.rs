#![cfg(feature = "mccp")]

use mcl_rust::mccp::{Decompressor, MccpInflate};
use mcl_rust::mccp::telopt::*;
use mcl_rust::telnet::TelnetParser;
use mcl_rust::ansi::{AnsiConverter, AnsiEvent};
use mcl_rust::scrollback::Scrollback;
use flate2::{Compression, write::ZlibEncoder};
use std::io::Write;

fn compress_bytes(data: &[u8]) -> Vec<u8> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    enc.write_all(data).unwrap();
    enc.finish().unwrap()
}

#[test]
fn pipeline_real_mccp_v2_telnet_ansi_scrollback() {
    let mut decomp = MccpInflate::new();
    let mut telnet = TelnetParser::new();
    let mut ansi = AnsiConverter::new();
    let mut sb = Scrollback::new(5, 2, 20);

    // Handshake: WILL COMPRESS2 → DO; start sequence v2
    decomp.receive(&[IAC, WILL, COMPRESS2]);
    assert_eq!(decomp.response().unwrap(), vec![IAC, DO, COMPRESS2]);
    decomp.receive(&[IAC, SB, COMPRESS2, IAC, SE]);

    let payload = compress_bytes(b"Hello\nWorld\n");
    // Feed in two fragments to simulate streaming
    let mid = payload.len()/2; decomp.receive(&payload[..mid]); decomp.receive(&payload[mid..]);

    let mut cur_color: u8 = 0x07;
    let mut line_bytes: Vec<u8> = Vec::new();
    while decomp.pending() {
        let out = decomp.take_output();
        telnet.feed(&out);
        let app = telnet.take_app_out();
        for ev in ansi.feed(&app) {
            match ev {
                AnsiEvent::SetColor(c) => cur_color = c,
                AnsiEvent::Text(b'\n') => { sb.print_line(&line_bytes, cur_color); line_bytes.clear(); }
                AnsiEvent::Text(b) => line_bytes.push(b),
            }
        }
    }

    let v = sb.viewport_slice();
    let text: Vec<u8> = v.iter().map(|a| (a & 0xFF) as u8).collect();
    assert_eq!(&text[0..5], b"Hello");
    assert_eq!(&text[5..10], b"World");
}

