use okros::control::{default_socket_path, ControlServer};
use okros::engine::SessionEngine;
use okros::mccp::PassthroughDecomp;
use std::io::{BufRead, Read, Write};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;

#[test]
fn control_server_send_and_get_buffer() {
    let inst = format!("test_{}", std::process::id());
    let path = default_socket_path(&inst);
    let eng = SessionEngine::new(PassthroughDecomp::new(), 20, 3, 100);
    let srv = ControlServer::new(path.clone(), eng);
    thread::spawn(move || {
        let _ = srv.run();
    });
    // give server a moment
    thread::sleep(Duration::from_millis(100));

    let mut c = UnixStream::connect(&path).expect("connect control");
    let _ = c.set_read_timeout(Some(Duration::from_millis(500)));
    // send a line into the buffer
    writeln!(
        c,
        "{}",
        serde_json::json!({"cmd":"send","data":"hello\n"}).to_string()
    )
    .unwrap();
    let mut resp = String::new();
    let mut br = std::io::BufReader::new(c);
    let _ = br.read_line(&mut resp).unwrap_or(0);
    // response includes at least one Ok
    assert!(resp.contains("\"event\":\"Ok\""));

    // connect anew and request buffer
    let mut d = UnixStream::connect(&path).unwrap();
    let _ = d.set_read_timeout(Some(Duration::from_millis(500)));
    writeln!(d, "{}", serde_json::json!({"cmd":"get_buffer"}).to_string()).unwrap();
    let mut buf = String::new();
    let mut br2 = std::io::BufReader::new(d);
    let _ = br2.read_line(&mut buf).unwrap_or(0);
    assert!(buf.contains("hello"));
}
