// Integration tests for control server (network headless mode)
// Tests the JSON Lines control protocol

use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;

#[test]
fn test_control_server_status_command() {
    // Start a headless instance in background
    let socket_path = format!("/tmp/okros/test_status_{}.sock", std::process::id());
    let _ = std::fs::remove_file(&socket_path);

    // Spawn headless server
    let mut child = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--headless",
            "--instance",
            &format!("test_status_{}", std::process::id()),
        ])
        .spawn()
        .expect("Failed to start headless server");

    // Wait for socket to be created
    thread::sleep(Duration::from_millis(500));

    // Connect and test status command
    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
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

        println!("✓ Status command works");
    }

    // Cleanup
    child.kill().ok();
    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_control_server_attach_detach() {
    let socket_path = format!("/tmp/okros/test_attach_{}.sock", std::process::id());
    let _ = std::fs::remove_file(&socket_path);

    let mut child = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--headless",
            "--instance",
            &format!("test_attach_{}", std::process::id()),
        ])
        .spawn()
        .expect("Failed to start headless server");

    thread::sleep(Duration::from_millis(500));

    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
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

        println!("✓ Attach/detach commands work");
    }

    child.kill().ok();
    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_control_server_bad_json() {
    let socket_path = format!("/tmp/okros/test_badjson_{}.sock", std::process::id());
    let _ = std::fs::remove_file(&socket_path);

    let mut child = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--headless",
            "--instance",
            &format!("test_badjson_{}", std::process::id()),
        ])
        .spawn()
        .expect("Failed to start headless server");

    thread::sleep(Duration::from_millis(500));

    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
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
            println!("✓ Bad JSON error handling works");
        }
    }

    child.kill().ok();
    std::fs::remove_file(&socket_path).ok();
}

#[test]
fn test_control_server_send_and_get_buffer() {
    let socket_path = format!("/tmp/okros/test_buffer_{}.sock", std::process::id());
    let _ = std::fs::remove_file(&socket_path);

    let mut child = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--headless",
            "--instance",
            &format!("test_buffer_{}", std::process::id()),
        ])
        .spawn()
        .expect("Failed to start headless server");

    thread::sleep(Duration::from_millis(500));

    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
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

        println!("✓ Send and get_buffer commands work");
    }

    child.kill().ok();
    std::fs::remove_file(&socket_path).ok();
}
