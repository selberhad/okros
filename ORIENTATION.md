# ORIENTATION — okros MUD Client

**Quick Start**: You're looking at a Rust port of MCL (MUD Client for Linux). **Headless mode** (~95% complete) works great for bots. **TTY interactive mode** (~30% complete) needs significant restoration work.

## What Is This?

**okros** = Rust MUD client reviving MCL's design, optimized for automation
- **Primary use case**: Transport layer for Perl/Python bots and LLM agents
- **Philosophy**: Client handles I/O, scripts handle logic (automation via Perl/Python or built-in features)
- **Actual Status**: ~50% complete overall - see `PORT_GAPS.md` for comprehensive analysis

## Current State (Oct 2025)

**⚠️ CRITICAL: Port fell short of claims - see `PORT_GAPS.md` for detailed analysis**

### ✅ Headless Mode (~95% Complete)
- **Network**: Socket, telnet, MCCP compression, ANSI parsing (full pipeline)
- **Data Pipeline**: MCCP → telnet → ANSI → scrollback working
- **Control Server**: Unix socket JSON Lines protocol functional
- **Automation**: Triggers/actions work in headless mode
- **Offline Mode**: Internal MUD for testing (5 rooms, ANSI colors)
- **Plugins**: Python (pyo3) and Perl (raw FFI) with feature gates
- **Tests**: 134 tests passing
- **Validated**: Full gameplay session on Nodeka via headless mode

### ❌ TTY Interactive Mode (~30% Complete - BROKEN)

**57+ critical features missing** - documented in `PORT_GAPS.md`:

**Session management (82% missing)**:
- ❌ No connection state tracking
- ❌ No interpreter hooks (sys/connect, sys/loselink, sys/prompt, sys/output)
- ❌ Prompt handling broken
- ❌ Per-line trigger checking missing
- ❌ Macro expansion not called

**InputLine (75% missing)**:
- ❌ No command history (up/down arrows don't work)
- ❌ No command execution (Enter may not work)
- ❌ No interpreter integration
- ❌ Missing keyboard shortcuts (Ctrl-W, Delete, etc.)

**Window system (60% missing)**:
- ❌ No keypress dispatch
- ❌ No focus management
- ❌ No print()/printf() methods
- ❌ No scrolling

**OutputWindow (74% missing)**:
- ❌ Can't scroll back through history
- ❌ No search
- ❌ Display-only

**Command execution (100% missing)**:
- ❌ No command queue
- ❌ No speedwalk expansion
- ❌ No semicolon splitting
- ❌ No variable expansion
- ❌ plugins/stack.rs is NOT a port of Interpreter.cc

**InputBox (100% missing)**:
- ❌ Not ported at all
- ❌ No modal dialogs

See `PORT_GAPS.md` for complete analysis with line-by-line comparison.

### 🔴 Intentionally Deferred (By Design)
- Chat.cc - Inter-client chat (niche feature)
- Borg.cc - Network monitoring (privacy concern)
- Group.cc - Multi-client coordination (post-MVP)

## Next Steps: Systematic Restoration

**Goal**: Fill the gaps to reach actual 98% completion (~4-6 weeks)

**See `PORT_GAPS.md` for comprehensive action plan with 3 phases.**

### Phase 1: Fix Critical TTY Mode Bugs (1-2 weeks)

**Priority order for P0 gaps:**

1. **Session.cc restoration** (3-4 days)
   - Add connection state machine
   - Implement interpreter hooks (sys/connect, sys/prompt, sys/output, sys/loselink)
   - Add triggerCheck() integration per line
   - Add prompt buffering across reads
   - Add macro expansion call

2. **InputLine.cc restoration** (2-3 days)
   - Implement History class
   - Add Enter → execute() → interpreter.add()
   - Add sys/userinput hook
   - Add up/down arrow history
   - Add Ctrl-W, Delete, shortcuts

3. **Command execution engine** (2-3 days)
   - Find/create Interpreter equivalent
   - Implement command queue
   - Add semicolon splitting
   - Add speedwalk expansion
   - Wire to InputLine.execute()

4. **Window event dispatch** (1-2 days)
   - Implement keypress() virtual dispatch
   - Add focus management
   - Add print()/printf() methods

5. **OutputWindow scrolling** (1 day)
   - Add scroll() method
   - Add Page Up/Down handlers
   - Wire to ScrollbackController

6. **InputBox modal dialogs** (1 day)
   - Port InputBox base class
   - Add xy_center positioning
   - Add Escape handling

### Methodology

**Follow Execution Mode** (not Discovery):
1. Read C++ implementation **line-by-line**
2. Port logic **1:1 to Rust** (not rewrite!)
3. Test against C++ MCL behavior
4. Check off items in PORT_GAPS.md

**No new toys needed** - all risky patterns already validated in toys 1-12.

### Phase 2 & 3

See `PORT_GAPS.md` for complete roadmap (variable expansion, history save/load, search, etc.)

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
cargo run                              # Interactive mode (TTY UI)
cargo run --offline                    # Offline mode (play internal MUD)
cargo run --headless --instance test   # Headless mode (control via Unix socket)
cargo run --attach test                # Attach to headless instance
OKROS_CONNECT=127.0.0.1:4000 cargo run # Auto-connect to MUD on startup
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
- `ORIENTATION.md` ← You are here (current state overview)
- **`PORT_GAPS.md`** ← **START HERE for restoration work** (comprehensive gap analysis)
- `PORTING_HISTORY.md` - Historical record of C++ → Rust porting (overly optimistic)
- `LOC_COMPARISON.md` - Auto-generated line count comparison (flags short files)
- `README.md` - User-facing overview
- `FUTURE_WORK.md` - Post-restoration enhancements
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
→ Read `PORT_GAPS.md` for comprehensive gap analysis and restoration plan

**"What actually works?"**
→ Headless mode works great (~95% complete). TTY interactive mode is broken (~30% complete).

**"Why the discrepancy?"**
→ Port optimized for headless mode (new feature), abandoned TTY mode (original core feature). Claimed "98% complete" based on headless validation only.

**"How much work to fix TTY mode?"**
→ ~4-6 weeks of systematic restoration following PORT_GAPS.md Phase 1-3 action plan

**"Do we need more toys?"**
→ NO - all 12 toys complete, all risky patterns validated. Remaining work is straightforward porting.

**"What's the actual completion?"**
→ ~50% overall (28.2% of critical file LOC, 57+ features missing). See PORT_GAPS.md conclusion.

**"Can I use this now?"**
→ YES for headless automation. NO for interactive TTY use (many features broken).

**"What went wrong?"**
→ Focused on new headless feature, skipped TTY restoration, claimed completion prematurely. See PORT_GAPS.md root cause analysis.

---

**Remember**: This is a transport layer. Scripts handle the smart stuff. Keep it simple. 🦀
