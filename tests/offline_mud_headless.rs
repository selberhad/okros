// Test: Full offline MUD (src/offline_mud) driven via headless control server
// Demonstrates: LLM-friendly architecture - no TTY needed, just JSON Lines protocol

use okros::offline_mud::{World, parse};
use okros::session::Session;
use okros::mccp::PassthroughDecomp;
use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{BufRead, BufReader, Write};
use std::thread;
use std::time::Duration;
use serde_json::json;

/// Control server for offline MUD - headless mode
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
                let text: String = viewport
                    .iter()
                    .map(|&a| (a & 0xFF) as u8 as char)
                    .collect();

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
                }).to_string()
            }
            _ => json!({"event":"Error","message":"Unknown command"}).to_string(),
        }
    }
}

#[test]
fn test_offline_mud_via_headless_control_socket() {
    let socket_path = format!("/tmp/okros_offline_test_{}.sock", std::process::id());
    let _ = std::fs::remove_file(&socket_path);

    let socket_path_clone = socket_path.clone();

    // Start control server in background
    thread::spawn(move || {
        let listener = UnixListener::bind(&socket_path_clone).expect("bind socket");
        let mut server = OfflineMudServer::new();

        if let Ok((stream, _)) = listener.accept() {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = stream;
            let mut line = String::new();

            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                let response = server.handle_command(line.trim());
                writeln!(writer, "{}", response).unwrap();
                line.clear();
            }
        }
    });

    thread::sleep(Duration::from_millis(100));

    // Connect as client (LLM agent pattern)
    let mut client = UnixStream::connect(&socket_path).expect("connect to server");
    client.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(client.try_clone().unwrap());

    // Helper to send command and read response
    let send_cmd = |client: &mut UnixStream, reader: &mut BufReader<UnixStream>, cmd: &str| -> serde_json::Value {
        let json_cmd = json!({"cmd":"send","data":cmd}).to_string();
        writeln!(client, "{}", json_cmd).unwrap();
        let mut response = String::new();
        reader.read_line(&mut response).unwrap();
        serde_json::from_str(&response).unwrap()
    };

    let get_buffer = |client: &mut UnixStream, reader: &mut BufReader<UnixStream>| -> Vec<String> {
        writeln!(client, r#"{{"cmd":"get_buffer"}}"#).unwrap();
        let mut response = String::new();
        reader.read_line(&mut response).unwrap();
        let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
        resp["lines"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    };

    // Test 1: Initial state shows Forest Clearing
    let lines = get_buffer(&mut client, &mut reader);
    let text = lines.join(" ");
    assert!(text.contains("Forest Clearing"), "Should show starting room");
    assert!(text.contains("rusty sword"), "Should show sword item");

    // Test 2: Take the sword
    send_cmd(&mut client, &mut reader, "take rusty sword\n");
    let lines = get_buffer(&mut client, &mut reader);
    let text = lines.join(" ");
    assert!(text.contains("You take the rusty sword"), "Should confirm taking sword");

    // Test 3: Check inventory
    send_cmd(&mut client, &mut reader, "inventory\n");
    let lines = get_buffer(&mut client, &mut reader);
    let text = lines.join(" ");
    assert!(text.contains("rusty sword"), "Should show sword in inventory");

    // Test 4: Navigate north to forest
    send_cmd(&mut client, &mut reader, "north\n");
    let lines = get_buffer(&mut client, &mut reader);
    let text = lines.join(" ");
    assert!(text.contains("Dense Forest"), "Should be in forest");

    // Test 5: Navigate back south
    send_cmd(&mut client, &mut reader, "s\n");
    let lines = get_buffer(&mut client, &mut reader);
    let text = lines.join(" ");
    assert!(text.contains("Forest Clearing"), "Should be back in clearing");

    // Test 6: Go to cave (east)
    send_cmd(&mut client, &mut reader, "e\n");
    let lines = get_buffer(&mut client, &mut reader);
    let text = lines.join(" ");
    assert!(text.contains("Dark Cave"), "Should be in cave");

    // Test 7: Take torch
    send_cmd(&mut client, &mut reader, "take torch\n");
    let lines = get_buffer(&mut client, &mut reader);
    let text = lines.join(" ");
    assert!(text.contains("You take the torch"), "Should get torch");

    // Test 8: Error handling - invalid command
    send_cmd(&mut client, &mut reader, "dance\n");
    let lines = get_buffer(&mut client, &mut reader);
    let text = lines.join(" ");
    assert!(text.contains("don't understand"), "Should show error for bad command");

    // Clean up
    drop(reader);
    drop(client);
    thread::sleep(Duration::from_millis(50));
    let _ = std::fs::remove_file(&socket_path);
}

#[test]
fn test_llm_agent_playthrough_pattern() {
    // Demonstrates: LLM agent pattern - read buffer, decide action, send command
    let socket_path = format!("/tmp/okros_llm_agent_{}.sock", std::process::id());
    let _ = std::fs::remove_file(&socket_path);

    let socket_path_clone = socket_path.clone();

    thread::spawn(move || {
        let listener = UnixListener::bind(&socket_path_clone).expect("bind");
        let mut server = OfflineMudServer::new();

        if let Ok((stream, _)) = listener.accept() {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = stream;
            let mut line = String::new();

            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                let response = server.handle_command(line.trim());
                writeln!(writer, "{}", response).unwrap();
                line.clear();
            }
        }
    });

    thread::sleep(Duration::from_millis(100));

    let mut client = UnixStream::connect(&socket_path).expect("connect");
    client.set_read_timeout(Some(Duration::from_secs(1))).ok();
    let mut reader = BufReader::new(client.try_clone().unwrap());

    // LLM Agent Loop: Read → Decide → Act
    let llm_actions: Vec<(&str, Box<dyn Fn(&str) -> bool>)> = vec![
        ("look", Box::new(|text: &str| text.contains("Forest Clearing"))),
        ("take rusty sword", Box::new(|text: &str| text.contains("You take"))),
        ("inventory", Box::new(|text: &str| text.contains("rusty sword"))),
        ("go east", Box::new(|text: &str| text.contains("Dark Cave"))),
        ("take torch", Box::new(|text: &str| text.contains("You take the torch"))),
        ("go west", Box::new(|text: &str| text.contains("Forest Clearing"))),
        ("go south", Box::new(|text: &str| text.contains("Mountain Stream"))),
        ("go south", Box::new(|text: &str| text.contains("Abandoned Village"))),
        ("take iron key", Box::new(|text: &str| text.contains("You take the iron key"))),
        ("inventory", Box::new(|text: &str| text.contains("rusty sword") && text.contains("torch") && text.contains("iron key"))),
    ];

    for (action, validator) in &llm_actions {
        // Send action
        let json_cmd = json!({"cmd":"send","data":format!("{}\n", action)}).to_string();
        writeln!(client, "{}", json_cmd).unwrap();
        let mut response = String::new();
        reader.read_line(&mut response).unwrap();

        // Read buffer (what LLM would see)
        writeln!(client, r#"{{"cmd":"get_buffer"}}"#).unwrap();
        response.clear();
        reader.read_line(&mut response).unwrap();
        let resp: serde_json::Value = serde_json::from_str(&response).unwrap();

        let lines: Vec<String> = resp["lines"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let text = lines.join(" ");

        // Validate (LLM would parse this text to decide next action)
        assert!(validator(&text), "Action '{}' failed validation. Buffer: {}", action, text);
    }

    // Final check: Collected all 3 items
    writeln!(client, r#"{{"cmd":"status"}}"#).unwrap();
    let mut response = String::new();
    reader.read_line(&mut response).unwrap();
    let status: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(status["inventory_count"], 3, "Should have all 3 items");
    assert_eq!(status["location"], "village", "Should be in village");

    // Clean up
    drop(reader);
    drop(client);
    thread::sleep(Duration::from_millis(50));
    let _ = std::fs::remove_file(&socket_path);
}
