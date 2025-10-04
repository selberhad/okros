use libc::{fcntl, F_SETFL, O_NONBLOCK};
use okros::control::{default_socket_path, ControlServer};
use okros::curses::get_acs_caps;
use okros::engine::SessionEngine;
use okros::input::{KeyCode, KeyDecoder, KeyEvent};
use okros::mccp::PassthroughDecomp;
use okros::screen::{self, DiffOptions};
use okros::select::{poll_fds, READ, WRITE};
use okros::session::Session;
use okros::socket::{ConnState, Socket};
use std::io::{self, BufRead, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs};

/// Resolve hostname to IPv4 address
/// Supports both hostnames (e.g., "nodeka.com") and IPv4 addresses (e.g., "127.0.0.1")
fn resolve_hostname(hostname: &str, port: u16) -> Result<Ipv4Addr, String> {
    // First, try parsing as IPv4 address directly
    if let Ok(ip) = hostname.parse::<Ipv4Addr>() {
        return Ok(ip);
    }

    // If not a direct IP, do DNS resolution
    let addr_str = format!("{}:{}", hostname, port);
    match addr_str.to_socket_addrs() {
        Ok(mut addrs) => {
            // Find first IPv4 address
            if let Some(SocketAddr::V4(v4_addr)) = addrs.find(|a| a.is_ipv4()) {
                Ok(*v4_addr.ip())
            } else {
                Err(format!("No IPv4 address found for {}", hostname))
            }
        }
        Err(e) => Err(format!("DNS lookup failed for {}: {}", hostname, e)),
    }
}

