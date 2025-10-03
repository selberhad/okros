use super::game::Direction;

#[derive(Debug, PartialEq)]
pub enum Command {
    Go(Direction),
    Look,
    Take(String),
    Drop(String),
    Inventory,
    Help,
    Quit,
}

pub fn parse(input: &str) -> Result<Command, String> {
    let trimmed = input.trim().to_lowercase();
    if trimmed.is_empty() {
        return Err("Type a command (or 'help' for help).".to_string());
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let verb = parts[0];

    match verb {
        // Navigation
        "go" => {
            if parts.len() < 2 {
                return Err("Go where? (north, south, east, west, up, down)".to_string());
            }
            Direction::parse(parts[1])
                .map(Command::Go)
                .ok_or_else(|| "I don't understand that direction.".to_string())
        }
        // Direction aliases
        "north" | "n" => Ok(Command::Go(Direction::North)),
        "south" | "s" => Ok(Command::Go(Direction::South)),
        "east" | "e" => Ok(Command::Go(Direction::East)),
        "west" | "w" => Ok(Command::Go(Direction::West)),
        "up" | "u" => Ok(Command::Go(Direction::Up)),
        "down" | "d" => Ok(Command::Go(Direction::Down)),

        // Observation
        "look" | "l" => Ok(Command::Look),

        // Item manipulation
        "take" | "get" => {
            if parts.len() < 2 {
                return Err("Take what?".to_string());
            }
            // Join remaining parts (handles multi-word items like "rusty sword")
            Ok(Command::Take(parts[1..].join(" ")))
        }
        "drop" => {
            if parts.len() < 2 {
                return Err("Drop what?".to_string());
            }
            Ok(Command::Drop(parts[1..].join(" ")))
        }

        // Inventory
        "inventory" | "inv" | "i" => Ok(Command::Inventory),

        // Meta
        "help" | "?" => Ok(Command::Help),
        "quit" | "q" | "exit" => Ok(Command::Quit),

        // Unknown
        _ => Err(format!(
            "I don't understand '{}'. Type 'help' for commands.",
            verb
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_go() {
        assert_eq!(parse("go north"), Ok(Command::Go(Direction::North)));
        assert_eq!(parse("go south"), Ok(Command::Go(Direction::South)));
        assert!(parse("go").is_err());
        assert!(parse("go nowhere").is_err());
    }

    #[test]
    fn test_parse_direction_aliases() {
        assert_eq!(parse("north"), Ok(Command::Go(Direction::North)));
        assert_eq!(parse("n"), Ok(Command::Go(Direction::North)));
        assert_eq!(parse("s"), Ok(Command::Go(Direction::South)));
        assert_eq!(parse("e"), Ok(Command::Go(Direction::East)));
        assert_eq!(parse("w"), Ok(Command::Go(Direction::West)));
        assert_eq!(parse("u"), Ok(Command::Go(Direction::Up)));
        assert_eq!(parse("d"), Ok(Command::Go(Direction::Down)));
    }

    #[test]
    fn test_parse_look() {
        assert_eq!(parse("look"), Ok(Command::Look));
        assert_eq!(parse("l"), Ok(Command::Look));
    }

    #[test]
    fn test_parse_take() {
        assert_eq!(parse("take sword"), Ok(Command::Take("sword".to_string())));
        assert_eq!(
            parse("take rusty sword"),
            Ok(Command::Take("rusty sword".to_string()))
        );
        assert_eq!(parse("get torch"), Ok(Command::Take("torch".to_string())));
        assert!(parse("take").is_err());
    }

    #[test]
    fn test_parse_drop() {
        assert_eq!(parse("drop sword"), Ok(Command::Drop("sword".to_string())));
        assert!(parse("drop").is_err());
    }

    #[test]
    fn test_parse_inventory() {
        assert_eq!(parse("inventory"), Ok(Command::Inventory));
        assert_eq!(parse("inv"), Ok(Command::Inventory));
        assert_eq!(parse("i"), Ok(Command::Inventory));
    }

    #[test]
    fn test_parse_meta() {
        assert_eq!(parse("help"), Ok(Command::Help));
        assert_eq!(parse("?"), Ok(Command::Help));
        assert_eq!(parse("quit"), Ok(Command::Quit));
        assert_eq!(parse("q"), Ok(Command::Quit));
        assert_eq!(parse("exit"), Ok(Command::Quit));
    }

    #[test]
    fn test_parse_unknown() {
        assert!(parse("dance").is_err());
        assert!(parse("fly to the moon").is_err());
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse("").is_err());
        assert!(parse("   ").is_err());
    }

    #[test]
    fn test_parse_case_insensitive() {
        assert_eq!(parse("NORTH"), Ok(Command::Go(Direction::North)));
        assert_eq!(parse("Look"), Ok(Command::Look));
        assert_eq!(parse("TAKE SWORD"), Ok(Command::Take("sword".to_string())));
    }
}
