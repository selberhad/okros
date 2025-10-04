# ORIENTATION — okros MUD Client

**Quick Start**: You're looking at a Rust port of MCL (MUD Client for Linux). **Headless mode** (~95% complete) works great for bots. **TTY interactive mode** (~90% complete after Phase 2) - Session, InputLine, and command expansion complete.

## What Is This?

**okros** = Rust MUD client reviving MCL's design, optimized for automation
- **Primary use case**: Transport layer for Perl/Python bots and LLM agents
- **Philosophy**: Client handles I/O, scripts handle logic (automation via Perl/Python or built-in features)
- **Actual Status**: ~92% complete overall (Phase 2 done) - see `PORT_GAPS.md` for comprehensive analysis

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

### ✅ TTY Interactive Mode (~90% Complete - Phase 2 Done)

**Phase 1 & 2 complete** - documented in `PORT_GAPS.md`:

**✅ Session management (Phase 1 - COMPLETE)**:
- ✅ Connection state tracking (SessionState, SessionManager)
- ✅ Interpreter hooks (sys/connect, sys/loselink, sys/prompt, sys/output)
- ✅ Prompt handling with multi-read buffering
- ✅ Per-line trigger checking integrated
- ✅ Macro expansion support
- ✅ Connection lifecycle (open/close/write_mud/idle)
- ✅ Statistics tracking

**✅ InputLine (Phase 2 - COMPLETE)**:
- ✅ Command history with persistence (~/.mcl/history)
- ✅ History cycling (up/down arrows)
- ✅ Command execution (Enter key)
- ✅ All keyboard shortcuts (Ctrl-A/E/U/W/K/J/C, Delete, arrows)
- ✅ Horizontal scrolling
- ✅ Prompt display with color stripping

**✅ Command Execution (Phase 2 - COMPLETE)**:
- ✅ Command queue with recursion protection
- ✅ Speedwalk expansion (3n2e → n;n;n;e;e)
- ✅ Semicolon splitting (north;south → 2 commands)
- ✅ Variable expansion (%h hostname, %p port, %H hour, %m minute, etc.)
- ✅ Alias expansion (fully integrated with MUD.find_alias())
- ✅ Full expansion pipeline (VARIABLES → ALIASES → SPEEDWALK → SEMICOLON)

**🟡 Remaining Gaps** (minor, ~10%):
- Window keypress() virtual dispatch (low priority)
- OutputWindow scrolling (Page Up/Down)
- InputBox modal dialogs (not yet needed)

See `PORT_GAPS.md` for complete analysis.

### 🔴 Intentionally Deferred (By Design)
- Chat.cc - Inter-client chat (niche feature)
- Borg.cc - Network monitoring (privacy concern)
- Group.cc - Multi-client coordination (post-MVP)

## Status Summary

**Phase 1**: ✅ **COMPLETE** (Session restoration - 100%)
**Phase 2**: ✅ **COMPLETE** (InputLine & command expansion - 100%)

**Overall**: ~92% complete, fully functional for interactive use

**Remaining work** (Phase 3 - optional polish):
- Window scrolling infrastructure (~5%)
- Modal dialogs (~3%)

**See `PORT_GAPS.md` for detailed completion analysis.**

### ✅ Phase 1 COMPLETE: Session.cc Restoration (commits 30eaf2f, 31902a7, b6ee0fb)

**All P0 Session gaps filled:**
- ✅ Connection state machine (SessionState, SessionManager)
- ✅ Interpreter hooks (sys/connect, sys/prompt, sys/output, sys/loselink)
- ✅ Trigger checking per line (check_line_triggers)
- ✅ Prompt multi-read buffering (handle_prompt_event)
- ✅ Macro expansion (expand_macros)
- ✅ Connection management (open/close/write_mud/idle)
- ✅ Statistics tracking (SessionStats)
- ✅ MUD action methods (check_action_match, check_replacement)

**Impact**: Session infrastructure complete - triggers, prompts, hooks ready for Phase 2 integration

### 🚀 Phase 2: InputLine & Command Engine (2-3 weeks)

**Priority order:**

1. **InputLine.cc restoration** (2-3 days) - **NEXT UP**
   - Implement History class (command history ring buffer)
   - Add execute() method (Enter → interpreter queue)
   - Add sys/userinput hook
   - Port keyboard shortcuts (up/down arrows, Ctrl-W, Delete)
   - History save/load to ~/.mcl/history

2. **Command execution engine** (2-3 days)
   - Create Interpreter command queue
   - Implement semicolon splitting
   - Add speedwalk expansion (3n2e → n;n;n;e;e)
   - Wire to InputLine.execute()

3. **Window event dispatch** (1-2 days)
   - Implement keypress() virtual dispatch
   - Add focus management
   - Add print()/printf() methods

4. **OutputWindow scrolling** (1 day)
   - Add scroll() method
   - Add Page Up/Down handlers
   - Wire to ScrollbackController

5. **InputBox modal dialogs** (1 day)
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

### Phase 3 & Beyond

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
→ Headless mode works great (~95% complete). TTY interactive mode ~65% complete (Phase 1 done: Session infrastructure).

**"Why the discrepancy?"**
→ Port optimized for headless mode (new feature), initially abandoned TTY mode. Now restoring systematically via Phase 1-3 plan.

**"How much work to fix TTY mode?"**
→ ~3-4 weeks remaining (Phase 1 complete, Phase 2-3 in progress). See PORT_GAPS.md for detailed plan.

**"Do we need more toys?"**
→ NO - all 12 toys complete, all risky patterns validated. Remaining work is straightforward porting.

**"What's the actual completion?"**
→ ~80% overall (Phase 1 complete: Session restoration). See PORT_GAPS.md for remaining Phase 2-3 items.

**"Can I use this now?"**
→ YES for headless automation. PARTIAL for TTY (triggers/prompts/hooks work; history/command-queue next).

**"What's been restored?"**
→ Phase 1 complete (Session): connection mgmt, interpreter hooks, triggers, prompts, macros, statistics. Phase 2 next: InputLine history and command execution.

---

**Remember**: This is a transport layer. Scripts handle the smart stuff. Keep it simple. 🦀
