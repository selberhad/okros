// Integration test: Internal MUD → Session pipeline
// Validates that MUD output flows through full okros pipeline:
// MUD → ANSI → Telnet → MCCP → Scrollback

use okros::session::Session;
use okros::mccp::PassthroughDecomp;
use std::collections::HashMap;

// Minimal MUD for testing (copied from toy12)
type RoomId = &'static str;
type ItemId = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
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
    player_inventory: Vec<ItemId>,
}

impl MiniWorld {
    fn new() -> Self {
        let mut rooms = HashMap::new();

        // Clearing
        let mut clearing_exits = HashMap::new();
        clearing_exits.insert(Direction::North, "forest");
        clearing_exits.insert(Direction::East, "cave");
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

        // Cave
        let mut cave_exits = HashMap::new();
        cave_exits.insert(Direction::West, "clearing");
        rooms.insert(
            "cave",
            Room {
                name: "Dark Cave",
                description: "You are in a dark cave.",
                exits: cave_exits,
                items: vec!["torch"],
            },
        );

        MiniWorld {
            rooms,
            player_location: "clearing",
            player_inventory: vec![],
        }
    }

    fn look(&self) -> String {
        let room = self.rooms.get(self.player_location).unwrap();
        let mut out = String::new();

        // Room name (green) + description
        out.push_str(&format!("\x1b[32m{}\x1b[0m\n", room.name));
        out.push_str(&format!("{}\n", room.description));

        // Exits (cyan)
        if !room.exits.is_empty() {
            let exits: Vec<&str> = room
                .exits
                .keys()
                .map(|d| match d {
                    Direction::North => "north",
                    Direction::South => "south",
                    Direction::East => "east",
                    Direction::West => "west",
                })
                .collect();
            out.push_str(&format!("\x1b[36mExits: {}\x1b[0m\n", exits.join(", ")));
        }

        // Items (yellow)
        if !room.items.is_empty() {
            out.push_str(&format!("\x1b[33mItems: {}\x1b[0m\n", room.items.join(", ")));
        }

        out
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

    fn take(&mut self, item: ItemId) -> String {
        let room = self.rooms.get_mut(self.player_location).unwrap();
        if let Some(pos) = room.items.iter().position(|&id| id == item) {
            room.items.remove(pos);
            self.player_inventory.push(item);
            format!("You take the {}.\n", item)
        } else {
            "\x1b[31mYou don't see that here.\x1b[0m\n".to_string()
        }
    }
}

#[test]
fn test_mud_output_through_session() {
    let mut world = MiniWorld::new();
    let mut session = Session::new(PassthroughDecomp::new(), 80, 10, 100);

    // Execute "look" and feed to session
    let output = world.look();
    session.feed(output.as_bytes());

    // Check scrollback contains room description
    let viewport = session.scrollback.viewport_slice();
    let text: String = viewport
        .iter()
        .map(|&a| (a & 0xFF) as u8 as char)
        .collect();

    assert!(text.contains("Forest Clearing"), "Should show room name");
    assert!(text.contains("clearing"), "Should show description");
    assert!(text.contains("sword"), "Should show item");
}

#[test]
fn test_mud_navigation_through_session() {
    let mut world = MiniWorld::new();
    let mut session = Session::new(PassthroughDecomp::new(), 80, 10, 100);

    // Go north
    let output = world.go(Direction::North);
    session.feed(output.as_bytes());

    let viewport = session.scrollback.viewport_slice();
    let text: String = viewport
        .iter()
        .map(|&a| (a & 0xFF) as u8 as char)
        .collect();

    assert!(text.contains("Dense Forest"), "Should be in forest");
}

#[test]
fn test_mud_item_management_through_session() {
    let mut world = MiniWorld::new();
    let mut session = Session::new(PassthroughDecomp::new(), 80, 10, 100);

    // Take sword
    let output = world.take("sword");
    session.feed(output.as_bytes());

    let viewport = session.scrollback.viewport_slice();
    let text: String = viewport
        .iter()
        .map(|&a| (a & 0xFF) as u8 as char)
        .collect();

    assert!(text.contains("You take the sword"), "Should confirm taking sword");
}

#[test]
fn test_deterministic_command_sequence() {
    let mut world = MiniWorld::new();
    let mut session = Session::new(PassthroughDecomp::new(), 80, 20, 200);

    // Run deterministic sequence
    session.feed(world.look().as_bytes());
    session.feed(world.take("sword").as_bytes());
    session.feed(world.go(Direction::North).as_bytes());
    session.feed(world.go(Direction::South).as_bytes());
    session.feed(world.go(Direction::East).as_bytes());

    let viewport = session.scrollback.viewport_slice();
    let text: String = viewport
        .iter()
        .map(|&a| (a & 0xFF) as u8 as char)
        .collect();

    // Should end in cave
    assert!(text.contains("Dark Cave"), "Should end in cave");
}
