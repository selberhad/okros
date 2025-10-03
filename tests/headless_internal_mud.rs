// Test: Internal MUD driven via headless mode control server
// Goal: Validate full automation loop - JSON commands → MUD → scrollback → JSON response

use okros::mccp::PassthroughDecomp;
use okros::session::Session;
use serde_json::json;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
use std::time::Duration;

// Minimal MUD (copied from internal_mud_integration.rs)
type RoomId = &'static str;
type ItemId = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    North,
    South,
    East,
}

#[derive(Debug, Clone)]
struct Room {
    name: &'static str,
    description: &'static str,
    exits: HashMap<Direction, RoomId>,
    items: Vec<ItemId>,
}

struct MiniWorld {
    rooms: HashMap<RoomId, Room>,
    player_location: RoomId,
}

impl MiniWorld {
    fn new() -> Self {
        let mut rooms = HashMap::new();

        // Clearing
        let mut clearing_exits = HashMap::new();
        clearing_exits.insert(Direction::North, "forest");
        rooms.insert(
            "clearing",
            Room {
                name: "Forest Clearing",
                description: "You are in a forest clearing.",
                exits: clearing_exits,
                items: vec!["sword"],
            },
        );

        // Forest
        let mut forest_exits = HashMap::new();
        forest_exits.insert(Direction::South, "clearing");
        rooms.insert(
            "forest",
            Room {
                name: "Dense Forest",
                description: "You are in a dense forest.",
                exits: forest_exits,
                items: vec![],
            },
        );

        MiniWorld {
            rooms,
            player_location: "clearing",
        }
    }

    fn execute(&mut self, input: &str) -> String {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return "".to_string();
        }

        match parts[0] {
            "look" | "l" => self.look(),
            "go" | "n" | "s" => {
                let dir = if parts[0] == "n" || (parts.len() > 1 && parts[1] == "north") {
                    Direction::North
                } else if parts[0] == "s" || (parts.len() > 1 && parts[1] == "south") {
                    Direction::South
                } else {
                    return "\x1b[31mGo where?\x1b[0m\n".to_string();
                };
                self.go(dir)
            }
            _ => "\x1b[31mUnknown command\x1b[0m\n".to_string(),
        }
    }

    fn look(&self) -> String {
        let room = self.rooms.get(self.player_location).unwrap();
        format!(
            "\x1b[32m{}\x1b[0m\n{}\n\x1b[36mExits: north, south\x1b[0m\n",
            room.name, room.description
        )
    }

    fn go(&mut self, dir: Direction) -> String {
        let room = self.rooms.get(self.player_location).unwrap();
        if let Some(&next) = room.exits.get(&dir) {
            self.player_location = next;
            self.look()
        } else {
            "\x1b[31mYou can't go that way.\x1b[0m\n".to_string()
        }
    }
}

// Minimal control server for internal MUD
struct MudControlServer {
    world: MiniWorld,
    session: Session<PassthroughDecomp>,
}

impl MudControlServer {
    fn new() -> Self {
        Self {
            world: MiniWorld::new(),
            session: Session::new(PassthroughDecomp::new(), 80, 24, 200),
        }
    }

    fn handle_command(&mut self, cmd_json: &str) -> String {
        let cmd: serde_json::Value = match serde_json::from_str(cmd_json) {
            Ok(v) => v,
            Err(_) => return json!({"event":"Error","message":"Invalid JSON"}).to_string(),
        };

        let cmd_type = cmd["cmd"].as_str().unwrap_or("");

        match cmd_type {
            "send" => {
                // Get command from data field
                let data = cmd["data"].as_str().unwrap_or("");

                // Execute in MUD
                let output = self.world.execute(data);

                // Feed to Session pipeline
                self.session.feed(output.as_bytes());

                json!({"event":"Ok"}).to_string()
            }
            "get_buffer" => {
                // Extract scrollback as lines
                let viewport = self.session.scrollback.viewport_slice();
                let text: String = viewport.iter().map(|&a| (a & 0xFF) as u8 as char).collect();

                // Split into lines and clean up
                let lines: Vec<String> = text
                    .lines()
                    .map(|s| s.trim_end().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                json!({"event":"Buffer","lines":lines}).to_string()
            }
            "status" => json!({"event":"Status","attached":false}).to_string(),
            _ => json!({"event":"Error","message":"Unknown command"}).to_string(),
        }
    }
}

#[test]
fn test_headless_mud_via_control_server() {
    let socket_path = format!("/tmp/okros_test_mud_{}.sock", std::process::id());

    // Clean up any existing socket
    let _ = std::fs::remove_file(&socket_path);

    let socket_path_clone = socket_path.clone();

    // Start control server in background thread
    let server_thread = thread::spawn(move || {
        let listener = UnixListener::bind(&socket_path_clone).expect("bind socket");
        listener.set_nonblocking(false).expect("set blocking");

        let mut server = MudControlServer::new();

        // Accept one connection
        if let Ok((stream, _)) = listener.accept() {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = stream;

            // Read commands and respond
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                let response = server.handle_command(line.trim());
                writeln!(writer, "{}", response).unwrap();
                line.clear();
            }
        }
    });

