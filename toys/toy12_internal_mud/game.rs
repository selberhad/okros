use std::collections::HashMap;

pub type RoomId = &'static str;
pub type ItemId = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Direction {
    pub fn parse(s: &str) -> Option<Direction> {
        match s.to_lowercase().as_str() {
            "north" | "n" => Some(Direction::North),
            "south" | "s" => Some(Direction::South),
            "east" | "e" => Some(Direction::East),
            "west" | "w" => Some(Direction::West),
            "up" | "u" => Some(Direction::Up),
            "down" | "d" => Some(Direction::Down),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct Room {
    pub id: RoomId,
    pub name: &'static str,
    pub description: &'static str,
    pub exits: HashMap<Direction, RoomId>,
    pub items: Vec<ItemId>,
}

#[derive(Debug)]
pub struct Player {
    pub location: RoomId,
    pub inventory: Vec<ItemId>,
    pub max_inventory: usize,
}

pub struct World {
    pub rooms: HashMap<RoomId, Room>,
    pub items: HashMap<ItemId, Item>,
    pub player: Player,
}

impl World {
    pub fn new() -> Self {
        // Create items
        let mut items = HashMap::new();
        items.insert(
            "sword",
            Item {
                id: "sword",
                name: "rusty sword",
                description: "An old rusty sword, but still sharp.",
            },
        );
        items.insert(
            "torch",
            Item {
                id: "torch",
                name: "torch",
                description: "A burning torch that illuminates the darkness.",
            },
        );
        items.insert(
            "key",
            Item {
                id: "key",
                name: "iron key",
                description: "A heavy iron key with strange markings.",
            },
        );

        // Create rooms
        let mut rooms = HashMap::new();

        // Forest
        let mut forest_exits = HashMap::new();
        forest_exits.insert(Direction::South, "clearing");
        rooms.insert(
            "forest",
            Room {
                id: "forest",
                name: "Dense Forest",
                description: "You are in a dense forest. Tall trees block most of the sunlight.",
                exits: forest_exits,
                items: vec![],
            },
        );

        // Clearing (starting room)
        let mut clearing_exits = HashMap::new();
        clearing_exits.insert(Direction::North, "forest");
        clearing_exits.insert(Direction::East, "cave");
        clearing_exits.insert(Direction::South, "stream");
        rooms.insert(
            "clearing",
            Room {
                id: "clearing",
                name: "Forest Clearing",
                description: "You are in a forest clearing. Sunlight streams through the canopy above.",
                exits: clearing_exits,
                items: vec!["sword"], // sword starts here
            },
        );

        // Cave
        let mut cave_exits = HashMap::new();
        cave_exits.insert(Direction::West, "clearing");
        rooms.insert(
            "cave",
            Room {
                id: "cave",
                name: "Dark Cave",
                description: "You are in a dark cave. You can barely see anything.",
                exits: cave_exits,
                items: vec!["torch"], // torch starts here
            },
        );

        // Stream
        let mut stream_exits = HashMap::new();
        stream_exits.insert(Direction::North, "clearing");
        stream_exits.insert(Direction::South, "village");
        rooms.insert(
            "stream",
            Room {
                id: "stream",
                name: "Mountain Stream",
                description: "You are standing by a crystal clear mountain stream.",
                exits: stream_exits,
                items: vec![],
            },
        );

        // Village
        let mut village_exits = HashMap::new();
        village_exits.insert(Direction::North, "stream");
        rooms.insert(
            "village",
            Room {
                id: "village",
                name: "Abandoned Village",
                description: "You are in an abandoned village. The houses are empty and decaying.",
                exits: village_exits,
                items: vec!["key"], // key starts here
            },
        );

        let player = Player {
            location: "clearing",
            inventory: vec![],
            max_inventory: 5,
        };

        World {
            rooms,
            items,
            player,
        }
    }

    pub fn current_room(&self) -> &Room {
        self.rooms.get(self.player.location).expect("Player in invalid room")
    }

    pub fn get_item(&self, item_id: ItemId) -> Option<&Item> {
        self.items.get(item_id)
    }

    pub fn item_in_room(&self, item_id: ItemId) -> bool {
        self.current_room().items.contains(&item_id)
    }

    pub fn item_in_inventory(&self, item_id: ItemId) -> bool {
        self.player.inventory.contains(&item_id)
    }

    pub fn move_item_to_inventory(&mut self, item_id: ItemId) -> Result<(), String> {
        if !self.item_in_room(item_id) {
            return Err("You don't see that here.".to_string());
        }
        if self.player.inventory.len() >= self.player.max_inventory {
            return Err("Your inventory is full.".to_string());
        }

        // Remove from room
        let room = self.rooms.get_mut(self.player.location).unwrap();
        room.items.retain(|&id| id != item_id);

        // Add to inventory
        self.player.inventory.push(item_id);

        Ok(())
    }

    pub fn move_item_to_room(&mut self, item_id: ItemId) -> Result<(), String> {
        if !self.item_in_inventory(item_id) {
            return Err("You don't have that.".to_string());
        }

        // Remove from inventory
        self.player.inventory.retain(|&id| id != item_id);

        // Add to room
        let room = self.rooms.get_mut(self.player.location).unwrap();
        room.items.push(item_id);

        Ok(())
    }

    pub fn move_player(&mut self, direction: Direction) -> Result<RoomId, String> {
        let current = self.current_room();
        if let Some(&next_room) = current.exits.get(&direction) {
            self.player.location = next_room;
            Ok(next_room)
        } else {
            Err("You can't go that way.".to_string())
        }
    }

    pub fn execute(&mut self, cmd: crate::parser::Command) -> String {
        use crate::parser::Command;

        match cmd {
            Command::Go(dir) => match self.move_player(dir) {
                Ok(_) => self.format_look(),
                Err(e) => format_error(&e),
            },
            Command::Look => self.format_look(),
            Command::Take(item_name) => {
                // Find item by name (match against item.name field)
                let item_id = self
                    .current_room()
                    .items
                    .iter()
                    .find(|&&id| {
                        self.items
                            .get(id)
                            .map(|item| item.name == item_name)
                            .unwrap_or(false)
                    })
                    .copied();

                match item_id {
                    Some(id) => match self.move_item_to_inventory(id) {
                        Ok(()) => {
                            let name = self.items.get(id).unwrap().name;
                            format!("You take the {}.\n", name)
                        }
                        Err(e) => format_error(&e),
                    },
                    None => format_error("You don't see that here."),
                }
            }
            Command::Drop(item_name) => {
                let item_id = self
                    .player
                    .inventory
                    .iter()
                    .find(|&&id| {
                        self.items
                            .get(id)
                            .map(|item| item.name == item_name)
                            .unwrap_or(false)
                    })
                    .copied();

                match item_id {
                    Some(id) => match self.move_item_to_room(id) {
                        Ok(()) => {
                            let name = self.items.get(id).unwrap().name;
                            format!("You drop the {}.\n", name)
                        }
                        Err(e) => format_error(&e),
                    },
                    None => format_error("You don't have that."),
                }
            }
            Command::Inventory => self.format_inventory(),
            Command::Help => format_help(),
            Command::Quit => "\x1b[33mGoodbye!\x1b[0m\n".to_string(),
        }
    }

    fn format_look(&self) -> String {
        let room = self.current_room();
        let mut output = String::new();

        // Room name and description (green)
        output.push_str(&format!(
            "\x1b[32m{}\x1b[0m\n{}\n",
            room.name, room.description
        ));

        // Exits (cyan)
        if !room.exits.is_empty() {
            let exit_list: Vec<String> = room
                .exits
                .keys()
                .map(|d| format!("{:?}", d).to_lowercase())
                .collect();
            output.push_str(&format!(
                "\x1b[36mExits: {}\x1b[0m\n",
                exit_list.join(", ")
            ));
        } else {
            output.push_str("\x1b[36mNo obvious exits.\x1b[0m\n");
        }

        // Items in room (yellow)
        if !room.items.is_empty() {
            let item_list: Vec<String> = room
                .items
                .iter()
                .filter_map(|&id| self.items.get(id).map(|item| item.name.to_string()))
                .collect();
            output.push_str(&format!(
                "\x1b[33mItems: {}\x1b[0m\n",
                item_list.join(", ")
            ));
        }

        output
    }

    fn format_inventory(&self) -> String {
        if self.player.inventory.is_empty() {
            "You are carrying nothing.\n".to_string()
        } else {
            let item_list: Vec<String> = self
                .player
                .inventory
                .iter()
                .filter_map(|&id| self.items.get(id).map(|item| item.name.to_string()))
                .collect();
            format!("You are carrying: {}\n", item_list.join(", "))
        }
    }
}

fn format_error(msg: &str) -> String {
    format!("\x1b[31m{}\x1b[0m\n", msg)
}

fn format_help() -> String {
    let mut help = String::new();
    help.push_str("\x1b[36mAvailable commands:\x1b[0m\n");
    help.push_str("  go <direction>   - Move (north, south, east, west, up, down)\n");
    help.push_str("  n, s, e, w, u, d - Direction shortcuts\n");
    help.push_str("  look (l)         - Look around\n");
    help.push_str("  take <item>      - Pick up an item\n");
    help.push_str("  drop <item>      - Drop an item\n");
    help.push_str("  inventory (i)    - Show your inventory\n");
    help.push_str("  help (?)         - Show this help\n");
    help.push_str("  quit (q)         - Quit the game\n");
    help
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = World::new();
        assert_eq!(world.player.location, "clearing");
        assert_eq!(world.rooms.len(), 5);
        assert_eq!(world.items.len(), 3);
    }

    #[test]
    fn test_navigation() {
        let mut world = World::new();
        assert_eq!(world.player.location, "clearing");

        // Go north to forest
        world.move_player(Direction::North).unwrap();
        assert_eq!(world.player.location, "forest");

        // Go back south to clearing
        world.move_player(Direction::South).unwrap();
        assert_eq!(world.player.location, "clearing");

        // Try invalid direction
        assert!(world.move_player(Direction::West).is_err());
    }

    #[test]
    fn test_item_management() {
        let mut world = World::new();

        // Sword should be in clearing
        assert!(world.item_in_room("sword"));
        assert!(!world.item_in_inventory("sword"));

        // Take sword
        world.move_item_to_inventory("sword").unwrap();
        assert!(!world.item_in_room("sword"));
        assert!(world.item_in_inventory("sword"));
        assert_eq!(world.player.inventory.len(), 1);

        // Drop sword
        world.move_item_to_room("sword").unwrap();
        assert!(world.item_in_room("sword"));
        assert!(!world.item_in_inventory("sword"));
        assert_eq!(world.player.inventory.len(), 0);
    }

    #[test]
    fn test_inventory_full() {
        let mut world = World::new();
        world.player.max_inventory = 1;

        // Take sword
        world.move_item_to_inventory("sword").unwrap();

        // Go to cave and try to take torch (should fail)
        world.move_player(Direction::East).unwrap();
        let result = world.move_item_to_inventory("torch");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Your inventory is full.");
    }
}
