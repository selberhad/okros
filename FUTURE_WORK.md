# Future Work & Enhancements

This document tracks post-MVP enhancements and deferred features for okros.

**Current Status**: Implementation ~95% complete, in validation phase
**MVP Philosophy**: okros is a transport layer - scripts handle command logic

---

## Immediate Priorities (Validation Phase)

### 1. ✅ Internal MUD Integration (DONE)
- [x] Port `toys/toy12_internal_mud/` to `src/offline_mud/` module
- [x] Add `--offline` CLI flag to main.rs
- [x] Wire internal MUD World + Session direct feed (no socket)
- [x] Validate with existing toy12 integration tests

**Status**: Complete (commit a091f0c)

### 2. Integration & Validation Testing — STATUS: COMPLETE ✅
**Priority**: HIGH - Required for MVP completion

- [x] Manual smoke test: connect to real MUD server
  - **VALIDATED**: Connected to Nodeka (nodeka.com:23) successfully
  - **VALIDATED**: Full gameplay session (char creation, questing, combat, leveling)
  - See `MUD_LEARNINGS.md` for comprehensive real-world test results
- [x] Internal MUD smoke test: `cargo run --offline` → game works
- [x] Feature combination testing:
  - Base build working
  - Python/Perl feature gates functional
- [x] **Real MUD Integration**: First AI/LLM to autonomously play Nodeka (2025-10-03)
  - Character creation, questing, combat, navigation all validated
  - Headless mode control protocol fully functional
  - ANSI rendering, game state tracking working correctly

**Status**: MVP validation complete. okros is production-ready for MUD gameplay.

### 3. Polish & Bug Fixes
**Priority**: HIGH - As discovered during testing

- [ ] Fix any panics/crashes found during MUD connection
- [ ] Address edge cases in telnet/ANSI parsing
- [ ] Improve error messages for better UX

### 4. Documentation Updates
**Priority**: MEDIUM - Keep docs in sync

- [x] Restructure IMPLEMENTATION_PLAN.md → PORTING_HISTORY.md
- [x] Extract future work to FUTURE_WORK.md
- [ ] Update ORIENTATION.md to reflect MVP status
- [ ] Update README.md if needed
- [ ] Update CODE_MAP.md for src/offline_mud/

---

## Optional Enhancements (Post-MVP v0.1)

### 5. DNS Hostname Resolution
**Priority**: LOW - Nice-to-have, workarounds exist

- Currently only IPv4 addresses work (e.g., `#open 127.0.0.1 4000`)
- Add hostname lookup (e.g., `#open example.com 4000`)
- Low priority - can be handled by wrapper scripts or Perl/Python pre-processing

**Estimated effort**: 1-2 hours

### 6. Extended # Commands
**Priority**: LOW - Minimal set sufficient for MVP

- Current set: `#quit`, `#open`
- C++ MCL has many more (see `mcl-cpp-reference/Interpreter.cc`)
- Most commands should be deferred to Perl/Python scripts (consistent with transport layer philosophy)

**Examples of C++ # commands**:
- `#save`, `#load` (session state)
- `#cd`, `#pwd` (directory navigation)
- `#exec` (run shell commands)
- **Recommendation**: Implement these as Perl/Python scripts instead

**Estimated effort**: Varies by command complexity

---

## Core Features (v0.2+)

### 6. Alias/Action/Macro System — STATUS: PARTIALLY COMPLETE ✅

**Implementation Complete**:
- [x] `src/alias.rs` - Text expansion with %N parameters (%1, %-2, %+3)
- [x] `src/action.rs` - Trigger/replacement/gag with regex (via Perl/Python)
- [x] `src/macro_def.rs` - Keyboard shortcut bindings
- [x] # commands: `#alias`, `#action`, `#subst`, `#macro`
- [x] Storage in MUD struct (per-session lists)

**Remaining Work**:
- [ ] Wire alias expansion into input pipeline
- [ ] Wire action/trigger checking into output pipeline
- [ ] Implement Perl/Python regex integration (match_prepare/substitute_prepare)
- [ ] Macro key code lookup (F1, F2, etc. - currently ASCII only)
- [ ] Hierarchical lookup (global → session-specific)

**Why Needed**: Simple automation without requiring full Perl/Python scripts. Good for quick aliases and basic triggers.

**Estimated effort**: 1-2 days to complete integration

## Deferred to v1.0+ (Not Needed for MVP)

### 7. Connect Menu & Config File Parsing
**Priority**: MEDIUM - Quality of life feature

