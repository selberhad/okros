# PLAN — Internal MUD Implementation

## Strategy

Build minimal text adventure in Rust, validate integration with Session pipeline, extract patterns for production.

**Approach**: Start with pure in-memory game engine, then add Session integration layer.

## Phase 1: Core Game Engine (Standalone)

### Step 1: Data Structures
**Goal**: Define game state representation

**Tasks**:
- [ ] `Room` struct (id, name, description, exits: HashMap<Dir, RoomId>, items: Vec<Item>)
- [ ] `Item` struct (id, name, description)
- [ ] `Direction` enum (North, South, East, West, Up, Down)
- [ ] `Player` struct (location: RoomId, inventory: Vec<Item>)
- [ ] `World` struct (rooms: HashMap<RoomId, Room>, player: Player)

**Test**: Create 5-room world, verify connectivity

**Files**: `game.rs` (~50 lines)

---

### Step 2: Command Parser
**Goal**: Parse text commands into actions

**Tasks**:
- [ ] `Command` enum (Go(Dir), Look, Take(String), Drop(String), Inventory, Help, Quit)
- [ ] `parse(input: &str) -> Result<Command, String>` function
- [ ] Handle aliases (n→North, i→Inventory, etc.)
- [ ] Error messages for malformed input

**Test**: Parse valid commands, reject invalid ones

**Files**: `parser.rs` (~40 lines)

---

### Step 3: Game Logic
**Goal**: Execute commands and update state

**Tasks**:
- [ ] `World::execute(&mut self, cmd: Command) -> String` method
- [ ] Navigation logic (check exits, move player, return description)
- [ ] Item manipulation (take/drop with validation)
- [ ] Look command (format room + exits + items)
- [ ] Inventory display

**Test**: Run command sequences, verify state changes

**Files**: `game.rs` additional ~60 lines

**Checkpoint**: Can play game via Rust function calls

---

## Phase 2: ANSI Formatting

### Step 4: Color Output
**Goal**: Emit ANSI escape sequences for colors

