# ORIENTATION — okros MUD Client

**Quick Start**: You're looking at a Rust port of MCL (MUD Client for Linux), ~95% complete (implementation done, validation pending), built for headless operation and LLM agent integration.

## What Is This?

**okros** = Rust MUD client reviving MCL's design, optimized for automation
- **Primary use case**: Transport layer for Perl/Python bots and LLM agents
- **Philosophy**: Client handles I/O, scripts handle logic (aliases/triggers/automation)
- **Status**: Implementation complete (all tiers done); validation pending (needs real MUD testing)

## Current State (Oct 2025)

### ✅ What Works (Implementation Complete)
- **Network**: Socket, telnet, MCCP compression, ANSI parsing (full pipeline)
- **UI**: ncurses wrapper, screen diff renderer, widgets (status/output/input), scrollback
- **Plugins**: Python (pyo3) and Perl (raw FFI) with feature gates, interpreter hooks
- **Headless Engine**: SessionEngine + control server (Unix socket, JSON Lines protocol)
- **Offline Mode**: Internal MUD for testing/demo (5 rooms, 3 items, ANSI colors, `--offline` flag)
- **Main Event Loop**: poll-based I/O on TTY + socket with 250ms timeout
- **CLI Args**: `--headless`, `--instance <name>`, `--attach <name>`, `--offline`, `--headless --offline` (combined) implemented
- **# Commands**: `#quit`, `#open`, `#alias`, `#action`, `#subst`, `#macro` functional
- **Automation**: Alias expansion (text with %N params), triggers/replacements, keyboard macros
- **Tests**: 97 total tests passing (89 unit + 8 integration) | 70% coverage
- **✅ MUD VALIDATED**: Full gameplay session on Nodeka (2025-10-03) - See `MUD_LEARNINGS.md`

### ⏸️ What Needs Completion
- **Alias/Action Integration**: Modules exist, need wiring into I/O pipeline
- **Perl/Python Regex**: Implement match_prepare/substitute_prepare for actions
- **Perl Bot Validation**: Test real automation scripts against headless mode

### ❌ What's Deferred (By Design)
- Chat, borg, group features (not needed for MVP)
- Extended # commands beyond current set
- DNS hostname resolution (IPv4 addresses work; scripts can resolve)
- Config file parsing and connect menu (use #open or MCL_CONNECT env var)

## Next Steps (Priority Order)

**MVP is VALIDATED. Focus is now on polishing automation features.**

### 1. Complete Alias/Action/Macro Integration
**Goal**: Wire automation features into I/O pipeline

**Tasks**:
- [ ] Input pipeline: Check aliases before sending to MUD
- [ ] Input pipeline: Check macros on key press
- [ ] Output pipeline: Check triggers/replacements on MUD lines
- [ ] Implement Perl match_prepare/substitute_prepare methods
- [ ] Implement Python match_prepare/substitute_prepare methods
- [ ] Test full automation workflow (alias → trigger → action)

**Estimated effort**: 1-2 days

### 2. Perl Bot Integration Testing
**Goal**: Validate real automation scripts

**Tasks**:
- [ ] Run existing Perl bot against headless mode
- [ ] Verify script can read buffer and send commands
- [ ] Test trigger/action execution via Perl
- [ ] Document best practices for bot developers

**Estimated effort**: 1 day

### 2. Polish & Bug Fixes (As Discovered)
**Goal**: Fix issues found during validation

**Tasks**:
- [ ] Address any panics/crashes from real MUD connections
- [ ] Handle telnet/ANSI edge cases (IAC escaping, SGR variants, etc.)
- [ ] Improve error messages for better UX
- [ ] Optional: Add DNS hostname resolution (currently IPv4 only)

### 3. Documentation Finalization
**Goal**: Ensure docs match reality

**Tasks**:
- [x] Restructure IMPLEMENTATION_PLAN.md → PORTING_HISTORY.md + FUTURE_WORK.md
- [x] Update ORIENTATION.md (this file) to reflect MVP status
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