    // Give server time to start
    thread::sleep(Duration::from_millis(100));

    // Connect as client
    let mut client = UnixStream::connect(&socket_path).expect("connect to server");
    client.set_read_timeout(Some(Duration::from_secs(1))).ok();

    let mut reader = BufReader::new(client.try_clone().unwrap());

    // Test 1: Send "look" command
    writeln!(client, r#"{{"cmd":"send","data":"look\n"}}"#).unwrap();
    let mut response = String::new();
    reader.read_line(&mut response).unwrap();
    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Ok");

    // Test 2: Get buffer - should contain room description
    writeln!(client, r#"{{"cmd":"get_buffer"}}"#).unwrap();
    response.clear();
    reader.read_line(&mut response).unwrap();
    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(resp["event"], "Buffer");

    let lines = resp["lines"].as_array().unwrap();
    let all_text = lines
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        all_text.contains("Forest Clearing"),
        "Should show room name"
    );
    assert!(all_text.contains("clearing"), "Should show description");

    // Test 3: Navigate north
    writeln!(client, r#"{{"cmd":"send","data":"go north\n"}}"#).unwrap();
    response.clear();
    reader.read_line(&mut response).unwrap();

    // Get buffer again
    writeln!(client, r#"{{"cmd":"get_buffer"}}"#).unwrap();
    response.clear();
    reader.read_line(&mut response).unwrap();
    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();

    let lines = resp["lines"].as_array().unwrap();
    let all_text = lines
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    assert!(all_text.contains("Dense Forest"), "Should be in forest now");

    // Clean up - close connection to unblock server
    drop(reader);
    drop(client);

    // Give server time to detect EOF and exit
    thread::sleep(Duration::from_millis(50));

    let _ = std::fs::remove_file(&socket_path);

    // Don't wait for server thread - it will exit when connection closes
}

#[test]
fn test_deterministic_command_sequence_via_json() {
    let socket_path = format!("/tmp/okros_test_sequence_{}.sock", std::process::id());
    let _ = std::fs::remove_file(&socket_path);

    let socket_path_clone = socket_path.clone();

    let server_thread = thread::spawn(move || {
        let listener = UnixListener::bind(&socket_path_clone).expect("bind");
        let mut server = MudControlServer::new();

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

    // Run deterministic sequence
    let commands = vec!["look\n", "go north\n", "go south\n"];

    for cmd in commands {
        let json_cmd = json!({"cmd":"send","data":cmd}).to_string();
        writeln!(client, "{}", json_cmd).unwrap();

        let mut response = String::new();
        reader.read_line(&mut response).unwrap();
    }

    // Verify final state
    writeln!(client, r#"{{"cmd":"get_buffer"}}"#).unwrap();
    let mut response = String::new();
    reader.read_line(&mut response).unwrap();
    let resp: serde_json::Value = serde_json::from_str(&response).unwrap();

    let lines = resp["lines"].as_array().unwrap();
    let all_text = lines
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    // Should be back in clearing after north then south
    assert!(
        all_text.contains("Forest Clearing"),
        "Should be back in clearing"
    );

    // Clean up - close connection to unblock server
    drop(reader);
    drop(client);

    // Give server time to detect EOF and exit
    thread::sleep(Duration::from_millis(50));

    let _ = std::fs::remove_file(&socket_path);

    // Don't wait for server thread - it will exit when connection closes
}
