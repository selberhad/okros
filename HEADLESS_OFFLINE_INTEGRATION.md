# Headless + Offline MUD Integration Plan

**Goal**: Wire the offline MUD into headless mode for complete LLM agent testing loop

**Status**: Currently separate modes - need to combine them

---

## Current State

### What Works ✅

**Offline Mode** (`cargo run -- --offline`):
- Full internal MUD (5 rooms, 3 items, ANSI colors)
- Interactive TTY UI (requires real terminal)
- Direct World + Session pipeline
- File: `src/main.rs::run_offline_mode()`

**Headless Mode** (`cargo run -- --headless --instance test`):
- SessionEngine + ControlServer
- Unix socket at `/tmp/mcl/test.sock`
- JSON Lines protocol working
- Requires external MUD connection (socket I/O)
- Files: `src/engine.rs`, `src/control.rs`

**Tests Validate Both**:
- `tests/offline_mud_playthrough.rs` - Direct World tests (5 tests)
- `tests/offline_mud_headless.rs` - OfflineMudServer wrapper (2 tests)
- Both patterns work independently!

### What's Missing ❌

**No `--headless --offline` combined mode**:
```bash
# This doesn't exist yet (should work):
cargo run -- --headless --offline --instance demo

# Then control via socket:
echo '{"cmd":"send","data":"look"}' | nc -U /tmp/mcl/demo.sock
echo '{"cmd":"get_buffer"}' | nc -U /tmp/mcl/demo.sock
```

**Problem**: Headless mode expects network socket, offline mode expects TTY

---

## Integration Plan

### Step 1: Add CLI Flag Combination

**File**: `src/main.rs`

**Current** (lines 14-43):
```rust
if args[1] == "--headless" { ... }
else if args[1] == "--attach" { ... }
else if args[1] == "--offline" { run_offline_mode(); return; }
```

**Needed**:
```rust
if args[1] == "--headless" {
    let offline = args.contains(&"--offline".to_string());
    if offline {
        run_headless_offline_mode(&args);
    } else {
        // existing headless network mode
    }
}
```

### Step 2: Create HeadlessOfflineMode

**Pattern**: Copy from `tests/offline_mud_headless.rs::OfflineMudServer`

**New function in** `src/main.rs`:
```rust
fn run_headless_offline_mode(args: &[String]) {
    use okros::offline_mud::{World, parse};
    use okros::session::Session;
    use okros::control::ControlServer;

    let inst = args.get(3).unwrap_or(&"default".to_string()).clone();
    let path = default_socket_path(&inst);

    // Create server with offline MUD backend
    struct OfflineMudEngine {
        world: World,
        session: Session<PassthroughDecomp>,
    }

    // Wire into ControlServer
    let server = OfflineControlServer::new(path, OfflineMudEngine::new());
    eprintln!("Headless offline MUD; control socket at {}", path.display());
    server.run();
}
```

### Step 3: Implement OfflineControlServer

**Two Options**:

**Option A: Extend ControlServer** (cleaner, more work)
- Make ControlServer generic over backend
- `ControlServer<T: MudBackend>`
- Offline MUD implements `MudBackend` trait
- Network socket implements `MudBackend` trait

**Option B: Standalone OfflineControlServer** (faster, MVP)
- Copy `src/control.rs::ControlServer` logic
- Replace socket I/O with `world.execute(parse(cmd))`
- Simpler for MVP, can refactor later

**Recommend**: Option B for MVP

### Step 4: Wire World → Session Pipeline

**Key insight** from `tests/offline_mud_headless.rs`:
```rust
// Parse MUD command
match parse(data.trim()) {
    Ok(mud_cmd) => {
        // Execute in World → get ANSI output
        let output = self.world.execute(mud_cmd);

        // Feed to Session pipeline (ANSI → scrollback)
        self.session.feed(output.as_bytes());

        json!({"event":"Ok"}).to_string()
    }
    Err(e) => {
        // Parse error → show in session
        let err_msg = format!("\x1b[31m{}\x1b[0m\n", e);
        self.session.feed(err_msg.as_bytes());
        json!({"event":"Ok"}).to_string()
    }
}
```

This pattern already works in tests - just needs to be in production!

### Step 5: Test End-to-End

**Manual test**:
```bash
# Start headless offline MUD
cargo run -- --headless --offline --instance demo &

# Wait for socket
sleep 1

# Play via control protocol
echo '{"cmd":"get_buffer"}' | nc -U /tmp/mcl/demo.sock
echo '{"cmd":"send","data":"look"}' | nc -U /tmp/mcl/demo.sock
echo '{"cmd":"get_buffer"}' | nc -U /tmp/mcl/demo.sock
echo '{"cmd":"send","data":"take rusty sword"}' | nc -U /tmp/mcl/demo.sock
echo '{"cmd":"get_buffer"}' | nc -U /tmp/mcl/demo.sock
echo '{"cmd":"send","data":"inventory"}' | nc -U /tmp/mcl/demo.sock
echo '{"cmd":"get_buffer"}' | nc -U /tmp/mcl/demo.sock
```

**Expected output**:
- Initial buffer shows Forest Clearing
- After `take rusty sword`: "You take the rusty sword"
- After `inventory`: Shows sword in inventory

---

## Implementation Estimate

**Time**: 1-2 hours

**Files to modify**:
1. `src/main.rs` - Add CLI flag handling (10 lines)
2. `src/main.rs` or new `src/offline_control.rs` - OfflineControlServer (~150 lines, copy from test)

**Files to reference**:
- `tests/offline_mud_headless.rs` - Working implementation pattern
- `src/control.rs` - Network ControlServer (template to copy)
- `src/offline_mud/` - World + parser (already working)

---

## Success Criteria

✅ `cargo run -- --headless --offline --instance test` starts successfully
✅ Socket created at `/tmp/mcl/test.sock`
✅ `{"cmd":"get_buffer"}` returns initial room description
✅ `{"cmd":"send","data":"look"}` executes MUD command
✅ `{"cmd":"get_buffer"}` shows updated buffer with command result
✅ Full playthrough via JSON control (all 5 rooms, all 3 items)
✅ No crashes, proper error handling for bad commands

---

## Future Enhancements (Post-MVP)

1. **Attach mode for offline**: `cargo run -- --attach --offline test`
   - Shows offline MUD in TTY UI via control socket
   - Demonstrates detach/reattach pattern

2. **Streaming mode**: `{"cmd":"stream"}` for live updates
   - Real-time output as game events happen
   - Useful for watching bot play

3. **Game state queries**: `{"cmd":"status"}` returns location, inventory
   - Structured data alongside raw text
   - LLM can parse both formats

4. **Save/load game**: Persist offline MUD state
   - `{"cmd":"save_game","file":"save1.json"}`
   - `{"cmd":"load_game","file":"save1.json"}`

5. **Multiple offline instances**: Different save files
   - Each instance = separate game state
   - Test multiple bot strategies in parallel

---

## Notes

- Tests in `tests/offline_mud_headless.rs` already prove this works
- Just need to move pattern from test → production
- No new concepts - pure code organization
- Main challenge: avoiding code duplication between network/offline control servers
  - MVP: Accept duplication
  - Post-MVP: Refactor with trait abstraction

**Next session**: Start with this doc, implement Option B (standalone OfflineControlServer)
