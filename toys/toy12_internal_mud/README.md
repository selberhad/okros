# Toy 12 - Internal MUD for Testing & Offline Play

## Purpose

A minimal single-player text adventure embedded in okros for:
1. **End-to-end testing** - Deterministic game server with zero external dependencies
2. **Offline demo** - Learn the client when disconnected from real MUDs
3. **Protocol validation** - Generate controlled telnet/ANSI/MCCP sequences for edge case testing

## What We Built

**Minimal fantasy MUD**:
- 5 rooms (forest, clearing, cave, stream, village)
- 3 items (rusty sword, torch, iron key)
- 8 commands (go, look, take, drop, inventory, help, quit + direction aliases)
- Full ANSI color output (green rooms, cyan exits, yellow items, red errors)

**Integration with okros**:
- Feeds output through Session pipeline: MUD → ANSI → Telnet → MCCP → Scrollback
- 4 integration tests in `tests/internal_mud_integration.rs`
- Deterministic output (same commands → same scrollback)

## Quick Start

### Standalone Interactive Mode

```bash
cargo run
```

Output:
```
=== okros Internal MUD Demo ===
Type 'help' for commands, 'quit' to exit.

Forest Clearing
You are in a forest clearing. Sunlight streams through the canopy above.
Exits: east, north, south
Items: rusty sword

> n
Dense Forest
You are in a dense forest. Tall trees block most of the sunlight.
Exits: south

> s
Forest Clearing
...
```

### Run Tests

```bash
# Unit tests (parser, game logic)
cargo test

# Integration tests (full pipeline)
cd ../.. && cargo test --test internal_mud_integration
```

## Key Findings

### ✅ Decisions Made

1. **Interface Pattern**: Direct feed to Session.feed() - no fake socket needed
2. **Telnet Protocol**: Bypass entirely - just emit raw ANSI text
3. **Game State**: Simple HashMap<RoomId, Room> + Player is sufficient
4. **ANSI Colors**: Work perfectly through Session pipeline
5. **Testing**: Fully automated e2e tests with zero external dependencies
6. **Headless Mode**: ✅ **VALIDATED** - Internal MUD works via control server!

### 📊 Results

- **Tests**: 14 unit tests + 6 integration tests passing
  - 4 tests: Session pipeline integration
  - 2 tests: Headless mode via control server
- **Performance**: <1ms per command
- **Memory**: <1KB game state
- **LOC**: ~300 lines for complete game

### 🎯 Production Integration

**Ready to port** - estimated 2-3 hours effort:

1. Add `--features offline-mud` to Cargo.toml
2. Create `src/offline_mud/` module
3. Add `--offline` CLI flag
4. Wire World + Session in main.rs

**Use for**:
- Automated e2e tests (no external MUD needed)
- Plugin integration tests
- ANSI/telnet/MCCP edge case validation
- Offline demo/tutorial mode

## Files

- `game.rs` - World, Room, Item, Player structs + game logic
- `parser.rs` - Command parser with aliases
- `main.rs` - Interactive REPL for manual testing
- `integration.rs` - MudSession wrapper (reference/doc)
- `SPEC.md` - Behavioral contract
- `PLAN.md` - Implementation phases
- `LEARNINGS.md` - Findings and patterns for production

## Example Test Pattern

```rust
#[test]
fn test_deterministic_sequence() {
    let mut world = MiniWorld::new();
    let mut session = Session::new(PassthroughDecomp::new(), 80, 20, 200);

    // Run command sequence
    session.feed(world.look().as_bytes());
    session.feed(world.take("sword").as_bytes());
    session.feed(world.go(Direction::North).as_bytes());

    // Verify scrollback
    let text = extract_text(&session.scrollback);
    assert!(text.contains("Dense Forest"));
}
```

## Next Steps

**Optional enhancements** (not needed for MVP):
- [ ] Drive via headless mode control server
- [ ] Add more rooms/items for richer demo
- [ ] Implement as Perl/Python plugin (dogfood plugin system)
- [ ] Add save/load state

**Current status**: ✅ **COMPLETE** - Ready for production integration

---

**Remember**: This is a transport layer testing tool, not a game engine. Keep it simple!
