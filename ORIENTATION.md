# ORIENTATION — okros MUD Client

**Quick Start**: You're looking at a Rust port of MCL (MUD Client for Linux), ~70% complete, built for headless operation and LLM agent integration.

## What Is This?

**okros** = Rust MUD client reviving MCL's design, optimized for automation
- **Primary use case**: Transport layer for Perl/Python bots and LLM agents
- **Philosophy**: Client handles I/O, scripts handle logic (aliases/triggers/automation)
- **Status**: MVP nearly complete (network, UI, plugins done; event loop pending)

## Current State (Jan 2025)

### ✅ What Works
- **Network**: Socket, telnet, MCCP compression, ANSI parsing
- **UI Components**: ncurses wrapper, screen renderer, widgets, scrollback buffer
- **Plugins**: Python (pyo3) and Perl (raw FFI) interpreters with feature gates
- **Headless Engine**: SessionEngine + control server (Unix socket, JSON Lines)
- **Discovery**: 11 toys validated all risky FFI/unsafe patterns

### ⏸️ What's Pending
- **Main Event Loop**: Wire TTY input → MUD socket → screen output
- **CLI Args**: `--headless`, `--instance`, `--attach` not implemented
- **Plugin Loading**: Feature-gated initialization in main
- **Integration Tests**: End-to-end validation (connect to real MUD, test with Perl bot)

### ❌ What's Deferred
- Aliases, actions, hotkeys (Perl/Python scripts handle these)
- Chat, borg, group features (not needed for MVP)
- Complex # command interpreter (minimal only: #quit, #open)

## Next Steps (Priority Order)

### 1. Wire Event Loop (Tier 6 - Main)
**Goal**: Functional standalone binary that connects to MUDs

**Tasks**:
- [ ] Implement select/poll on sockets + TTY in `src/main.rs`
- [ ] Connect: TTY input → send to MUD socket
- [ ] Connect: MUD socket → ANSI parser → screen renderer
- [ ] Add basic # commands: `#quit`, `#open <host> <port>`
- [ ] Wire plugin loading behind `--features python/perl`

**Files to edit**: `src/main.rs` (expand current demo)

**Reference**: C++ `main.cc` event loop structure

### 2. Add CLI Args (Tier 6 - Main)
**Goal**: Support headless and attach modes

**Tasks**:
- [ ] Parse args: `--headless`, `--instance <name>`, `--attach <name>`
- [ ] Headless mode: Start SessionEngine, publish Unix socket
- [ ] Attach mode: Connect to running engine, render via Screen
- [ ] Interactive mode (default): Full TTY UI

**Dependency**: Needs step 1 (event loop) complete

### 3. Integration & Validation (Tier 7)
**Goal**: Prove it works end-to-end

**Tasks**:
- [ ] Manual smoke test: `okros example.com 4000` → connect, send/receive
- [ ] Headless test: Start headless, send commands via control socket
- [ ] Perl bot integration: Run your real bot, validate transport layer
- [ ] Feature combos: Test base, `--features python`, `--features perl`, all

**Success**: Can play a MUD interactively AND your Perl bot can automate via headless mode

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

**Current Demo** (minimal):
```bash
cargo run                            # Tiny TTY demo with key input
```

## Key Files

**Documentation** (Read These First):
- `ORIENTATION.md` ← You are here
- `README.md` - User-facing overview
- `IMPLEMENTATION_PLAN.md` - Comprehensive status (living document)
- `CLAUDE.md` - Project-specific dev guidelines
- `DDD.md` - Doc-Driven Development methodology

**Code Navigation**:
- `src/CODE_MAP.md` - Module-by-module guide to src/
- `src/plugins/CODE_MAP.md` - Plugin system guide
- `toys/` - Discovery phase experiments (11 toys with LEARNINGS.md)

**Critical Paths**:
- `src/main.rs` - Event loop (NEEDS WORK)
- `src/engine.rs` - Headless SessionEngine (DONE)
- `src/control.rs` - Unix socket control server (DONE)
- `src/session.rs` - MCCP→telnet→ANSI→scrollback pipeline (DONE)

## Development Workflow

**Making Changes**:
1. Update `IMPLEMENTATION_PLAN.md` status BEFORE committing structural changes
2. Update `CODE_MAP.md` if adding/removing/renaming files
3. Keep C++ reference (`mcl-cpp-reference/`) open for comparison
4. Commit with conventional format: `type(scope): description`

**Porting Discipline**:
- Reference implementation (`mcl-cpp-reference/`) is oracle
- Side-by-side comparison (always have source open)
- Behavioral equivalence > structural equivalence
- Document deviations: `// NOTE: differs from C++ because X`

## Questions?

**"Where do I start coding?"**
→ `src/main.rs` - Wire the event loop (see Next Steps #1)

**"How do I test my changes?"**
→ `cargo test` + manual `cargo run` smoke test

**"What if I need to validate a risky pattern?"**
→ Build a toy in `toys/toyN_name/` with SPEC/PLAN/LEARNINGS

**"How do plugins work?"**
→ See `src/plugins/CODE_MAP.md` + `toys/toy4_python/` and `toys/toy5_perl/`

**"What's the MVP definition?"**
→ Client connects to MUD, sends/receives text, Perl bot can automate via headless mode

---

**Remember**: This is a transport layer. Scripts handle the smart stuff. Keep it simple. 🦀
