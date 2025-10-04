// In-process integration tests for control server
// These tests run the control server in a background thread (same process)
// This allows llvm-cov to track coverage, unlike subprocess spawning

use okros::control::{default_socket_path, ControlServer};
use okros::engine::SessionEngine;
use okros::mccp::PassthroughDecomp;
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;

fn start_test_server(instance: &str) -> std::path::PathBuf {
    let socket_path = default_socket_path(instance);
    let _ = std::fs::remove_file(&socket_path);

    let engine = SessionEngine::new(PassthroughDecomp::new(), 80, 24, 1000);
    let server = ControlServer::new(socket_path.clone(), engine);

    thread::spawn(move || {
        let _ = server.run();
    });

    // Give server time to start
    thread::sleep(Duration::from_millis(100));

    socket_path
}

#[test]
fn test_inprocess_status_command() {
    let instance = format!("inproc_status_{}", std::process::id());
    let socket_path = start_test_server(&instance);

    // Connect and test status command
    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");
    stream.set_read_timeout(Some(Duration::from_secs(1))).ok();

    // Send status command
    writeln!(stream, r#"{{"cmd":"status"}}"#).unwrap();

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).ok();

    // Verify response
    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Status");
    assert!(resp["attached"].is_boolean());

    println!("✓ In-process status command works");

    // Cleanup
    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_inprocess_attach_detach() {
    let instance = format!("inproc_attach_{}", std::process::id());
    let socket_path = start_test_server(&instance);

    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");
    stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Test attach command
    writeln!(stream, r#"{{"cmd":"attach"}}"#).unwrap();
    let mut response = String::new();
    reader.read_line(&mut response).ok();
    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Ok");

    // Test detach command
    response.clear();
    writeln!(stream, r#"{{"cmd":"detach"}}"#).unwrap();
    reader.read_line(&mut response).ok();
    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Ok");

    println!("✓ In-process attach/detach commands work");

    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_inprocess_send_and_get_buffer() {
    let instance = format!("inproc_buffer_{}", std::process::id());
    let socket_path = start_test_server(&instance);

    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");
    stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Send data to session
    writeln!(stream, r#"{{"cmd":"send","data":"test line"}}"#).unwrap();
    let mut response = String::new();
    reader.read_line(&mut response).ok();

    // Get buffer
    response.clear();
    writeln!(stream, r#"{{"cmd":"get_buffer"}}"#).unwrap();
    reader.read_line(&mut response).ok();

    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Buffer");
    assert!(resp["lines"].is_array());

    println!("✓ In-process send and get_buffer commands work");

    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_inprocess_peek_command() {
    let instance = format!("inproc_peek_{}", std::process::id());
    let socket_path = start_test_server(&instance);

    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");
    stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Send data first
    writeln!(stream, r#"{{"cmd":"send","data":"test line"}}"#).unwrap();
    let mut response = String::new();
    reader.read_line(&mut response).ok();

    // Peek at buffer (without consuming)
    response.clear();
    writeln!(stream, r#"{{"cmd":"peek","lines":10}}"#).unwrap();
    reader.read_line(&mut response).ok();

    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Buffer");
    assert!(resp["lines"].is_array());

    println!("✓ In-process peek command works");

    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_inprocess_hex_command() {
    let instance = format!("inproc_hex_{}", std::process::id());
    let socket_path = start_test_server(&instance);

    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");
    stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Send data first
    writeln!(stream, r#"{{"cmd":"send","data":"test"}}"#).unwrap();
    let mut response = String::new();
    reader.read_line(&mut response).ok();

    // Get hex view
    response.clear();
    writeln!(stream, r#"{{"cmd":"hex","lines":5}}"#).unwrap();
    reader.read_line(&mut response).ok();

    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Hex");
    assert!(resp["lines"].is_array());

    println!("✓ In-process hex command works");

    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_inprocess_bad_json() {
    let instance = format!("inproc_badjson_{}", std::process::id());
    let socket_path = start_test_server(&instance);

    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");
    stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Send invalid JSON
    writeln!(stream, "not json at all").unwrap();

    let mut response = String::new();
    reader.read_line(&mut response).ok();

    if !response.is_empty() {
        let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(resp["event"], "Error");
        assert!(resp["message"].as_str().unwrap().contains("bad json"));
        println!("✓ In-process bad JSON error handling works");
    }

    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_inprocess_unknown_command() {
    let instance = format!("inproc_unknown_{}", std::process::id());
    let socket_path = start_test_server(&instance);

    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");
    stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Send unknown command
    writeln!(stream, r#"{{"cmd":"bogus_command"}}"#).unwrap();

    let mut response = String::new();
    reader.read_line(&mut response).ok();

    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Error");
    assert!(resp["message"].as_str().unwrap().contains("unknown cmd"));

    println!("✓ In-process unknown command error handling works");

    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_inprocess_send_missing_data() {
    let instance = format!("inproc_sendnodata_{}", std::process::id());
    let socket_path = start_test_server(&instance);

    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");
    stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Send command without data field
    writeln!(stream, r#"{{"cmd":"send"}}"#).unwrap();

    let mut response = String::new();
    reader.read_line(&mut response).ok();

    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Error");
    assert!(resp["message"].as_str().unwrap().contains("missing data"));

    println!("✓ In-process send missing data error handling works");

    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_inprocess_sock_send_not_connected() {
    let instance = format!("inproc_socksend_{}", std::process::id());
    let socket_path = start_test_server(&instance);

    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");
    stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Try sock_send without being connected to a MUD
    writeln!(stream, r#"{{"cmd":"sock_send","data":"test"}}"#).unwrap();

    let mut response = String::new();
    reader.read_line(&mut response).ok();

    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Error");
    assert!(resp["message"].as_str().unwrap().contains("not connected"));

    println!("✓ In-process sock_send not connected error handling works");

    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_inprocess_sock_send_missing_data() {
    let instance = format!("inproc_socksendnodata_{}", std::process::id());
    let socket_path = start_test_server(&instance);

    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");
    stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Try sock_send without data
    writeln!(stream, r#"{{"cmd":"sock_send"}}"#).unwrap();

    let mut response = String::new();
    reader.read_line(&mut response).ok();

    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Error");
    assert!(resp["message"].as_str().unwrap().contains("missing data"));

    println!("✓ In-process sock_send missing data error handling works");

    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_inprocess_connect_missing_data() {
    let instance = format!("inproc_connectnodata_{}", std::process::id());
    let socket_path = start_test_server(&instance);

    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");
    stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Try connect without data (host:port)
    writeln!(stream, r#"{{"cmd":"connect"}}"#).unwrap();

    let mut response = String::new();
    reader.read_line(&mut response).ok();

    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Error");
    assert!(resp["message"].as_str().unwrap().contains("missing data"));

    println!("✓ In-process connect missing data error handling works");

    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_inprocess_connect_bad_address() {
    let instance = format!("inproc_connectbad_{}", std::process::id());
    let socket_path = start_test_server(&instance);

    let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");
    stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Try connect with malformed address
    writeln!(
        stream,
        r#"{{"cmd":"connect","data":"not-a-valid-address"}}"#
    )
    .unwrap();

    let mut response = String::new();
    reader.read_line(&mut response).ok();

    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Error");
    // Should get resolve error
    assert!(resp["message"].is_string());

    println!("✓ In-process connect bad address error handling works");

    std::fs::remove_file(&socket_path).ok();
}
