# ORIENTATION — okros MUD Client

**Quick Start**: You're looking at a Rust port of MCL (MUD Client for Linux), ~99% complete (core implementation + validation done), built for headless operation and LLM agent integration.

## What Is This?

**okros** = Rust MUD client reviving MCL's design, optimized for automation
- **Primary use case**: Transport layer for Perl/Python bots and LLM agents
- **Philosophy**: Client handles I/O, scripts handle logic (automation via Perl/Python or built-in features)
- **Status**: Core implementation complete + validated (all tiers done, tested on Nodeka MUD); Alt-O hotkey integration remaining

## Current State (Oct 2025)

### ✅ What Works (Implementation Complete + Validated)
- **Network**: Socket, telnet, MCCP compression, ANSI parsing (full pipeline)
- **UI**: ncurses wrapper, screen diff renderer, widgets (status/output/input), scrollback, selection widget
- **Plugins**: Python (pyo3) and Perl (raw FFI) with feature gates, interpreter hooks
- **Headless Engine**: SessionEngine + control server (Unix socket, JSON Lines protocol)
- **Offline Mode**: Internal MUD for testing/demo (5 rooms, 3 items, ANSI colors, `--offline` flag)
- **Main Event Loop**: poll-based I/O on TTY + socket with 250ms timeout
- **CLI Args**: `--headless`, `--instance <name>`, `--attach <name>`, `--offline`, `--headless --offline` (combined)
- **# Commands**: `#quit`, `#open`, `#alias`, `#action`, `#subst`, `#macro` all functional
- **Automation**: ✅ Fully wired into I/O pipeline
  - Alias expansion (text with %N params) - wired into input pipeline
  - Triggers/replacements/gags with Perl/Python regex - wired into output pipeline
  - Keyboard macros - wired into key handling
  - MUD inheritance (child inherits parent's aliases/macros)
- **Config System**: Dual format parser (old + new MUD block syntax)
  - MUD definitions with hostname/port/commands/aliases/actions
  - MUD inheritance (child inherits parent features)
  - Automatic Offline MUD injection as entry #0
- **Connect Menu Infrastructure**: Selection widget, MUDSelection, config loading (Alt-O integration remaining)
- **Tests**: 134 total tests passing (126 unit + 8 integration) | extensive coverage
- **✅ MUD VALIDATED**: Full gameplay session on Nodeka (2025-10-03) - See `MUD_LEARNINGS.md`

### ⏸️ What Needs Completion (Minimal Remaining Work)
- **Alt-O Hotkey**: Connect menu UI integration (infrastructure complete, needs Screen/Window wiring)
- **Perl Bot Validation**: Test real automation scripts against headless mode (real-world integration)

### ❌ What's Deferred (By Design)
- Chat, borg, group features (not needed for MVP)
- Extended # commands beyond current set (scripts handle complex commands)
- DNS hostname resolution for interactive #open (headless mode has DNS via control server)

## Next Steps (Priority Order)

**MVP is VALIDATED and FEATURE-COMPLETE. Remaining: UI polish + real-world bot testing.**

### 1. Alt-O Hotkey Integration (Optional Polish)
**Goal**: Wire connect menu to Alt-O keypress

**What's Done**:
- ✅ Config file parser (old + new formats, inheritance)
- ✅ MUD list storage (MudList with find/insert/iter)
- ✅ Selection widget (base scrollable list with navigation)
- ✅ MudSelection widget (specialized for MUD menu)
- ✅ Offline MUD auto-injection as entry #0
- ✅ Comprehensive tests (27 tests for config/selection/integration)

**Remaining Work**:
- [ ] Load ~/.okros/config on Alt-O keypress
- [ ] Create MudSelection widget from loaded config
- [ ] Render widget to screen (Screen/Window integration)
- [ ] Handle modal input loop (arrow keys, enter to select)
- [ ] Connect to selected MUD via Mud::connect()

**Estimated effort**: 2-3 hours (full UI) or 30 minutes (simple CLI prompt)
**Alternative**: Use `#open` command or `MCL_CONNECT` env var (already working)

### 2. Perl Bot Integration Testing (Real-World Validation)
**Goal**: Validate production automation scripts

**Tasks**:
- [ ] Run existing Perl bot against headless mode
- [ ] Verify trigger/action execution via Perl match_prepare/substitute_prepare
- [ ] Test buffer reading + command sending via control protocol
- [ ] Document best practices for bot developers

**Estimated effort**: 1 day

### 3. Polish & Bug Fixes (As Discovered)
**Goal**: Address issues from real-world usage

**Tasks**:
- [ ] Fix any crashes/panics from production MUD connections
- [ ] Handle edge cases (telnet negotiations, ANSI variants)
- [ ] Improve error messages for better UX
- [ ] Optional: DNS resolution for interactive `#open` (headless already has it)
- [ ] Update README.md if needed
- [ ] Write user guide for Perl bot integration (control socket protocol)

## Quick Reference

**Build**:
```bash
cargo build                          # Base client
cargo build --features python        # With Python
cargo build --features perl          # With Perl
cargo build --all-features           # Everything
```

**Test**:
```bash
cargo test                           # Unit tests
cargo test --all-features            # Include plugin tests
```

**Run**:
```bash
cargo run                            # Interactive mode (TTY UI)
cargo run --offline                  # Offline mode (play internal MUD)
cargo run --headless --instance test # Headless mode (control via Unix socket)
cargo run --attach test              # Attach to headless instance
MCL_CONNECT=127.0.0.1:4000 cargo run # Auto-connect to MUD on startup
```

**Basic Usage** (interactive mode):
```
#open 127.0.0.1 4000   # Connect to MUD (IPv4 only currently)
type and press Enter   # Send to MUD
#quit                  # Exit
```

**Offline Mode** (internal MUD):
```
cargo run --offline    # Play built-in text adventure
look                   # Look around (starting room: Forest Clearing)
n                      # Go north
take sword             # Pick up rusty sword
inventory              # Check inventory
help                   # Show all commands
quit                   # Exit
```

## Key Files

**Documentation** (Read These First):
- `ORIENTATION.md` ← You are here
- `README.md` - User-facing overview
- `PORTING_HISTORY.md` - Historical record of C++ → Rust porting
- `FUTURE_WORK.md` - Remaining tasks and future enhancements
- `CLAUDE.md` - Project-specific dev guidelines
- `DDD.md` - Doc-Driven Development methodology

**Code Navigation**:
- `src/CODE_MAP.md` - Module-by-module guide to src/
- `src/plugins/CODE_MAP.md` - Plugin system guide
- `toys/` - Discovery phase experiments (12 toys with LEARNINGS.md)
  - `toys/toy12_internal_mud/` - Built-in test MUD for e2e validation

**Critical Paths**:
- `src/main.rs` - Event loop, CLI args, plugin loading (DONE - 318 lines)
- `src/engine.rs` - Headless SessionEngine (DONE)
- `src/control.rs` - Unix socket control server (DONE)
- `src/session.rs` - MCCP→telnet→ANSI→scrollback pipeline (DONE)

## Development Workflow

**Making Changes**:
1. Update `PORTING_HISTORY.md` / `FUTURE_WORK.md` if needed BEFORE committing structural changes
2. Update `CODE_MAP.md` if adding/removing/renaming files
3. Keep C++ reference (`mcl-cpp-reference/`) open for comparison
4. Commit with conventional format: `type(scope): description`

**Porting Discipline**:
- Reference implementation (`mcl-cpp-reference/`) is oracle
- Side-by-side comparison (always have source open)
- Behavioral equivalence > structural equivalence
- Document deviations: `// NOTE: differs from C++ because X`

## Questions?

**"Where do I start?"**
→ Testing! All implementation is done. Run `cargo run` and try `#open <mud-ip> <port>`

**"How do I test my changes?"**
→ `cargo test` (57 unit + 2 integration tests) + manual MUD connection

**"Is the event loop done?"**
→ YES! See `src/main.rs:147-290` (poll-based I/O, full pipeline wired)

**"Are plugins working?"**
→ YES! Build with `--features python` or `--features perl`, hooks implemented

**"How do I test headless mode?"**
→ `cargo run --headless --instance test`, then connect to `/tmp/okros/test.sock`

**"What's left to do?"**
→ Validation: Connect to real MUD, test with Perl bot, find/fix edge case bugs

**"What's the MVP definition?"**
→ Client connects to MUD, sends/receives text, Perl bot can automate via headless mode
→ **Status**: Implementation complete, awaiting validation ✅

---

**Remember**: This is a transport layer. Scripts handle the smart stuff. Keep it simple. 🦀
