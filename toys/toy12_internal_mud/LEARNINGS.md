# LEARNINGS — Internal MUD for Testing & Offline Play

## Goals (What We Need to Learn)

### Primary Questions
1. **Interface Pattern**: How does an internal MUD feed the Session pipeline?
   - Fake socket with loopback?
   - Memory pipe/channel?
   - Direct feed to Session::feed()?
   - What's simplest for both testing and production?

2. **Telnet Protocol**: Should internal MUD emit real telnet sequences?
   - IAC commands (WILL/DO/DONT/WONT)?
   - GA/EOR for prompts?
   - MCCP compression negotiation?
   - Or bypass telnet layer entirely?

3. **ANSI Output**: What level of ANSI support?
   - Just SGR color codes (ESC[31m)?
   - Full attribute support (bold, underline)?
   - Can we test edge cases (fragmentation, reset)?

4. **State Management**: How to structure game state?
   - Room graph (nodes + edges)?
   - Item/inventory system?
   - Player state (location, inventory, flags)?
   - Serializable for save/load?

5. **Command Parser**: What's the minimal parser?
   - Simple verb-noun (go north, take sword)?
   - Aliases (n → go north)?
   - Error handling (unknown commands)?

6. **Testing Integration**: How to drive e2e tests?
   - Programmatic command injection?
   - Deterministic outcomes (no RNG)?
   - Observable state for assertions?
   - Can we run headless mode with internal MUD?

### Secondary Questions
7. **Feature Scope**: What's MVP vs nice-to-have?
   - MVP: 3-5 rooms, basic navigation, simple items
   - Nice: combat, NPCs, quests, persistence

8. **Production Integration**: How to wire into okros?
   - CLI flag (--offline, --demo)?
   - Feature gate (#[cfg(feature = "offline-mud")])?
   - Separate binary or integrated?

9. **Script Integration**: Can we implement MUD in Perl/Python?
   - Would dogfood plugin system
   - More flexible game content
   - But adds complexity to testing

## Decisions (To Be Made)

- [ ] Interface mechanism: fake socket vs memory pipe vs direct feed
- [ ] Telnet layer: full protocol vs bypass
- [ ] Game complexity: Zork-like adventure vs minimal test harness
- [ ] Implementation language: Rust vs Perl/Python plugin
- [ ] Integration approach: feature flag vs CLI mode vs separate tool

## Hypotheses (To Test)

1. **Fake Socket Hypothesis**: Using a socketpair() for loopback will let us reuse all existing Session pipeline code without modification
2. **Telnet Bypass Hypothesis**: We can skip telnet negotiation and just emit ANSI text, feeding Session after telnet parsing
3. **Minimal State Hypothesis**: A simple HashMap<RoomId, Room> + player state is sufficient for testing
4. **Script Plugin Hypothesis**: Implementing the MUD as a Python/Perl script will be more flexible than Rust

## Experiments to Run

1. Build minimal room graph (3 rooms, bidirectional navigation)
2. Test fake socket integration with Session pipeline
3. Emit ANSI color codes and verify scrollback rendering
4. Implement basic command parser (go, look, take, inventory, quit)
5. Drive via headless mode control server (JSON commands)
6. Measure: Can we write e2e tests with zero external dependencies?

## Success Criteria

- [x] Can navigate between rooms using Session pipeline
- [x] ANSI colors render correctly in scrollback
- [x] Can run deterministic e2e test (command sequence → expected output)
- [ ] Works in headless mode (control server sends commands, reads output) - NOT TESTED YET
- [x] Zero external dependencies (no real MUD server needed)
- [x] Pattern extracted for production integration

## Anti-Goals

- Not building a full MUD engine (that's a different project)
- Not replacing real MUD testing (still need that for network/protocol validation)
- Not a user-facing game (just testing infrastructure with offline bonus)

---

## Findings (Phases 1-3 Complete)

### Decision 1: Interface Pattern → **Direct Feed (WINNER)**

**Answer**: Session already has `pub fn feed(&mut self, chunk: &[u8])` - no fake socket needed!

**What we tried**:
- Considered socketpair() for loopback (Option A from PLAN.md)
- Discovered Session.feed() is already public and perfect for our use case

**What worked**:
```rust
let mut session = Session::new(PassthroughDecomp::new(), width, height, lines);
let output = mud.execute(command);  // Returns ANSI-colored String
session.feed(output.as_bytes());    // Feed directly to pipeline
```

**Pattern for production**:
- No special integration code needed
- Just call `session.feed(mud_output.as_bytes())`
- Simplest possible approach

### Decision 2: Telnet Protocol → **Bypass (As Expected)**

**Answer**: Skip telnet negotiation, just emit raw ANSI text.

**Reasoning**:
- Internal MUD doesn't need telnet handshakes (WILL/DO/WONT)
- Session pipeline handles raw ANSI text fine
- Telnet layer is pass-through for non-IAC bytes
- No prompt negotiation needed (GA/EOR not required)

**Pattern for production**:
```rust
// Just emit ANSI, no IAC commands:
format!("\x1b[32m{}\x1b[0m\n", room.name)  // Green room name
```

### Decision 3: Game State → **Minimal HashMap Works**

**Answer**: Simple HashMap<RoomId, Room> + Player is sufficient.

**What we built**:
- 5 rooms in HashMap
- Player: location + inventory (Vec<ItemId>)
- Items in HashMap by ID
- ~300 lines total for complete game

**Performance**: Instant (<1ms per command)
**Memory**: <1KB game state

**Pattern for production**:
- Keep game simple (this is testing infrastructure, not a game engine)
- Static room/item data (const or lazy_static)
- Mutable player state only

### Decision 4: ANSI Output → **Works Perfectly**

**Answer**: ANSI SGR codes work great through Session pipeline.

**Colors tested**:
- ESC[32m (green) - Room names ✅
- ESC[36m (cyan) - Exits ✅
- ESC[33m (yellow) - Items ✅
- ESC[31m (red) - Errors ✅
- ESC[0m (reset) - Works ✅

**Pattern for production**:
```rust
fn format_error(msg: &str) -> String {
    format!("\x1b[31m{}\x1b[0m\n", msg)
}

fn format_room(room: &Room) -> String {
    format!(
        "\x1b[32m{}\x1b[0m\n{}\n\x1b[36mExits: {}\x1b[0m\n",
        room.name, room.description, exits
    )
}
```

### Decision 5: Testing Integration → **Deterministic E2E Works**

**Answer**: Can write fully automated tests with zero external dependencies.

**Test pattern**:
```rust
#[test]
fn test_deterministic_sequence() {
    let mut world = MiniWorld::new();
    let mut session = Session::new(PassthroughDecomp::new(), 80, 20, 200);

    // Run deterministic command sequence
    session.feed(world.look().as_bytes());
    session.feed(world.take("sword").as_bytes());
    session.feed(world.go(Direction::North).as_bytes());

    // Assert scrollback contains expected text
    let text = extract_scrollback_text(&session);
    assert!(text.contains("Dense Forest"));
}
```

**Key insight**: Same commands always produce same scrollback (no RNG, no async).

### Hypothesis Results

1. **Fake Socket Hypothesis**: ❌ **REJECTED** - Not needed, direct feed is simpler
2. **Telnet Bypass Hypothesis**: ✅ **CONFIRMED** - Works perfectly without telnet
3. **Minimal State Hypothesis**: ✅ **CONFIRMED** - HashMap is sufficient
4. **Script Plugin Hypothesis**: ⏸️ **NOT TESTED** - Rust implementation was so easy, didn't need scripts

### Production Integration Recommendations

**For okros MVP**:

1. **Feature Flag**: Add `--features offline-mud` to Cargo.toml
2. **Location**: Create `src/offline_mud/` module
3. **CLI Integration**: Add `--offline` or `--demo` flag
4. **Game Content**: Copy toy12 data structures (rooms, items, parser)
5. **Wiring**: In main.rs, create World + Session, drive via normal input loop

**Code changes needed**:
```rust
// In src/main.rs
#[cfg(feature = "offline-mud")]
if args.offline {
    let mut world = offline_mud::World::new();
    let output = world.execute(parse(&input)?);
    session.feed(output.as_bytes());
}
```

**Estimated effort**: 2-3 hours to port toy → production

### E2E Testing Pattern (Production)

**Use internal MUD for automated e2e tests**:

```rust
// tests/e2e_with_internal_mud.rs
#[test]
fn test_full_pipeline_with_plugins() {
    let mut session = create_session_with_python_plugin();
    let mut world = InternalMud::new();

    // Run command sequence
    for cmd in test_sequence {
        session.feed(world.execute(cmd).as_bytes());
    }

    // Verify plugin hooks ran
    assert!(session.plugin_output.contains("sys/postoutput called"));
}
```

**Benefits**:
- No external MUD server needed
- Deterministic (same commands → same output)
- Fast (instant, no network latency)
- Can test plugin integration
- Can test ANSI/telnet/MCCP edge cases by controlling MUD output

### Open Questions (Not Yet Answered)

1. **Headless Mode Integration**: Can we drive internal MUD via control server?
   - Status: Not tested yet
   - Next step: Create test that sends JSON commands via Unix socket

2. **DNS Names**: Should offline MUD support `#open localhost 4000`?
   - Current: Only works with real MUD servers
   - Offline MUD: Just use `--offline` flag, no connection needed

3. **Save/Load**: Should game state persist?
   - Answer: No - it's for testing, not playing
   - Reset on each run is fine (deterministic tests)
