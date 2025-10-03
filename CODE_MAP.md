# MCL Rust Port - Code Map

This file tracks the structure and status of the codebase.

## Root Directory

### Documentation
- `CLAUDE.md` - Project instructions for Claude Code (methodology, workflow)
- `TOY_PLAN.md` - Discovery phase strategy and progress (toys 1-5)
- `IMPLEMENTATION_PLAN.md` - Execution phase strategy (tier-by-tier port)
- `DDD.md` - Doc-Driven Development methodology
- `PLAYBOOK.md` - Condensed porting workflow guide
- `AGENTS.md` - Quick reference summary
- `TOY_DEV.md` - Toy development methodology
- `CODE_MAP.md` - This file (project structure map)

### Configuration
- `Cargo.toml` - Not yet created (will be root Rust project)

### Source Code
- `src/` - Not yet created (main Rust port will go here)
- `tests/` - Not yet created (integration tests)

### Reference Implementation
- `mcl-cpp-reference/` - Original C++ codebase (~11k LOC, 29 .cc files, ~50 headers)
  - `*.cc` - C++ implementation files
  - `h/*.h` - C++ header files
  - `plugins/` - Python/Perl interpreter plugins

---

## Toys (Discovery Phase Artifacts)

**Status**: 5/5 complete ✅ (Discovery Phase COMPLETE!)

### toys/toy1_string_buffer/
**Status**: Analysis complete (SKIPPED toy implementation)
**Decision**: Use Rust `String` and `Vec<u8>` with minimal wrappers
**Key findings**:
- C++ String class is simple char* wrapper
- Only quirk: case-insensitive comparison (use `.eq_ignore_ascii_case()`)
- Buffer class maps cleanly to `Vec<u8>`
- StaticBuffer optimization unnecessary in modern Rust

**Files**:
- `LEARNINGS.md` - Analysis and decisions

### toys/toy2_ncurses/
**Status**: COMPLETE ✅
**Decision**: Use `ncurses` crate (raw FFI), skip `pancurses`
**Key findings**:
- MCL uses ncurses minimally (terminal setup, capability queries only)
- No ncurses window management (custom in-memory window system)
- Direct ANSI escape output (no ncurses rendering functions)
- `ncurses` crate provides sufficient low-level access

**Files**:
- `Cargo.toml` - Toy dependencies
- `src/ncurses_test.rs` - Basic terminfo access test
- `src/complete_test.rs` - Full initialization pattern (matching C++)
- `src/pancurses_test.rs` - Comparison test (shows limitations)
- `LEARNINGS.md` - Complete pattern documentation
- `README.md` - Quick reference

### toys/toy3_globals/
**Status**: COMPLETE ✅
**Decision**: Use `unsafe static mut` with helper functions
**Key findings**:
- Single-threaded architecture = no data races
- Helper functions hide `unsafe` in one place
- Clean call sites: `screen().refresh()` (just like C++)
- OnceCell adds complexity without benefit (Sync issues)
- Zero runtime overhead, perfect C++ mapping

**Files**:
- `Cargo.toml` - Toy dependencies
- `src/raw_static_test.rs` - Validated pattern (WORKS ✅)
- `src/oncecell_test.rs` - OnceCell comparison (rejected)
- `LEARNINGS.md` - Complete pattern documentation with production template
- `README.md` - Quick reference

### toys/toy4_python/
**Status**: COMPLETE ✅
**Decision**: Use `pyo3` (simpler and safer than C API)
**Key findings**:
- pyo3 abstracts Python C API beautifully
- Automatic reference counting (no manual INCREF/DECREF)
- Result<> for error handling (no PyErr_Print)
- GIL management via `Python::with_gil()`
- All C++ patterns replicate cleanly (eval, load_file, call_function, get/set vars)

**Files**:
- `Cargo.toml` - pyo3 dependency
- `src/lib.rs` - PythonEmbeddedInterpreter wrapper with tests
- `LEARNINGS.md` - Complete pyo3 pattern documentation
- `README.md` - Quick reference

### toys/toy5_perl/
**Status**: COMPLETE ✅
**Decision**: Use raw FFI with `PERL_SYS_INIT3` for modern Perl
**Key findings**:
- MCL targets Perl 5.10 (2007-era), modern Perl 5.34+ has threading
- **CRITICAL**: `PERL_SYS_INIT3()` required for threaded Perl (didn't exist in 5.10)
- Function name mangling: Most use `Perl_` prefix (e.g., `Perl_eval_pv`)
- Threading context (pTHX_) becomes explicit first parameter
- Working init sequence: sys_init3 → alloc → construct → parse → run → destruct → free → sys_term
- perl_eval_pv, variable get/set all working perfectly

**Files**:
- `build.rs` - Build system (finds libperl via perl -MConfig)
- `src/perl_ffi_test.rs` - Complete working FFI implementation
- `test2.c` - C reference with PERL_SYS_INIT3 (proves solution)
- `LEARNINGS.md` - Complete FFI pattern documentation (including modern Perl fix)
- `README.md` - Solution summary

---

## Source Code (Not Yet Created)

When porting begins, structure will follow IMPLEMENTATION_PLAN.md tiers:

```
src/
  globals.rs          # Global state (screen, config, currentSession)

  # Tier 1: Foundation
  string.rs           # String utilities (minimal wrappers)
  buffer.rs           # Buffer class (Vec<u8> wrapper)
  color.rs            # Color/attribute constants
  list.rs             # List utilities

  # Tier 2: Core
  selectable.rs       # Selectable interface
  tty.rs              # TTY operations
  config.rs           # Configuration
  mud.rs              # MUD connection
  socket.rs           # Socket handling

  # Tier 3: UI
  curses.rs           # ncurses wrapper
  window.rs           # Window class (in-memory canvas)
  screen.rs           # Screen class (rendering)
  output_window.rs    # Output window
  input_line.rs       # Input line
  status_line.rs      # Status line

  # Tier 4: Logic
  session.rs          # Session management
  alias.rs            # Alias system
  hotkey.rs           # Hotkey system
  interpreter.rs      # Base interpreter
  chat.rs             # Chat system
  borg.rs             # Borg mode

  # Tier 5: Plugins (optional features)
  python.rs           # Python embedding (--features python)
  perl.rs             # Perl embedding (--features perl)

  # Tier 6: Application
  main.rs             # Main entry point
```

---

## Status Summary

**Current Phase**: Discovery Phase COMPLETE ✅ → Ready for Execution Phase

**Key Decisions Made**:
- ✅ String/Buffer: Use Rust stdlib
- ✅ ncurses: Use `ncurses` crate (raw FFI)
- ✅ Globals: Use `unsafe static mut` with helpers
- ✅ Python: Use `pyo3` (simpler than C API)
- ✅ Perl: Raw FFI with `PERL_SYS_INIT3` for modern Perl

**All Patterns Validated**: No blockers - ready to begin tier-by-tier porting (IMPLEMENTATION_PLAN.md)

**Next Steps**:
1. Initialize git repository
2. Begin Execution Phase (Tier 1: Foundation)
3. Port following IMPLEMENTATION_PLAN.md structure
