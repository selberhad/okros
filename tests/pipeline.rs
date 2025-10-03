use mcl_rust::mccp::{Decompressor, PassthroughDecomp};
use mcl_rust::telnet::TelnetParser;
use mcl_rust::ansi::{AnsiConverter, AnsiEvent};
use mcl_rust::scrollback::Scrollback;

#[test]
fn pipeline_passthrough_telnet_ansi_scrollback() {
    // Pipeline: MCCP (passthrough) → Telnet (clean IAC) → ANSI (SGR to color) → Scrollback
    let mut decomp = PassthroughDecomp::new();
    let mut telnet = TelnetParser::new();
    let mut ansi = AnsiConverter::new();
    let mut sb = Scrollback::new(5, 2, 20);

    let chunks: Vec<&[u8]> = vec![
        b"He",
        b"\x1b",
        b"[31m",
        b"llo\n",
        b"World\n",
    ];

    let mut cur_color: u8 = 0x07; // white-on-black
    let mut line_bytes: Vec<u8> = Vec::new();

    for ch in chunks {
        // Decompress
        decomp.receive(ch);
        while decomp.pending() {
            let out = decomp.take_output();
            // Telnet parse
            telnet.feed(&out);
            // Any clean app bytes → ANSI converter
            let app = telnet.take_app_out();
            if !app.is_empty() {
                for ev in ansi.feed(&app) {
                    match ev {
                        AnsiEvent::SetColor(c) => cur_color = c,
                        AnsiEvent::Text(b'\n') => {
                            sb.print_line(&line_bytes, cur_color);
                            line_bytes.clear();
                        }
                        AnsiEvent::Text(b) => line_bytes.push(b),
                    }
                }
            }
        }
    }
    if !line_bytes.is_empty() { sb.print_line(&line_bytes, cur_color); }

    // Assert scrollback viewport shows the two lines, with the first "Hello" and second "World"
    let v = sb.viewport_slice();
    let text: Vec<u8> = v.iter().map(|a| (a & 0xFF) as u8).collect();
    assert_eq!(&text[0..5], b"Hello");
    assert_eq!(&text[5..10], b"World");
    // Note: our simple integration writes lines with the color active at flush time.
    // Detailed per-character color application is covered in unit tests.
}
