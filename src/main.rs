use std::io::{self, Read, Write, BufRead};
use okros::input::{KeyDecoder, KeyEvent, KeyCode};
use okros::screen::{self, DiffOptions};
use okros::curses::get_acs_caps;
use okros::session::Session;
use okros::mccp::PassthroughDecomp;
use okros::engine::SessionEngine;
use okros::control::{ControlServer, default_socket_path};
use okros::select::{poll_fds, READ, WRITE};
use okros::socket::{Socket, ConnState};
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

    // Initialize embedded interpreters (matching main.cc:64, 101-105)
    #[cfg(feature = "python")]
    let mut python_interp = {
        use okros::plugins::python::PythonInterpreter;
        match PythonInterpreter::new() {
            Ok(interp) => {
                println!("Python interpreter initialized");
                Some(interp)
            }
            Err(e) => {
                eprintln!("Failed to initialize Python: {}", e);
                None
            }
        }
    };

    #[cfg(feature = "perl")]
    let mut perl_interp = {
        use okros::plugins::perl::PerlPlugin;
        match PerlPlugin::new() {
            Ok(interp) => {
                println!("Perl interpreter initialized");
                Some(interp)
            }
            Err(e) => {
                eprintln!("Failed to initialize Perl: {}", e);
                None
            }
        }
    };

    // Set initial interpreter variables (main.cc:101-105)
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    #[cfg(feature = "python")]
    if let Some(ref mut interp) = python_interp {
        use okros::plugins::stack::Interpreter;
        interp.set_int("now", current_time);
        interp.set_str("VERSION", env!("CARGO_PKG_VERSION"));
        interp.set_str("commandCharacter", "#");
        // Run sys/init script if it exists
        let mut out = String::new();
        let _ = interp.run_quietly("sys/init", "", &mut out, true);
    }

    #[cfg(feature = "perl")]
    if let Some(ref mut interp) = perl_interp {
        use okros::plugins::stack::Interpreter;
        interp.set_int("now", current_time);
        interp.set_str("VERSION", env!("CARGO_PKG_VERSION"));
        interp.set_str("commandCharacter", "#");
        // Run sys/init script if it exists
        let mut out = String::new();
        let _ = interp.run_quietly("sys/init", "", &mut out, true);
    }

    // Minimal demo: set raw mode, optional keypad app mode, then run a tiny event loop
    let mut tty = match okros::tty::Tty::new() { Ok(t) => t, Err(e) => { eprintln!("tty init failed: {}", e); return; } };
    let _ = tty.enable_raw();
    let _ = tty.keypad_application_mode(true);

    // Compose a small UI: status (top), output (middle), input (bottom)
    let width = 40usize; let height = 8usize; // small demo surface
    let caps = get_acs_caps();
    let mut prev = vec![0u16; width*height];
    let mut cur = prev.clone();

    // Session for processing incoming bytes (MCCP->Telnet->ANSI->Scrollback)
    let mut session = Session::new(PassthroughDecomp::new(), width, height.saturating_sub(2), 200);
    // Input line buffer
    let mut input = okros::input_line::InputLine::new(width, 0x07);
    // Status line
    let mut status = okros::status_line::StatusLine::new(width, 0x07);
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
    let mut buf = [0u8; 1024];
    let mut quit = false;
    let mut last_callout_time = current_time;

    // Main event loop (matching main.cc:141-170)
    while !quit {
        // 1. Render UI (main.cc:142)
        render_surface(width, height, &mut prev, &mut cur, &session, &input, &status, &caps);

        // 2. Poll file descriptors (main.cc:147) - stdin + socket with 250ms timeout
        let mut fds = vec![(libc::STDIN_FILENO, READ)];
        if let Some(s) = &sock {
            let mut ev = READ;
            if s.state == ConnState::Connecting { ev |= WRITE; }
            fds.push((s.as_raw_fd(), ev));
        }
        let ready = poll_fds(&fds, 250).unwrap_or_default();

        // 3. Process I/O events
        for (fd, r) in ready {
            if fd == libc::STDIN_FILENO && (r.revents & READ) != 0 {
                // TTY input (keyboard)
                if let Ok(n) = io::stdin().read(&mut buf) { if n>0 {
                    for ev in dec.feed(&buf[..n]) {
                        match ev {
                            KeyEvent::Byte(b'\n') => {
                                let line = input.take_line();
                                if !line.is_empty() {
                                    // Check for # commands (basic interpreter)
                                    if line.starts_with(b"#quit") {
                                        quit = true;
                                        status.set_text("Quit.");
                                    } else if line.starts_with(b"#open ") {
                                        // #open <host> <port>
                                        let args = String::from_utf8_lossy(&line[6..]);
                                        if let Some((host_str, port_str)) = args.trim().split_once(' ') {
                                            if let Ok(port) = port_str.parse::<u16>() {
                                                // Parse hostname (support IPv4 for now)
                                                if let Ok(ip) = host_str.parse::<Ipv4Addr>() {
                                                    let mut s = Socket::new().unwrap();
                                                    let _ = s.connect_ipv4(ip, port);
                                                    sock = Some(s);
                                                    status.set_text(format!("Connecting to {}:{}...", host_str, port));
                                                } else {
                                                    status.set_text(format!("Invalid IP: {}", host_str));
                                                }
                                            } else {
                                                status.set_text("Usage: #open <ip> <port>");
                                            }
                                        } else {
                                            status.set_text("Usage: #open <ip> <port>");
                                        }
                                    } else if line.starts_with(b"#") {
                                        // Other # commands - just echo for now
                                        session.scrollback.print_line(&line, 0x07);
                                    } else if let Some(ref mut s) = sock {
                                        // Send to MUD
                                        let mut send_buf = line.clone();
                                        send_buf.push(b'\n');
                                        unsafe { libc::write(s.as_raw_fd(), send_buf.as_ptr() as *const libc::c_void, send_buf.len()); }
                                    } else {
                                        // No socket - just echo
                                        session.scrollback.print_line(&line, 0x07);
                                    }
                                }
                            }
                            KeyEvent::Byte(b) if b.is_ascii_graphic() || b==b' ' => input.insert(b),
                            KeyEvent::Key(KeyCode::ArrowLeft) => input.move_left(),
                            KeyEvent::Key(KeyCode::ArrowRight) => input.move_right(),
                            KeyEvent::Key(KeyCode::Home) => input.home(),
                            KeyEvent::Key(KeyCode::End) => input.end(),
                            KeyEvent::Key(KeyCode::Delete) => input.backspace(),
                            KeyEvent::Byte(0x7f) | KeyEvent::Byte(0x08) => input.backspace(), // Backspace key
                            _ => {}
                        }
                    }
                }}
            } else if let Some(s) = &mut sock {
                if fd == s.as_raw_fd() {
                    // Socket writable (connection completing)
                    if (r.revents & WRITE) != 0 && s.state == ConnState::Connecting {
                        let _ = s.on_writable();
                        if s.state == ConnState::Connected {
                            status.set_text("Connected.");
                        }
                    }
                    // Socket readable (MUD data)
                    if (r.revents & READ) != 0 {
                        let n = unsafe { libc::read(s.as_raw_fd(), buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
                        if n > 0 {
                            session.feed(&buf[..n as usize]);
                        } else if n == 0 {
                            // Connection closed
                            status.set_text("Connection closed.");
                            sock = None;
                        }
                    }
                }
            }
        }

        // 4. Run interpreter hooks (main.cc:149)
        #[cfg(feature = "python")]
        if let Some(ref mut interp) = python_interp {
            use okros::plugins::stack::Interpreter;
            let mut out = String::new();
            let _ = interp.run_quietly("sys/postoutput", "", &mut out, true);
        }

        #[cfg(feature = "perl")]
        if let Some(ref mut interp) = perl_interp {
            use okros::plugins::stack::Interpreter;
            let mut out = String::new();
            let _ = interp.run_quietly("sys/postoutput", "", &mut out, true);
        }

        // 5. Session idle callbacks (main.cc:155) - time updates, etc.
        // (not implemented yet in Session)

        // 6. Widget idle callbacks (main.cc:160)
        // (not implemented yet in widgets)

        // 7. Timed interpreter callouts (main.cc:161 - EmbeddedInterpreter::runCallouts)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        if now != last_callout_time {
            last_callout_time = now;

            #[cfg(feature = "python")]
            if let Some(ref mut interp) = python_interp {
                use okros::plugins::stack::Interpreter;
                interp.set_int("now", now);
                let mut out = String::new();
                let _ = interp.run_quietly("sys/idle", "", &mut out, true);
            }

            #[cfg(feature = "perl")]
            if let Some(ref mut interp) = perl_interp {
                use okros::plugins::stack::Interpreter;
                interp.set_int("now", now);
                let mut out = String::new();
                let _ = interp.run_quietly("sys/idle", "", &mut out, true);
            }
        }
    }

    // Restore keypad mode will be handled by Drop, but be explicit
    let _ = tty.keypad_application_mode(false);
}

fn render_surface(width: usize, height: usize, prev: &mut Vec<u16>, cur: &mut Vec<u16>, session: &Session<PassthroughDecomp>, input: &okros::input_line::InputLine, status: &okros::status_line::StatusLine, caps: &okros::curses::AcsCaps) {
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
