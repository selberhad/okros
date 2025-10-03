# SPEC — Internal MUD for okros

## Purpose

A minimal single-player text adventure embedded in okros for:
1. **End-to-end testing**: Deterministic game server with no external dependencies
2. **Offline demo**: Learn the client when disconnected from real MUDs
3. **Protocol testing**: Generate controlled telnet/ANSI/MCCP sequences for edge case validation

## Behavioral Contract

### Input Format

**Command Interface**:
```
go <direction>     # Navigate (north, south, east, west, up, down)
look [target]      # Examine room or item
take <item>        # Pick up item
drop <item>        # Drop item
inventory          # Show carried items
quit               # Exit game
help               # Show commands
```

**Aliases**:
```
n, s, e, w, u, d   → go north, go south, ...
l                  → look
i                  → inventory
q                  → quit
```

### Output Format

**Text Protocol**: ANSI-colored text with newlines
```
\x1b[32mYou are in a forest clearing.\x1b[0m
Exits: north, south
Items: rusty sword

>
```

**Color Conventions**:
- Room descriptions: Green (ESC[32m)
- Items: Yellow (ESC[33m)
- Exits: Cyan (ESC[36m)
- Errors: Red (ESC[31m)
- Prompts: White (ESC[37m or default)

### Game State

**Room Graph** (minimal 5-room demo):
```
     [Forest]
         |
    [Clearing] --- [Cave]
         |
     [Stream]
         |
      [Village]
```

**Rooms**:
- Each room has: name, description, exits (direction → room_id), items
- Initial room: Clearing

**Items** (minimal set):
- rusty sword (in Clearing)
- torch (in Cave)
- key (in Village)

**Player State**:
- Current room
- Inventory (list of items)
- Max inventory: 5 items

### Operations

#### Navigation
```
Input:  "go north"
Effect: Move player to connected room if exit exists
Output: New room description OR "You can't go that way."
```

#### Look
```
Input:  "look"
Effect: None (query only)
Output: Room description + exits + items + inventory
```

#### Take Item
```
Input:  "take sword"
Effect: Move item from room to inventory (if present, inventory not full)
Output: "You take the rusty sword." OR error
Errors:
  - "You don't see that here."
  - "Your inventory is full."
```

#### Drop Item
```
Input:  "drop sword"
Effect: Move item from inventory to current room
Output: "You drop the rusty sword." OR "You don't have that."
```

#### Inventory
```
Input:  "inventory"
Effect: None (query only)
Output: "You are carrying: rusty sword, torch" OR "You are carrying nothing."
```

### Invariants

1. **Conservation**: Items exist in exactly one place (room OR inventory)
2. **Connectivity**: All rooms reachable from starting room
3. **Determinism**: Same command sequence always produces same output
4. **No RNG**: No random elements (for testing predictability)

### Error Semantics

**Unknown Command**:
```
Input:  "dance"
Output: "I don't understand that command. Type 'help' for commands."
```

**Parse Errors**:
```
Input:  "go"
Output: "Go where? (north, south, east, west, up, down)"
```

**Invalid Direction**:
```
Input:  "go north" (when no north exit)
Output: "You can't go that way."
```

### Integration Contract

**Interface to Session Pipeline**:

Option A (Fake Socket):
```rust
// Create socketpair
let (client_sock, server_sock) = socketpair()?;
// Session reads from client_sock
// MUD writes to server_sock
mud.write(b"\x1b[32mWelcome!\x1b[0m\n> ")?;
```

Option B (Direct Feed):
```rust
// Session exposes feed() method
session.feed(mud.tick())?;
```

**Command Injection** (for testing):
```rust
mud.execute("go north")?;
mud.execute("take sword")?;
assert!(mud.get_output().contains("You take the rusty sword"));
```

### Test Scenarios

#### Scenario 1: Basic Navigation
```
Input:  "look"
Output: "You are in a forest clearing..."
Input:  "go north"
Output: "You are in a dense forest..."
Input:  "go south"
Output: "You are in a forest clearing..."
```

#### Scenario 2: Item Management
```
Input:  "take sword"
Output: "You take the rusty sword."
Input:  "inventory"
Output: "You are carrying: rusty sword"
Input:  "drop sword"
Output: "You drop the rusty sword."
Input:  "look"
Output: "... Items: rusty sword"
```

#### Scenario 3: Error Handling
```
Input:  "go nowhere"
Output: "I don't understand that direction."
Input:  "take spaceship"
Output: "You don't see that here."
Input:  "go west" (no west exit)
Output: "You can't go that way."
```

#### Scenario 4: Edge Cases
```
# Inventory full
Input:  "take item" (when inventory at max)
Output: "Your inventory is full."

# Item already taken
Input:  "take sword"
Output: "You take the rusty sword."
Input:  "take sword"
Output: "You don't see that here."
```

### Success Criteria

1. **Functional**: All commands work as specified
2. **ANSI**: Colors render correctly through Session pipeline
3. **Testable**: Can run deterministic command sequence via code
4. **Offline**: Zero external dependencies (no network, no files)
5. **Minimal**: <200 lines of Rust for core engine

### Out of Scope

- Combat system
- NPCs / dialogue
- Quests / objectives
- Save/load persistence
- Multi-room item dependencies (keys/doors)
- Score / achievements
- Time-based events
- Complex parser (just verb-noun)

## Non-Functional Requirements

- **Performance**: Instant response (<1ms per command)
- **Memory**: <1KB game state
- **Determinism**: Same input sequence → same output (critical for testing)
- **Isolation**: No side effects (no files, no network, no global state mutations outside game)
