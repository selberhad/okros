use std::io::{self, Read, Write, BufRead};
use std::time::{Duration, Instant};
use mcl_rust::input::{KeyDecoder, KeyEvent, KeyCode};
use mcl_rust::screen::{self, DiffOptions};
use mcl_rust::curses::get_acs_caps;
use mcl_rust::session::Session;
use mcl_rust::mccp::PassthroughDecomp;
use mcl_rust::engine::SessionEngine;
use mcl_rust::control::{ControlServer, default_socket_path};
use mcl_rust::select::{poll_fds, READ, WRITE};
use mcl_rust::socket::{Socket, ConnState};
use libc::{fcntl, F_SETFL, O_NONBLOCK};
use std::net::Ipv4Addr;

fn main() {
    // CLI: --headless --instance NAME | --attach NAME
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 && args[1] == "--headless" {
        let inst = args.get(3).cloned().unwrap_or_else(|| "default".to_string());
        let path = default_socket_path(&inst);
        let eng = SessionEngine::new(PassthroughDecomp::new(), 80, 20, 2000);
        let srv = ControlServer::new(path.clone(), eng);
        eprintln!("Headless engine; control socket at {}", path.display());
        let _ = srv.run();
        return;
    } else if args.len() > 2 && args[1] == "--attach" {
        let inst = args.get(2).cloned().unwrap_or_else(|| "default".to_string());
        let path = default_socket_path(&inst);
        match std::os::unix::net::UnixStream::connect(&path) {
            Ok(mut s) => {
                let _ = s.set_read_timeout(Some(std::time::Duration::from_millis(500)));
                let _ = writeln!(s, "{}", serde_json::json!({"cmd":"get_buffer"}).to_string());
                let mut buf = String::new();
                let mut br = std::io::BufReader::new(s);
                let _ = br.read_line(&mut buf);
                println!("{}", buf.trim_end());
            }
            Err(e) => { eprintln!("attach failed: {}", e); }
        }
        return;
    }
    println!("MCL Rust Port scaffold initialized.");

    #[cfg(feature = "python")]
    println!("Feature enabled: python (pyo3)");

    #[cfg(feature = "perl")]
    println!("Feature enabled: perl (FFI)");

    // Minimal demo: set raw mode, optional keypad app mode, then run a tiny event loop
    let mut tty = match mcl_rust::tty::Tty::new() { Ok(t) => t, Err(e) => { eprintln!("tty init failed: {}", e); return; } };
    let _ = tty.enable_raw();
    let _ = tty.keypad_application_mode(true);

    let mut out = io::stdout();

    // Compose a small UI: status (top), output (middle), input (bottom)
    let width = 40usize; let height = 8usize; // small demo surface
    let caps = get_acs_caps();
    let mut prev = vec![0u16; width*height];
    let mut cur = prev.clone();

    // Session for processing incoming bytes (MCCP->Telnet->ANSI->Scrollback)
    let mut session = Session::new(PassthroughDecomp::new(), width, height.saturating_sub(2), 200);
    // Input line buffer
    let mut input = mcl_rust::input_line::InputLine::new(width, 0x07);
    // Status line
    let mut status = mcl_rust::status_line::StatusLine::new(width, 0x07);
    status.set_text("MCL-Rust demo: type, Enter to echo; q quits");

    // Simple demo loop: read stdin nonblocking, normalize keys, print them; quit on 'q'
    unsafe { let _ = fcntl(libc::STDIN_FILENO, F_SETFL, O_NONBLOCK); }
    // Optional: try to connect if MCL_CONNECT=127.0.0.1:PORT is set
    let mut sock: Option<Socket> = None;
    if let Ok(addr) = std::env::var("MCL_CONNECT") {
        if let Some((ip_s, port_s)) = addr.split_once(':') {
            if let (Ok(ip), Ok(port)) = (ip_s.parse::<Ipv4Addr>(), port_s.parse::<u16>()) {
                let mut s = Socket::new().unwrap();
                let _ = s.connect_ipv4(ip, port);
                sock = Some(s);
                status.set_text(format!("Connecting to {}:{}...", ip, port));
            }
        }
    }

    let mut dec = KeyDecoder::new();
    let start = Instant::now();
    let timeout = Duration::from_secs(2); // short demo
    let mut buf = [0u8; 1024];
    loop {
        // poll stdin + socket for events
        let mut fds = vec![(libc::STDIN_FILENO, READ)];
        if let Some(s) = &sock {
            let mut ev = READ;
            if s.state == ConnState::Connecting { ev |= WRITE; }
            fds.push((s.as_raw_fd(), ev));
        }
        let ready = poll_fds(&fds, 100).unwrap_or_default();
        for (fd, r) in ready {
            if fd == libc::STDIN_FILENO && (r.revents & READ) != 0 {
                if let Ok(n) = io::stdin().read(&mut buf) { if n>0 {
                    for ev in dec.feed(&buf[..n]) {
                        match ev {
                            KeyEvent::Byte(b'\n') => { let line = input.take_line(); if !line.is_empty() { session.scrollback.print_line(&line, 0x07); } }
                            KeyEvent::Byte(b'q') => { status.set_text("Quit."); render_surface(width, height, &mut prev, &mut cur, &session, &input, &status, &caps); let _ = tty.keypad_application_mode(false); return; }
                            KeyEvent::Byte(b) if b.is_ascii_graphic() || b==b' ' => input.insert(b),
                            KeyEvent::Key(KeyCode::ArrowLeft) => input.move_left(),
                            KeyEvent::Key(KeyCode::ArrowRight) => input.move_right(),
                            KeyEvent::Key(KeyCode::Home) => input.home(),
                            KeyEvent::Key(KeyCode::End) => input.end(),
                            KeyEvent::Key(KeyCode::Delete) => input.backspace(),
                            _ => {}
                        }
                    }
                }}
            } else if let Some(s) = &mut sock {
                if fd == s.as_raw_fd() {
                    if (r.revents & WRITE) != 0 && s.state == ConnState::Connecting { let _ = s.on_writable(); if s.state==ConnState::Connected { status.set_text("Connected."); } }
                    if (r.revents & READ) != 0 {
                        // Read from socket and feed session
                        let n = unsafe { libc::read(s.as_raw_fd(), buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
                        if n>0 { session.feed(&buf[..n as usize]); }
                    }
                }
            }
        }
        render_surface(width, height, &mut prev, &mut cur, &session, &input, &status, &caps);
        if start.elapsed() > timeout { break; }
    }

    // Restore keypad mode will be handled by Drop, but be explicit
    let _ = tty.keypad_application_mode(false);
}

fn render_surface(width: usize, height: usize, prev: &mut Vec<u16>, cur: &mut Vec<u16>, session: &Session<PassthroughDecomp>, input: &mcl_rust::input_line::InputLine, status: &mcl_rust::status_line::StatusLine, caps: &mcl_rust::curses::AcsCaps) {
    // Compose status + session viewport + input into `cur`
    let mut surface = vec![0u16; width*height];
    // Status at row 0
    surface[0..width].copy_from_slice(&status.render());
    // Output rows (1..height-1)
    let view = session.scrollback.viewport_slice();
    let out_h = height.saturating_sub(2);
    for row in 0..out_h {
        let dst = (1+row) * width;
        let src = row * width;
        surface[dst .. dst+width].copy_from_slice(&view[src .. src+width]);
    }
    // Input at bottom row
    let input_row = height-1;
    surface[input_row*width .. input_row*width + width].copy_from_slice(&input.render());

    cur.copy_from_slice(&surface);
    let ansi = screen::diff_to_ansi(prev, cur, &DiffOptions{ width, height, cursor_x: 0, cursor_y: input_row, smacs: caps.smacs.as_deref(), rmacs: caps.rmacs.as_deref(), set_bg_always: true });
    let mut out = io::stdout(); let _ = out.write_all(ansi.as_bytes()); let _ = out.flush();
    prev.copy_from_slice(cur);
}