Port C++ connect menu system:
- **Selection.cc** (UI list widget) - base class for menus
- **MUDSelection widget** - connect menu triggered by Alt-O
- **Config file parsing** (~/.okros/config) - load MUD definitions
- **MUD list storage** (MUDList class) - manage saved MUDs

**MVP approach**: Use `--offline` flag to launch internal MUD directly
**Future approach**: Port Selection/MUDSelection, add internal MUD as default entry #0

**Implementation steps**:
1. Port Selection base class (scrollable list widget)
2. Port MUDSelection (specialized for MUD connections)
3. Implement config file parser (old format: `mudname hostname port [commands]`)
4. Implement config file parser (new format: `MUD mudname { host hostname port; alias ...; }`)
5. Add Alt-O hotkey binding
6. Add internal MUD as entry #0 in empty lists

**Estimated effort**: 1-2 days
**Reference**: `mcl-cpp-reference/Selection.cc`, `mcl-cpp-reference/Config.cc` (lines 329-508)

### 8. Client-Side Command Processing
**Priority**: LOW - Perl/Python handles this by design

Deferred features (scripts should handle):
- **Alias.cc** (command expansion) - scripts handle
- **Hotkey.cc** (keyboard macros) - scripts handle
- **Advanced interpreter** (# commands) - minimal set sufficient for transport layer

**Rationale**: okros philosophy is "client handles I/O, scripts handle logic"

### 9. Advanced MCL Features
**Priority**: OUT OF SCOPE - Niche or problematic

Features explicitly skipped:
- **Chat.cc** (peer-to-peer chat) - niche feature, small user base
- **Borg.cc** (phone-home stats) - privacy concern
- **Group.cc** (grouped sessions) - complex feature, defer to post-MVP

**Estimated effort**: N/A (not planned)

### 10. Cross-Platform & Performance
**Priority**: LOW - Future iteration

- **macOS/Windows support** (currently Linux-only)
  - ncurses/TTY abstractions need platform-specific implementations
  - Consider `crossterm` or similar for cross-platform TUI
- **Performance profiling and optimization**
  - Profile hot paths with real MUD usage
  - Optimize screen diffing algorithm if needed
- **Idiomatic Rust refactoring pass**
  - Review unsafe usage, reduce where possible
  - Apply Clippy suggestions
  - Improve error handling (less unwrap/expect)

**Estimated effort**: 1-2 weeks per platform

---

## Known Limitations (By Design)

These are intentional trade-offs for the MVP:

1. **IPv4 only** - No DNS hostname resolution (use scripts or IP addresses)
2. **Minimal # commands** - Only `#quit`, `#open` (scripts handle rest)
3. **No config file loading** - MUD connections via CLI or `#open` command
4. **No connect menu** - Use `--offline` for internal MUD, or `#open` for network
5. **Linux-only** - ncurses/TTY code is platform-specific
6. **Single session** - No multi-session or grouped sessions (tmux/screen for multiple instances)

**Rationale**: Keep core simple, let ecosystem tools (scripts, tmux, DNS) handle complexity

---

## Ideas & Future Exploration

### Integration with LLM Agents
- Headless mode already supports JSON Lines control protocol
- Consider structured event format (optional, alongside raw buffer)
- Example: `{"event":"room","name":"Forest","exits":["north","south"]}`
- **Needs research**: Do LLMs benefit from structured data, or is raw MUD text better?

### Plugin System Enhancements
- **Hot reload** - Reload Perl/Python scripts without restarting client
- **Plugin manager** - Install/update scripts from repository
- **Sandboxing** - Limit plugin capabilities (file access, network, etc.)

### Linux Virtual Console Support (`/dev/vcsa`)
**Priority**: LOW - Historical feature, fun to implement

Original MCL's secret weapon for blazing-fast rendering:
- **Direct memory-mapped access** to Linux virtual console framebuffer
- Writes screen + cursor in **single syscall** via `writev()`
- No ANSI parsing overhead, no escape sequence generation
- **Much faster** than terminal emulation (instant screen updates)

**How it worked (C++ MCL):**
```cpp
// Detect Linux VT: /dev/tty1 → /dev/vcsa1
char buf[256];
sprintf(buf, "/dev/vcsa%d", ttyno);
fd = open(buf, O_WRONLY);

// Write entire screen in one shot
writev(fd, {cursor_pos, canvas}, 2);  // Instant!
```

**Current status:**
- okros uses ANSI escape sequences (portable, works everywhere)
- Screen diffing minimizes output (only changed cells)
- Performance is excellent (Nodeka gameplay validated)

**Why it's deferred:**
- **Narrow use case**: Only Linux virtual consoles (Ctrl+Alt+F1-F6)
- **Not available** on: macOS, SSH, xterm, tmux, WSL, etc.
- **okros philosophy**: Headless/detachable mode is primary use case
- **Permissions**: Requires root or setgid tty group membership

**Why it could be fun:**
- Cool historical technology (1990s Linux optimization)
- Simple Rust FFI implementation (~100 LOC)
- Educational: Learn about `/dev/vcs*` character devices
- Performance testing: Measure ANSI vs vcsa on real Linux VT

**Implementation approach:**
1. Detect Linux VT: Parse `ttyname(0)` for `/dev/tty[0-9]+`
2. Open `/dev/vcsa{N}` for writing
3. Fallback to ANSI if open fails (portable default)
4. Write cursor position (2 bytes) + canvas (width×height×2 bytes)
5. Feature gate: `--features vcsa` (Linux-only compilation)

**Estimated effort**: 4-6 hours (straightforward FFI)
**Testing requirements**: Real Linux system with VT access (not macOS/VM)

**References:**
- `mcl-cpp-reference/Screen.cc` (lines 39-100: vcsa detection & writing)
- Linux kernel docs: `/dev/vcs` and `/dev/vcsa` character devices
- Historical context: Before GPU acceleration, this was THE way to do fast TUIs

**Fun fact**: This is why MCL was so fast in the 1990s - direct framebuffer access was revolutionary for a MUD client!

### WASM Plugin System (v2)
**Priority**: FUTURE - Post-v1.0 architectural enhancement

Replace language-specific FFI (pyo3, Perl XS) with universal WASM runtime:
- **Multi-language support** - Any WASM-capable language (Rust, C, C++, Go, AssemblyScript, etc.)
- **Sandboxing by default** - WASM's capability-based security model
- **Hot reload** - WASM modules can be loaded/unloaded dynamically
- **Performance** - Near-native execution, no interpreter overhead
- **Distribution** - Single `.wasm` file per plugin, no language runtime dependencies

**Implementation approach**:
- Runtime: `wasmtime` or `wasmer` (both mature Rust WASM runtimes)
- Host API: WASI + custom okros extensions (MUD events, send commands, buffer access)
- Plugin interface: Standardized event handlers (on_line, on_prompt, on_connect, etc.)
- Backward compat: Keep Perl/Python support alongside WASM (feature gates)

**Example plugin workflow**:
1. Write plugin in any language (Rust, Go, C++, etc.)
2. Compile to WASM with okros bindings
3. Drop `.wasm` file in `~/.okros/plugins/`
4. Client loads at runtime, calls event handlers

**Benefits over current FFI**:
- No per-language integration (current: pyo3 for Python, raw FFI for Perl)
- Sandboxed by default (current: full process access)
- Cross-platform (WASM is portable, native FFI is not)
- Version isolation (each plugin brings its own runtime)

**Estimated effort**: 2-3 weeks for MVP WASM runtime integration
**Reference**: `wasmtime` embedding guide, WASI preview 2 spec

### Toy12 (Internal MUD) Enhancements
- More rooms, items, NPCs (richer offline demo)
- Save/load game state (persistent offline world)
- Script-driven content (Perl/Python can extend the MUD)
- Use as test harness for protocol edge cases

### Testing Infrastructure
- Golden tests: Compare Rust output with C++ MCL
- Protocol fuzzing: Generate random telnet/ANSI sequences
- Stress testing: High-volume MUD output, long sessions
- Headless CI: Automated testing without TTY

---

## How to Contribute

**Adding new features**:
1. Check if feature aligns with "transport layer" philosophy
2. If it's command logic → implement as Perl/Python script
3. If it's core I/O → propose in issue/PR with C++ reference

**Porting C++ features**:
1. Reference `mcl-cpp-reference/` for original implementation
2. Follow patterns in `PORTING_HISTORY.md` (formerly IMPLEMENTATION_PLAN.md)
3. Update `CODE_MAP.md` before structural commits
4. Aim for behavioral equivalence, not structural fidelity

**Documentation updates**:
- `ORIENTATION.md` - High-level status and next steps
- `PORTING_HISTORY.md` - Historical record of porting work
- `FUTURE_WORK.md` - This file (future enhancements)
- `CODE_MAP.md` - Module-by-module structure

---

**Last Updated**: 2025-10-03
**Status**: Validation phase, ~95% implementation complete