**Tasks**:
- [ ] Format room descriptions with green ESC[32m
- [ ] Format items with yellow ESC[33m
- [ ] Format exits with cyan ESC[36m
- [ ] Format errors with red ESC[31m
- [ ] Reset codes ESC[0m at appropriate points

**Test**: Verify ANSI sequences in output strings

**Files**: `game.rs` or `ansi.rs` (~20 lines helpers)

---

## Phase 3: Session Integration

### Step 5: Interface Layer (DECISION POINT)
**Goal**: Connect MUD engine to Session pipeline

**Option A: Fake Socket** (RECOMMENDED)
```rust
use std::os::unix::net::UnixStream;

let (client, server) = UnixStream::pair()?;
// Session reads from client
// MUD writes to server
let session = Session::new(..., client);
let mud = MudServer::new(server);
```

**Option B: Direct Feed**
```rust
let mut session = Session::new(...);
let mut mud = MudEngine::new();

loop {
    let output = mud.tick();
    session.feed(output.as_bytes());
}
```

**Tasks**:
- [ ] Implement chosen integration mechanism
- [ ] Wire MUD output → Session input
- [ ] Wire user input → MUD commands
- [ ] Test full pipeline: command → MUD → ANSI → Session → Scrollback

**Test**: Send "go north", verify scrollback contains new room description

**Files**: `integration.rs` or `main.rs` (~40 lines)

---

### Step 6: Prompt Handling
**Goal**: Emit proper prompts for interactive feel

**Tasks**:
- [ ] Send "> " prompt after each command output
- [ ] Optional: Use telnet GA or EOR for prompt detection
- [ ] Test prompt rendering in scrollback

**Files**: `game.rs` modifications (~10 lines)

---

## Phase 4: Testing Harness

### Step 7: Programmatic Testing
**Goal**: Drive MUD via code for e2e tests

**Tasks**:
- [ ] Expose command injection: `mud.send_command("go north")`
- [ ] Expose output capture: `mud.get_output() -> String`
- [ ] Write deterministic test: command sequence → expected output
- [ ] Test via Session pipeline (not just raw MUD)

**Test**:
```rust
#[test]
fn test_navigation_through_session() {
    let (session, mud) = create_integrated_mud();
    mud.send("go north\n");
    assert!(session.scrollback.contains("You are in a dense forest"));
}
```

**Files**: `tests/mud_integration.rs` (~30 lines)

---

### Step 8: Headless Mode Test
**Goal**: Verify works with control server

**Tasks**:
- [ ] Start internal MUD in headless mode
- [ ] Connect via Unix socket control server
- [ ] Send commands via JSON: `{"cmd":"send","data":"go north\n"}`
- [ ] Read output via `{"cmd":"get_buffer"}`
- [ ] Verify deterministic output

**Test**: Full e2e with no human interaction

**Files**: `tests/headless_mud.rs` (~40 lines)

---

## Phase 5: Production Integration (Post-Toy)

### Step 9: Extract Patterns
**Goal**: Document findings for production use

**Tasks**:
- [ ] Update LEARNINGS.md with what worked
- [ ] Document integration approach (fake socket vs direct)
- [ ] Note ANSI formatting patterns
- [ ] Capture testing patterns (command injection, output capture)
- [ ] Recommend feature flag approach

**Files**: `LEARNINGS.md` (findings section)

---

### Step 10: Production Plan
**Goal**: Roadmap for integrating into okros

**Tasks**:
- [ ] Decide: feature flag (`--features offline-mud`) or CLI arg (`--offline`)?
- [ ] Decide: Rust implementation or Perl/Python script?
- [ ] Plan: Where to put code (src/offline_mud/ or separate crate)?
- [ ] Plan: How to expose in main.rs (CLI parsing, mode selection)
- [ ] Estimate: Effort to port toy → production (~1-2 days)

**Files**: `LEARNINGS.md` (production recommendations)

---

## Testing Strategy

**Unit Tests** (per phase):
- Phase 1: Game logic (room navigation, item management)
- Phase 2: ANSI formatting (color codes present and correct)
- Phase 3: Session integration (pipeline works end-to-end)
- Phase 4: Deterministic e2e (same commands → same output)

**Integration Tests**:
- Full pipeline: command → MUD → Session → Scrollback
- Headless mode: control server → internal MUD → JSON response

**Golden Tests**:
- Capture expected output for command sequences
- Regression test: ensure output doesn't change

## Success Criteria

- [x] SPEC.md written (behavioral contract)
- [x] PLAN.md written (this document)
- [ ] Phase 1-2 complete: Standalone MUD works with ANSI output
- [ ] Phase 3 complete: Integrated with Session pipeline
- [ ] Phase 4 complete: Automated e2e tests pass
- [ ] Phase 5 complete: LEARNINGS.md documents patterns for production

## Risks & Mitigations

**Risk**: Session integration is harder than expected
- Mitigation: Start with Option B (direct feed), fall back to fake socket

**Risk**: ANSI codes don't render correctly through pipeline
- Mitigation: Test with existing Session/ANSI tests first

**Risk**: Deterministic testing breaks due to async/threading
- Mitigation: Run MUD synchronously in test mode, no background threads

**Risk**: Scope creep (building too complex a game)
- Mitigation: Stick to SPEC (5 rooms, 3 items, 6 commands max)

## Timeline Estimate

- Phase 1-2: 1-2 hours (core game + ANSI)
- Phase 3: 1-2 hours (Session integration)
- Phase 4: 1 hour (testing harness)
- Phase 5: 30 minutes (documentation)

**Total**: 3-5 hours for complete toy validation

## Next Steps

1. Start with Step 1 (data structures)
2. Build incrementally, test each phase
3. Defer integration until core game works standalone
4. Extract patterns immediately after validation
5. Propose production integration approach based on findings
