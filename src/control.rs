use crate::engine::SessionEngine;
use crate::select::{poll_fds, READ, WRITE};
use crate::socket::{Socket, ConnState};
use crate::mccp::PassthroughDecomp;
use serde::{Deserialize, Serialize};
use std::net::ToSocketAddrs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Deserialize)]
struct Command {
    cmd: String,
    data: Option<String>,
    from: Option<u64>,
    interval_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "event")]
enum Event {
    Ok,
    Error { message: String },
    Status { attached: bool },
    Buffer { lines: Vec<String> },
}

pub struct ControlState {
    engine: Arc<Mutex<SessionEngine<PassthroughDecomp>>>,
    sock: Arc<Mutex<Option<Socket>>>,
}

pub struct ControlServer {
    path: PathBuf,
    state: Arc<ControlState>,
}

impl ControlServer {
    pub fn new(path: PathBuf, engine: SessionEngine<PassthroughDecomp>) -> Self {
        Self { path, state: Arc::new(ControlState{ engine: Arc::new(Mutex::new(engine)), sock: Arc::new(Mutex::new(None)) }) }
    }

    pub fn run(self) -> std::io::Result<()> {
        // Remove existing socket if present
        let _ = std::fs::remove_file(&self.path);
        let listener = UnixListener::bind(&self.path)?;
        let state = self.state.clone();
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let st = state.clone();
                    thread::spawn(move || {
                        let _ = handle_client(s, st);
                    });
                }
                Err(e) => eprintln!("control: accept error: {}", e),
            }
        }
        Ok(())
    }
}

fn handle_client(mut stream: UnixStream, state: Arc<ControlState>) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 { break; }
        let trimmed = line.trim_end();
        if trimmed.is_empty() { continue; }
        let cmd: serde_json::Result<Command> = serde_json::from_str(trimmed);
        match cmd {
            Ok(c) if c.cmd == "stream" => {
                // Enter streaming loop until client disconnects
                let interval = c.interval_ms.unwrap_or(200);
                let _ = stream_loop(&mut stream, &state.engine, interval as u64);
                break;
            }
            Ok(c) => {
                let response = handle_command(c, &state);
                let s = serde_json::to_string(&response).unwrap();
                writeln!(stream, "{}", s)?;
                stream.flush()?;
            }
            Err(e) => {
                let s = serde_json::to_string(&Event::Error { message: format!("bad json: {}", e) }).unwrap();
                writeln!(stream, "{}", s)?;
                stream.flush()?;
            }
        }
    }
    Ok(())
}

fn handle_command(cmd: Command, state: &Arc<ControlState>) -> Event {
    match cmd.cmd.as_str() {
        "status" => {
            let eng = state.engine.lock().unwrap();
            Event::Status { attached: eng.is_attached() }
        }
        "attach" => { let mut eng=state.engine.lock().unwrap(); eng.attach(); Event::Ok }
        "detach" => { let mut eng=state.engine.lock().unwrap(); eng.detach(); Event::Ok }
        "get_buffer" => {
            let eng = state.engine.lock().unwrap();
            let lines = eng.viewport_text();
            Event::Buffer { lines }
        }
        "connect" => {
            if let Some(addr) = &cmd.data {
                match resolve_ipv4(addr) {
                    Ok((ip,port)) => {
                        match Socket::new().and_then(|mut s| { let _ = s.connect_ipv4(ip,port); Ok(s) }) {
                            Ok(s) => {
                                *state.sock.lock().unwrap() = Some(s);
                                spawn_net_loop(state.clone());
                                Event::Ok
                            }
                            Err(e) => Event::Error { message: format!("connect: {}", e) }
                        }
                    }
                    Err(e) => Event::Error { message: format!("resolve: {}", e) }
                }
            } else { Event::Error { message: "missing data".to_string() } }
        }
        // Append data to the session buffer
        "send" => {
            if let Some(data) = cmd.data {
                let mut eng = state.engine.lock().unwrap();
                if !data.is_empty() { eng.session.scrollback.print_line(data.as_bytes(), 0x07); }
                Event::Ok
            } else { Event::Error { message: "missing data".to_string() } }
        }
        // Write raw bytes to the connected socket, if any
        "sock_send" => {
            if let Some(data) = cmd.data {
                if let Some(sock) = &mut *state.sock.lock().unwrap() {
                    unsafe {
                        let _ = libc::write(sock.as_raw_fd(), data.as_ptr() as *const libc::c_void, data.len());
                    }
                    Event::Ok
                } else {
                    Event::Error { message: "not connected".to_string() }
                }
            } else { Event::Error { message: "missing data".to_string() } }
        }
        _ => Event::Error { message: "unknown cmd".to_string() },
    }
}

fn stream_loop(stream: &mut UnixStream, engine: &Arc<Mutex<SessionEngine<PassthroughDecomp>>>, interval_ms: u64) -> std::io::Result<()> {
    loop {
        let lines = {
            let eng = engine.lock().unwrap();
            eng.viewport_text()
        };
        let evt = Event::Buffer { lines };
        let s = serde_json::to_string(&evt).unwrap();
        if writeln!(stream, "{}", s).is_err() { break; }
        if stream.flush().is_err() { break; }
        std::thread::sleep(std::time::Duration::from_millis(interval_ms));
    }
    Ok(())
}

pub fn default_socket_path(instance: &str) -> PathBuf {
    let base = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let mut p = PathBuf::from(base);
    p.push("okros"); let _ = std::fs::create_dir_all(&p);
    p.push(format!("{}.sock", instance));
    p
}

fn resolve_ipv4(addr: &str) -> std::io::Result<(std::net::Ipv4Addr, u16)> {
    let (host, port_str) = addr.split_once(':').ok_or_else(|| io_err("expected host:port"))?;
    let port: u16 = port_str.parse().map_err(|_| io_err("bad port"))?;
    let mut addrs = (host, port).to_socket_addrs()?;
    while let Some(sa) = addrs.next() {
        if let std::net::IpAddr::V4(ip) = sa.ip() { return Ok((ip, sa.port())); }
    }
    Err(io_err("no IPv4 address"))
}

fn io_err(msg: &str) -> std::io::Error { std::io::Error::new(std::io::ErrorKind::Other, msg) }

fn spawn_net_loop(state: Arc<ControlState>) {
    thread::spawn(move || {
        loop {
            let fd_ev = {
                let s = state.sock.lock().unwrap();
                if let Some(sock) = s.as_ref() {
                    let mut ev = READ;
                    if sock.state == ConnState::Connecting { ev |= WRITE; }
                    Some((sock.as_raw_fd(), ev))
                } else { None }
            };
            if fd_ev.is_none() { break; }
            let (fd, ev) = fd_ev.unwrap();
            let ready = poll_fds(&[(fd, ev)], 200).unwrap_or_default();
            for (_fd, r) in ready {
                let mut drop_sock = false;
                {
                    let mut s = state.sock.lock().unwrap();
                    if let Some(sock) = s.as_mut() {
                        if (r.revents & WRITE) != 0 && sock.state == ConnState::Connecting { let _ = sock.on_writable(); }
                        if (r.revents & READ) != 0 {
                            let mut buf = [0u8; 4096];
                            let n = unsafe { libc::read(sock.as_raw_fd(), buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
                            if n > 0 {
                                let mut eng = state.engine.lock().unwrap();
                                eng.feed_inbound(&buf[..n as usize]);
                            } else if n == 0 { drop_sock = true; }
                        }
                    }
                }
                if drop_sock { *state.sock.lock().unwrap() = None; }
            }
        }
    });
}
