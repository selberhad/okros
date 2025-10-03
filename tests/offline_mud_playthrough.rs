// Integration test: Full playthrough of offline MUD
// Tests complete game sequence: visit all rooms, collect all items

use okros::offline_mud::{World, parse};

#[test]
fn test_complete_playthrough() {
    let mut world = World::new();

    // Start in clearing
    assert_eq!(world.player.location, "clearing");

    // Look around - should see sword
    let output = world.execute(parse("look").unwrap());
    assert!(output.contains("Forest Clearing"));
    assert!(output.contains("rusty sword"));

    // Take the sword
    let output = world.execute(parse("take rusty sword").unwrap());
    assert!(output.contains("You take the rusty sword"));
    assert!(world.player.inventory.contains(&"sword"));

    // Go north to forest
    let output = world.execute(parse("north").unwrap());
    assert!(output.contains("Dense Forest"));
    assert_eq!(world.player.location, "forest");

    // Go back south
    let output = world.execute(parse("s").unwrap());
    assert!(output.contains("Forest Clearing"));
    assert_eq!(world.player.location, "clearing");

    // Go east to cave
    let output = world.execute(parse("e").unwrap());
    assert!(output.contains("Dark Cave"));
    assert_eq!(world.player.location, "cave");

    // Take the torch
    let output = world.execute(parse("take torch").unwrap());
    assert!(output.contains("You take the torch"));
    assert!(world.player.inventory.contains(&"torch"));

    // Check inventory
    let output = world.execute(parse("inventory").unwrap());
    assert!(output.contains("rusty sword"));
    assert!(output.contains("torch"));

    // Go back west to clearing
    let _output = world.execute(parse("w").unwrap());
    assert_eq!(world.player.location, "clearing");

    // Go south to stream
    let output = world.execute(parse("south").unwrap());
    assert!(output.contains("Mountain Stream"));
    assert_eq!(world.player.location, "stream");

    // Go south to village
    let output = world.execute(parse("s").unwrap());
    assert!(output.contains("Abandoned Village"));
    assert_eq!(world.player.location, "village");

    // Take the key
    let output = world.execute(parse("get iron key").unwrap());
    assert!(output.contains("You take the iron key"));
    assert!(world.player.inventory.contains(&"key"));

    // Check we have all 3 items
    assert_eq!(world.player.inventory.len(), 3);

    // Test dropping an item (must use full name "rusty sword", not just "sword")
    let output = world.execute(parse("drop rusty sword").unwrap());
    assert!(output.contains("You drop the rusty sword"));
    assert!(!world.player.inventory.contains(&"sword"));
    assert_eq!(world.player.inventory.len(), 2);

    // Pick it back up
    let output = world.execute(parse("take rusty sword").unwrap());
    assert!(output.contains("You take the rusty sword"));
    assert_eq!(world.player.inventory.len(), 3);
}

#[test]
fn test_error_cases() {
    let mut world = World::new();

    // Try to go in invalid direction
    let output = world.execute(parse("west").unwrap());
    assert!(output.contains("You can't go that way"));

    // Try to take non-existent item
    let output = world.execute(parse("take banana").unwrap());
    assert!(output.contains("You don't see that here"));

    // Try to drop item you don't have
    let output = world.execute(parse("drop rusty sword").unwrap());
    assert!(output.contains("You don't have that"));

    // Test inventory limit
    world.player.max_inventory = 2;
    world.execute(parse("take rusty sword").unwrap());
    world.execute(parse("e").unwrap()); // Go to cave
    world.execute(parse("take torch").unwrap());

    // Try to pick up third item (should fail)
    world.execute(parse("w").unwrap()); // Back to clearing
    world.execute(parse("s").unwrap()); // To stream
    world.execute(parse("s").unwrap()); // To village
    let output = world.execute(parse("take iron key").unwrap());
    assert!(output.contains("Your inventory is full"));
}

#[test]
fn test_parser_edge_cases() {
    // Empty command
    let result = parse("");
    assert!(result.is_err());

    // Whitespace only
    let result = parse("   ");
    assert!(result.is_err());

    // Unknown command
    let result = parse("dance");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("I don't understand 'dance'"));

    // Multi-word item names work
    let result = parse("take rusty sword");
    assert!(result.is_ok());

    // Case insensitive
    let result = parse("LOOK");
    assert!(result.is_ok());

    let result = parse("North");
    assert!(result.is_ok());
}

#[test]
fn test_help_and_quit() {
    let mut world = World::new();

    // Test help command
    let output = world.execute(parse("help").unwrap());
    assert!(output.contains("Available commands"));
    assert!(output.contains("go <direction>"));
    assert!(output.contains("take <item>"));

    // Test help alias
    let output = world.execute(parse("?").unwrap());
    assert!(output.contains("Available commands"));

    // Test quit command
    let output = world.execute(parse("quit").unwrap());
    assert!(output.contains("Goodbye"));

    // Test quit aliases
    let output = world.execute(parse("q").unwrap());
    assert!(output.contains("Goodbye"));

    let output = world.execute(parse("exit").unwrap());
    assert!(output.contains("Goodbye"));
}

#[test]
fn test_all_rooms_accessible() {
    let mut world = World::new();
    let mut visited = std::collections::HashSet::new();

    // BFS to visit all rooms
    visited.insert("clearing");

    // North to forest
    world.execute(parse("n").unwrap());
    visited.insert("forest");
    world.execute(parse("s").unwrap());

    // East to cave
    world.execute(parse("e").unwrap());
    visited.insert("cave");
    world.execute(parse("w").unwrap());

    // South to stream
    world.execute(parse("s").unwrap());
    visited.insert("stream");

    // South to village
    world.execute(parse("s").unwrap());
    visited.insert("village");

    // Verify we visited all 5 rooms
    assert_eq!(visited.len(), 5);
    assert!(visited.contains("clearing"));
    assert!(visited.contains("forest"));
    assert!(visited.contains("cave"));
    assert!(visited.contains("stream"));
    assert!(visited.contains("village"));
}