fn main() {
    // CLI: --headless [--offline] --instance NAME | --attach NAME | --offline
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 && args[1] == "--headless" {
        // Check for --offline flag in args
        let offline = args.iter().any(|a| a == "--offline");

        if offline {
            // Headless offline mode: control socket + internal MUD
            run_headless_offline_mode(&args);
            return;
        } else {
            // Regular headless mode: control socket + network
            let inst = args
                .get(3)
                .cloned()
                .unwrap_or_else(|| "default".to_string());
            let path = default_socket_path(&inst);
            let eng = SessionEngine::new(PassthroughDecomp::new(), 80, 20, 2000);
            let srv = ControlServer::new(path.clone(), eng);
            eprintln!("Headless engine; control socket at {}", path.display());
            let _ = srv.run();
            return;
        }
    } else if args.len() > 2 && args[1] == "--attach" {
        let inst = args
            .get(2)
            .cloned()
            .unwrap_or_else(|| "default".to_string());
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
            Err(e) => {
                eprintln!("attach failed: {}", e);
            }
        }
        return;
    } else if args.len() > 1 && args[1] == "--offline" {
        // Offline mode: internal MUD
        run_offline_mode();
        return;
    }

    // Interactive TTY mode - suppress stdout before entering UI
    // (messages would corrupt the screen)

    // Initialize embedded interpreters (matching main.cc:64, 101-105)
    #[cfg(feature = "python")]
    let mut python_interp = {
        use okros::plugins::python::PythonInterpreter;
        PythonInterpreter::new().ok()
    };

    #[cfg(feature = "perl")]
    let mut perl_interp = {
        use okros::plugins::perl::PerlPlugin;
        PerlPlugin::new().ok()
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

    // Interactive TTY mode: set raw mode, clear screen, hide cursor
    let mut tty = match okros::tty::Tty::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("tty init failed: {}", e);
            return;
        }
    };
    let _ = tty.enable_raw();
    let _ = tty.keypad_application_mode(true);

    // Clear screen and hide cursor
    print!("\x1b[2J\x1b[H\x1b[?25l");
    std::io::stdout().flush().unwrap();

    // Get terminal size (C++ Screen.cc:16-34)
    let (width, height) = unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDIN_FILENO, libc::TIOCGWINSZ, &mut ws) < 0 {
            eprintln!("Failed to get terminal size, using 80x24");
            (80usize, 24usize)
        } else {
            (ws.ws_col as usize, ws.ws_row as usize)
        }
    };
    let caps = get_acs_caps();
    // Initialize prev to non-zero to force full render on first frame
    let mut prev = vec![0xFFFFu16; width * height];
    let mut cur = vec![0u16; width * height];

    // Session for processing incoming bytes (MCCP->Telnet->ANSI->Scrollback)
    let mut session = Session::new(
        PassthroughDecomp::new(),
        width,
        height.saturating_sub(2),
        2000,
    );
    // Input line buffer (0x17 = blue background, white foreground)
    let mut input = okros::input_line::InputLine::new(width, 0x17);
    // Status line (0x07 = black background, white foreground)
    let mut status = okros::status_line::StatusLine::new(width, 0x07);
    status.set_text("okros v0.1 - Press Alt-O for connect menu, #quit to exit");

    // Simple demo loop: read stdin nonblocking, normalize keys, print them; quit on 'q'
    unsafe {
        let _ = fcntl(libc::STDIN_FILENO, F_SETFL, O_NONBLOCK);
    }
    // MUD instance (contains socket + aliases/actions/macros)
    let mut mud = okros::mud::Mud::empty();
    // Optional: try to connect if OKROS_CONNECT=hostname:PORT is set
    let mut sock: Option<Socket> = None;
    if let Ok(addr) = std::env::var("OKROS_CONNECT") {
        if let Some((host, port_s)) = addr.split_once(':') {
            if let Ok(port) = port_s.parse::<u16>() {
                match resolve_hostname(host, port) {
                    Ok(ip) => {
                        let mut s = Socket::new().unwrap();
                        let _ = s.connect_ipv4(ip, port);
                        sock = Some(s);
                        status.set_text(format!("Connecting to {}:{} -> {}...", host, port, ip));
                    }
                    Err(e) => {
                        status.set_text(format!("OKROS_CONNECT DNS error: {}", e));
                    }
                }
            }
        }
    }

    let mut dec = KeyDecoder::new();
    let mut buf = [0u8; 1024];
    let mut quit = false;
    let mut last_callout_time = current_time;

    // Modal state for connect menu
    enum ModalState {
        Normal,
        ConnectMenu(okros::mud_selection::MudSelection),
    }
    let mut modal = ModalState::Normal;

    // Main event loop (matching main.cc:141-170)
    while !quit {
        // 1. Render UI (main.cc:142)
        if let ModalState::ConnectMenu(ref menu) = modal {
            // Render connect menu modal
            render_connect_menu(menu, width, height);
        } else {
            // Normal UI rendering
            render_surface(
                width, height, &mut prev, &mut cur, &session, &input, &status, &caps,
            );
        }

        // 2. Poll file descriptors (main.cc:147) - stdin + socket with 250ms timeout
        let mut fds = vec![(libc::STDIN_FILENO, READ)];
        if let Some(s) = &sock {
            let mut ev = READ;
            if s.state == ConnState::Connecting {
                ev |= WRITE;
            }
            fds.push((s.as_raw_fd(), ev));
        }
        let ready = poll_fds(&fds, 250).unwrap_or_default();

        // 3. Process I/O events
        for (fd, r) in ready {
            if fd == libc::STDIN_FILENO && (r.revents & READ) != 0 {
                // TTY input (keyboard)
                if let Ok(n) = io::stdin().read(&mut buf) {
                    if n > 0 {
                        for ev in dec.feed(&buf[..n]) {
                            // Handle modal connect menu first
                            if let ModalState::ConnectMenu(ref mut menu) = modal {
                                if menu.keypress(ev) {
                                    // Enter pressed - connect to selected MUD
                                    if matches!(ev, KeyEvent::Byte(b'\n')) {
                                        let idx = menu.get_selection();
                                        if let Some((name, hostname, port)) =
                                            menu.get_mud_at(idx as usize)
                                        {
                                            // Check if this is the Offline MUD (no hostname)
                                            if hostname.is_empty() {
                                                status.set_text(
                                                    "Offline MUD - use cargo run --offline instead",
                                                );
                                                modal = ModalState::Normal;
                                            } else {
                                                // Resolve hostname and connect to network MUD
                                                match resolve_hostname(hostname, port) {
                                                    Ok(ip) => {
                                                        let mut s = Socket::new().unwrap();
                                                        let _ = s.connect_ipv4(ip, port);
                                                        sock = Some(s);
                                                        status.set_text(format!(
                                                            "Connecting to {} ({}:{} -> {})...",
                                                            name, hostname, port, ip
                                                        ));
                                                        modal = ModalState::Normal;
                                                    }
                                                    Err(e) => {
                                                        status
                                                            .set_text(format!("DNS error: {}", e));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else if matches!(ev, KeyEvent::Key(KeyCode::Escape)) {
                                    // Escape pressed - exit connect menu
                                    modal = ModalState::Normal;
                                    status.set_text("Connect menu closed.");
                                }
                                continue; // Skip normal processing while in modal
                            }

                            // Alt-O: Open connect menu
                            if matches!(ev, KeyEvent::Key(KeyCode::Alt(b'o'))) {
                                // Load config file
                                let config_path = std::env::var("HOME")
                                    .map(|h| std::path::PathBuf::from(h).join(".okros/config"))
                                    .unwrap_or_else(|_| std::path::PathBuf::from(".okros/config"));

                                let mut config = okros::config::Config::new();
                                if config.load_file(&config_path).is_ok() {
                                    let menu = okros::mud_selection::MudSelection::new(
                                        config, width, height,
                                    );
                                    if menu.count() > 0 {
                                        modal = ModalState::ConnectMenu(menu);
                                        status.set_text("Select MUD (arrows to navigate, Enter to connect, Esc to cancel)");
                                    } else {
                                        status.set_text("No MUDs found in config");
                                    }
                                } else {
                                    status.set_text("Config file not found");
                                }
                                continue;
                            }

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
                                            if let Some((host_str, port_str)) =
                                                args.trim().split_once(' ')
                                            {
                                                if let Ok(port) = port_str.parse::<u16>() {
                                                    // Resolve hostname (supports both DNS and IPv4)
                                                    match resolve_hostname(host_str, port) {
                                                        Ok(ip) => {
                                                            let mut s = Socket::new().unwrap();
                                                            let _ = s.connect_ipv4(ip, port);
                                                            sock = Some(s);
                                                            status.set_text(format!(
                                                                "Connecting to {}:{} -> {}...",
                                                                host_str, port, ip
                                                            ));
                                                        }
                                                        Err(e) => {
                                                            status.set_text(format!(
                                                                "DNS error: {}",
                                                                e
                                                            ));
                                                        }
                                                    }
                                                } else {
                                                    status.set_text("Usage: #open <host> <port>");
                                                }
                                            } else {
                                                status.set_text("Usage: #open <host> <port>");
                                            }
                                        } else if line.starts_with(b"#alias ") {
                                            // #alias <name> <expansion>
                                            let args = String::from_utf8_lossy(&line[7..])
                                                .trim()
                                                .to_string();
                                            if let Some((name, text)) = args.split_once(' ') {
                                                use okros::alias::Alias;
                                                if let Some(pos) = mud
                                                    .alias_list
                                                    .iter()
                                                    .position(|a| a.name == name)
                                                {
                                                    mud.alias_list[pos] = Alias::new(name, text);
                                                    status.set_text(format!(
                                                        "Updated alias '{}' = {}",
                                                        name, text
                                                    ));
                                                } else {
                                                    mud.alias_list.push(Alias::new(name, text));
                                                    status.set_text(format!(
                                                        "Added alias '{}' = {}",
                                                        name, text
                                                    ));
                                                }
                                            } else if !args.is_empty() {
                                                // Remove alias
                                                mud.alias_list.retain(|a| a.name != args);
                                                status
                                                    .set_text(format!("Removed alias '{}'", args));
                                            } else {
                                                status.set_text("Usage: #alias <name> <expansion>");
                                            }
                                        } else if line.starts_with(b"#action ") {
                                            // #action <pattern> <commands>
                                            let args = String::from_utf8_lossy(&line[8..])
                                                .trim()
                                                .to_string();
                                            use okros::action::{Action, ActionType};
                                            match Action::parse(&args, ActionType::Trigger) {
                                                Ok(mut action) => {
                                                    // Compile action with available interpreter
                                                    #[cfg(feature = "perl")]
                                                    if let Some(ref mut interp) = perl_interp {
                                                        use okros::plugins::stack::Interpreter;
                                                        action.compile(interp);
                                                    }
                                                    #[cfg(all(
                                                        feature = "python",
                                                        not(feature = "perl")
                                                    ))]
                                                    if let Some(ref mut interp) = python_interp {
                                                        use okros::plugins::stack::Interpreter;
                                                        action.compile(interp);
                                                    }

                                                    mud.action_list
                                                        .retain(|a| a.pattern != action.pattern);
                                                    status.set_text(format!(
                                                        "Added trigger: {} => {}",
                                                        action.pattern, action.commands
                                                    ));
                                                    mud.action_list.push(action);
                                                }
                                                Err(e) => status.set_text(e),
                                            }
                                        } else if line.starts_with(b"#subst ") {
                                            // #subst <pattern> <replacement>
                                            let args = String::from_utf8_lossy(&line[7..])
                                                .trim()
                                                .to_string();
                                            use okros::action::{Action, ActionType};
                                            match Action::parse(&args, ActionType::Replacement) {
                                                Ok(mut action) => {
                                                    // Compile substitution with available interpreter
                                                    #[cfg(feature = "perl")]
                                                    if let Some(ref mut interp) = perl_interp {
                                                        use okros::plugins::stack::Interpreter;
                                                        action.compile(interp);
                                                    }
                                                    #[cfg(all(
                                                        feature = "python",
                                                        not(feature = "perl")
                                                    ))]
                                                    if let Some(ref mut interp) = python_interp {
                                                        use okros::plugins::stack::Interpreter;
                                                        action.compile(interp);
                                                    }

                                                    mud.action_list
                                                        .retain(|a| a.pattern != action.pattern);
                                                    status.set_text(format!(
                                                        "Added substitute: {} => {}",
                                                        action.pattern, action.commands
                                                    ));
                                                    mud.action_list.push(action);
                                                }
                                                Err(e) => status.set_text(e),
                                            }
                                        } else if line.starts_with(b"#macro ") {
                                            // #macro <keyname> <text>
                                            let args = String::from_utf8_lossy(&line[7..])
                                                .trim()
                                                .to_string();
                                            if let Some((key_name, text)) = args.split_once(' ') {
                                                // For now, just use ASCII value as key code
                                                // TODO: implement key_lookup() like C++ for named keys (F1, etc.)
                                                if let Some(ch) = key_name.chars().next() {
                                                    use okros::macro_def::Macro;
                                                    let key = ch as i32;
                                                    mud.macro_list.retain(|m| m.key != key);
                                                    mud.macro_list.push(Macro::new(key, text));
                                                    status.set_text(format!(
                                                        "Added macro: {} => {}",
                                                        key_name, text
                                                    ));
                                                } else {
                                                    status.set_text("Invalid key name");
                                                }
                                            } else {
                                                status.set_text("Usage: #macro <key> <text>");
                                            }
                                        } else if line.starts_with(b"#") {
                                            // Other # commands - just echo for now
                                            session.scrollback.print_line(&line, 0x07);
                                        } else {
                                            // Check for alias expansion
                                            let line_str = String::from_utf8_lossy(&line);
                                            let mut send_text = line_str.to_string();

                                            // Extract first word (command name)
                                            if let Some(first_word_end) =
                                                line_str.find(char::is_whitespace)
                                            {
                                                let cmd = &line_str[..first_word_end];
                                                let args = &line_str[first_word_end..].trim_start();

                                                // Check if command is an alias
                                                if let Some(alias) = mud.find_alias(cmd) {
                                                    send_text = alias.expand(args);
                                                    status.set_text(format!(
                                                        "{} -> {}",
                                                        cmd, send_text
                                                    ));
                                                }
                                            } else {
                                                // No arguments, check if entire line is an alias
                                                if let Some(alias) = mud.find_alias(&line_str) {
                                                    send_text = alias.expand("");
                                                    status.set_text(format!(
                                                        "{} -> {}",
                                                        line_str.trim(),
                                                        send_text
                                                    ));
                                                }
                                            }

                                            // Send to MUD (or echo if no socket)
                                            if let Some(ref mut s) = sock {
                                                let mut send_buf = send_text.into_bytes();
                                                send_buf.push(b'\n');
                                                unsafe {
                                                    libc::write(
                                                        s.as_raw_fd(),
                                                        send_buf.as_ptr() as *const libc::c_void,
                                                        send_buf.len(),
                                                    );
                                                }
                                            } else {
                                                session
                                                    .scrollback
                                                    .print_line(&send_text.as_bytes(), 0x07);
                                            }
                                        }
                                    }
                                }
                                KeyEvent::Byte(b) if b.is_ascii_graphic() || b == b' ' => {
                                    input.insert(b)
                                }
                                KeyEvent::Key(KeyCode::ArrowLeft) => input.move_left(),
                                KeyEvent::Key(KeyCode::ArrowRight) => input.move_right(),
                                KeyEvent::Key(KeyCode::Home) => input.home(),
                                KeyEvent::Key(KeyCode::End) => input.end(),
                                KeyEvent::Key(KeyCode::Delete) => input.backspace(),
                                KeyEvent::Byte(0x7f) | KeyEvent::Byte(0x08) => input.backspace(), // Backspace key
                                _ => {}
                            }
                        }
                    }
                }
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
                        let n = unsafe {
                            libc::read(
                                s.as_raw_fd(),
                                buf.as_mut_ptr() as *mut libc::c_void,
                                buf.len(),
                            )
                        };
                        if n > 0 {
                            session.feed(&buf[..n as usize]);

                            // Check triggers/actions on current incomplete line
                            // TODO: This should check completed lines from scrollback,
                            // but for MVP we check the current incomplete line
                            let current_line = session.current_line();
                            if !current_line.is_empty() {
                                let line_str = String::from_utf8_lossy(&current_line);

                                // Check triggers with available interpreter
                                #[cfg(feature = "perl")]
                                if let Some(ref mut interp) = perl_interp {
                                    use okros::action::ActionType;
                                    use okros::plugins::stack::Interpreter;

                                    for action in &mud.action_list {
                                        if action.action_type == ActionType::Trigger {
                                            if let Some(commands) =
                                                action.check_match(&line_str, interp)
                                            {
                                                // Trigger matched - execute commands
                                                // For now, just send the commands to MUD
                                                if let Some(ref mut s) = sock {
                                                    let mut cmd_buf = commands.into_bytes();
                                                    cmd_buf.push(b'\n');
                                                    unsafe {
                                                        libc::write(
                                                            s.as_raw_fd(),
                                                            cmd_buf.as_ptr() as *const libc::c_void,
                                                            cmd_buf.len(),
                                                        );
                                                    }
                                                    status.set_text(format!(
                                                        "Trigger fired: {}",
                                                        action.pattern
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }

                                #[cfg(all(feature = "python", not(feature = "perl")))]
                                if let Some(ref mut interp) = python_interp {
                                    use okros::action::ActionType;
                                    use okros::plugins::stack::Interpreter;

                                    for action in &mud.action_list {
                                        if action.action_type == ActionType::Trigger {
                                            if let Some(commands) =
                                                action.check_match(&line_str, interp)
                                            {
                                                // Trigger matched - execute commands
                                                if let Some(ref mut s) = sock {
                                                    let mut cmd_buf = commands.into_bytes();
                                                    cmd_buf.push(b'\n');
                                                    unsafe {
                                                        libc::write(
                                                            s.as_raw_fd(),
                                                            cmd_buf.as_ptr() as *const libc::c_void,
                                                            cmd_buf.len(),
                                                        );
                                                    }
                                                    status.set_text(format!(
                                                        "Trigger fired: {}",
                                                        action.pattern
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
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

    // Restore keypad mode, show cursor, clear screen
    let _ = tty.keypad_application_mode(false);
    print!("\x1b[?25h\x1b[2J\x1b[H");
    std::io::stdout().flush().unwrap();
}

fn run_offline_mode() {
    use okros::offline_mud::{parse, World};

    // Initialize internal MUD
    let mut world = World::new();

    // Set up TTY
    let mut tty = match okros::tty::Tty::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("tty init failed: {}", e);
            return;
        }
    };
    let _ = tty.enable_raw();
    let _ = tty.keypad_application_mode(true);

    // UI setup
    let width = 80usize;
    let height = 24usize;
    let caps = get_acs_caps();
    let mut prev = vec![0u16; width * height];
    let mut cur = prev.clone();

    // Session for processing output
    let mut session = Session::new(
        PassthroughDecomp::new(),
        width,
        height.saturating_sub(2),
        200,
    );
    // Input line
    let mut input = okros::input_line::InputLine::new(width, 0x07);
    // Status line
    let mut status = okros::status_line::StatusLine::new(width, 0x07);
    status.set_text("Internal MUD - type 'help' for commands, 'quit' to exit");

    // Show initial room
    let look_output = world.execute(parse("look").unwrap());
    session.feed(look_output.as_bytes());

    // Set stdin nonblocking
    unsafe {
        let _ = fcntl(libc::STDIN_FILENO, F_SETFL, O_NONBLOCK);
    }

    let mut dec = KeyDecoder::new();
    let mut buf = [0u8; 1024];
    let mut quit = false;

    // Main event loop for offline mode
    while !quit {
        // Render UI
        render_surface(
            width, height, &mut prev, &mut cur, &session, &input, &status, &caps,
        );

        // Poll stdin with 250ms timeout
        let fds = vec![(libc::STDIN_FILENO, READ)];
        let ready = poll_fds(&fds, 250).unwrap_or_default();

        // Process input
        for (fd, r) in ready {
            if fd == libc::STDIN_FILENO && (r.revents & READ) != 0 {
                if let Ok(n) = io::stdin().read(&mut buf) {
                    if n > 0 {
                        for ev in dec.feed(&buf[..n]) {
                            match ev {
                                KeyEvent::Byte(b'\n') => {
                                    let line = input.take_line();
                                    if !line.is_empty() {
                                        let cmd_str = String::from_utf8_lossy(&line).to_string();

                                        // Parse and execute MUD command
                                        match parse(&cmd_str) {
                                            Ok(cmd) => {
                                                // Check for quit command
                                                if matches!(
                                                    cmd,
                                                    okros::offline_mud::parser::Command::Quit
                                                ) {
                                                    quit = true;
                                                }
                                                let output = world.execute(cmd);
                                                session.feed(output.as_bytes());
                                            }
                                            Err(e) => {
                                                // Parse error - show in red
                                                let err_msg = format!("\x1b[31m{}\x1b[0m\n", e);
                                                session.feed(err_msg.as_bytes());
                                            }
                                        }
                                    }
                                }
                                KeyEvent::Byte(b) if b.is_ascii_graphic() || b == b' ' => {
                                    input.insert(b)
                                }
                                KeyEvent::Key(KeyCode::ArrowLeft) => input.move_left(),
                                KeyEvent::Key(KeyCode::ArrowRight) => input.move_right(),
                                KeyEvent::Key(KeyCode::Home) => input.home(),
                                KeyEvent::Key(KeyCode::End) => input.end(),
                                KeyEvent::Key(KeyCode::Delete) => input.backspace(),
                                KeyEvent::Byte(0x7f) | KeyEvent::Byte(0x08) => input.backspace(),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    // Restore keypad mode
    let _ = tty.keypad_application_mode(false);
}

fn run_headless_offline_mode(args: &[String]) {
    use okros::offline_mud::{parse, World};
    use serde_json::json;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    use std::thread;

    // Parse instance name from args
    let inst = args
        .iter()
        .position(|a| a == "--instance")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "default".to_string());

    let path = default_socket_path(&inst);

    // Remove existing socket if present
    let _ = std::fs::remove_file(&path);

    // Create Unix socket listener
    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind socket: {}", e);
            return;
        }
    };

    eprintln!("Headless offline MUD; control socket at {}", path.display());

    // Server state: World + Session
    struct OfflineMudServer {
        world: World,
        session: Session<PassthroughDecomp>,
    }

    impl OfflineMudServer {
        fn new() -> Self {
            let mut world = World::new();
            let session = Session::new(PassthroughDecomp::new(), 80, 24, 2000);

            // Show initial room
            let initial = world.execute(parse("look").unwrap());
            let mut server = Self { world, session };
            server.session.feed(initial.as_bytes());
            server
        }

        fn handle_command(&mut self, cmd_json: &str) -> String {
            let cmd: serde_json::Value = match serde_json::from_str(cmd_json) {
                Ok(v) => v,
                Err(_) => return json!({"event":"Error","message":"Invalid JSON"}).to_string(),
            };

            let cmd_type = cmd["cmd"].as_str().unwrap_or("");

            match cmd_type {
                "send" => {
                    let data = cmd["data"].as_str().unwrap_or("");

                    // Parse MUD command
                    match parse(data.trim()) {
                        Ok(mud_cmd) => {
                            // Execute in World
                            let output = self.world.execute(mud_cmd);

                            // Feed to Session pipeline (ANSI → scrollback)
                            self.session.feed(output.as_bytes());

                            json!({"event":"Ok"}).to_string()
                        }
                        Err(e) => {
                            // Parse error - show in session
                            let err_msg = format!("\x1b[31m{}\x1b[0m\n", e);
                            self.session.feed(err_msg.as_bytes());
                            json!({"event":"Ok"}).to_string()
                        }
                    }
                }
                "get_buffer" => {
                    // Extract scrollback as lines
                    let viewport = self.session.scrollback.viewport_slice();
                    let text: String = viewport.iter().map(|&a| (a & 0xFF) as u8 as char).collect();

                    let lines: Vec<String> = text
                        .lines()
                        .map(|s| s.trim_end().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    json!({"event":"Buffer","lines":lines}).to_string()
                }
                "status" => {
                    let location = self.world.player.location;
                    let inv_count = self.world.player.inventory.len();
                    json!({
                        "event":"Status",
                        "location":location,
                        "inventory_count":inv_count
                    })
                    .to_string()
                }
                _ => json!({"event":"Error","message":"Unknown command"}).to_string(),
            }
        }
    }

    // Create shared server state
    use std::sync::{Arc, Mutex};
    let server = Arc::new(Mutex::new(OfflineMudServer::new()));

    // Accept connections and handle them
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let server_clone = server.clone();
                thread::spawn(move || {
                    let mut reader = BufReader::new(match s.try_clone() {
                        Ok(s) => s,
                        Err(_) => return,
                    });
                    let mut writer = s;
                    let mut line = String::new();

                    while reader.read_line(&mut line).unwrap_or(0) > 0 {
                        let response = {
                            let mut srv = server_clone.lock().unwrap();
                            srv.handle_command(line.trim())
                        };
                        if writeln!(writer, "{}", response).is_err() {
                            break;
                        }
                        line.clear();
                    }
                });
            }
            Err(e) => eprintln!("control: accept error: {}", e),
        }
    }
}

fn render_connect_menu(menu: &okros::mud_selection::MudSelection, _width: usize, _height: usize) {
    use std::io::Write;

    // Clear screen and position cursor at top
    print!("\x1b[2J\x1b[H");

    // Title
    println!("\x1b[1;36m=== MUD Connect Menu ===\x1b[0m\n");

    // Instructions
    println!("\x1b[33mArrows: Navigate | Enter: Connect | Esc: Cancel\x1b[0m\n");

    // Menu items
    let selection = menu.get_selection();
    let count = menu.count();

    for i in 0..count.min(20) {
        // Show max 20 items
        if i as i32 == selection {
            print!("\x1b[1;32m> ");
        } else {
            print!("  ");
        }

        // Get MUD info at this index
        if let Some((name, hostname, port)) = menu.get_mud_at(i) {
            if hostname.is_empty() {
                // Offline MUD or MUD without hostname
                println!("{}\x1b[0m", name);
            } else {
                // Network MUD
                println!("{:<20} {}:{}\x1b[0m", name, hostname, port);
            }
        }
    }

    std::io::stdout().flush().unwrap();
}

fn render_surface(
    width: usize,
    height: usize,
    prev: &mut Vec<u16>,
    cur: &mut Vec<u16>,
    session: &Session<PassthroughDecomp>,
    input: &okros::input_line::InputLine,
    status: &okros::status_line::StatusLine,
    caps: &okros::curses::AcsCaps,
) {
    // Compose status + session viewport + input into `cur`
    let mut surface = vec![0u16; width * height];
    // Status at row 0
    surface[0..width].copy_from_slice(&status.render());
    // Output rows (1..height-1)
    let view = session.scrollback.viewport_slice();
    let out_h = height.saturating_sub(2);
    for row in 0..out_h {
        let dst = (1 + row) * width;
        let src = row * width;
        surface[dst..dst + width].copy_from_slice(&view[src..src + width]);
    }
    // Input at bottom row
    let input_row = height - 1;
    surface[input_row * width..input_row * width + width].copy_from_slice(&input.render());

    cur.copy_from_slice(&surface);
    let ansi = screen::diff_to_ansi(
        prev,
        cur,
        &DiffOptions {
            width,
            height,
            cursor_x: input.cursor,
            cursor_y: input_row,
            smacs: caps.smacs.as_deref(),
            rmacs: caps.rmacs.as_deref(),
            set_bg_always: true,
        },
    );
    let mut out = io::stdout();
    let _ = out.write_all(ansi.as_bytes());
    let _ = out.flush();
    prev.copy_from_slice(cur);
}
