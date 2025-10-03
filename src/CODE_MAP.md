# CODE MAP — src/

Brief index mapping Rust modules to their C++ counterparts (when applicable). Update before structural commits.

## Core Modules

- `lib.rs` → Module declarations + feature gate organization (no C++ analog).
- `main.rs` → `main.cc` (**SUBSTANTIALLY COMPLETE** - full event loop structure matching main.cc:141-170, plugin initialization, # commands, interpreter hooks).
- `globals.rs` → Global state (placeholder; Toy 3 pattern documented but **not yet applied** - using locals in main.rs instead).

## Foundation (Tier 1)

- `color.rs` → Color/attribute constants.
- `ansi.rs` → ANSI SGR/attrib conversion (from rendering logic in `OutputWindow.cc`/`Screen.cc`).

## Core Abstractions (Tier 2)

- `selectable.rs` → `Selectable.cc` (trait definition; **no implementations yet** - gap blocking event loop).
- `select.rs` → poll wrapper analogous to `Selection.cc`.
- `socket.rs` → `Socket.cc` (nonblocking IPv4 socket over raw fd; Toy 9 patterns).
- `tty.rs` → `TTY.cc` (raw mode + keypad app mode; Toy 6 patterns).
- `input.rs` → Key decoder (ESC sequence normalization; from `TTY.cc` + Toy 6).
- `config.rs` → `Config.cc` (config file parser; old/new format; MUD list; auto-injects Offline MUD).
- `mud.rs` → `MUD.cc` (MUD definitions with name/hostname/port/inheritance; MudList collection; alias/action/macro storage).
- `telnet.rs` → `Telnet.cc` (IAC parsing, SB handling; Toy 8 patterns).
- `mccp.rs` → `Mccp.cc` (decompressor trait + flate2 inflate; gated by `mccp` feature; Toy 8 patterns).
- `scrollback.rs` → Scrollback/ring buffer (from `OutputWindow.cc` + Toy 10 patterns).

## UI Layer (Tier 3)

- `curses.rs` → `Curses.cc` (minimal ncurses wrapper; terminfo/ACS; Toy 2 patterns).
- `screen.rs` → `Screen.cc` (renderer + scroll region planner; Toy 7 patterns).
- `window.rs` → `Window.cc` (base widget).
- `output_window.rs` → `OutputWindow.cc` (rendering and color attrs).
- `input_line.rs` → `InputLine.cc` (line editor basics).
- `status_line.rs` → `StatusLine.cc` (status UI stripe).
- `selection.rs` → `Selection.cc` (base scrollable list widget; arrow navigation, letter jump).
- `mud_selection.rs` → `MUDSelection` class from `Selection.cc` (specialized MUD connect menu).

## Logic Layer (Tier 4)

- `session.rs` → `Session.cc` (pipeline MCCP→Telnet→ANSI→Scrollback).
- `engine.rs` → Headless engine (no strict C++ analog; extraction from `main.cc` event loop).
- `control.rs` → New (Unix domain control server; headless/attach support).
- `alias.rs` → `Alias.cc` (text expansion with %N parameters; wired into input pipeline).
- `action.rs` → `Action.cc` (triggers/replacements/gags with regex; wired into output pipeline).
- `macro_def.rs` → `Hotkey.cc` (keyboard shortcuts; wired into key handling).

## Plugins (Tier 5)

See `plugins/CODE_MAP.md` for detailed documentation.

- `plugins/stack.rs` → Stacked interpreter (consolidates `StackedInterpreter` behavior from C++; Toy 11).
- `plugins/python.rs` → `plugins/PythonEmbeddedInterpreter.cc` (pyo3; feature-gated; Toy 4).
- `plugins/perl.rs` → `plugins/PerlEmbeddedInterpreter.cc` (raw FFI; feature-gated; Toy 5).

## Known Gaps (Optional for MVP)

1. **Selectable trait implementations** (src/selectable.rs:10-13)
   - Trait defined but no types implement it yet
   - C++ uses `Selectable::select()` pattern (main.cc:147)
   - **Workaround**: Using direct `poll_fds()` in main.rs (simpler, works fine)
   - Not blocking MVP - can refactor later if needed

2. **Widget idle() methods** (Session, StatusLine, etc.)
   - C++ has idle() for time-based updates (connection timeout, message expiry)
   - Event loop has placeholders (main.rs:242-246)
   - **Deferred**: Not critical for basic functionality
   - Can implement when needed (e.g., Session::idle for connection timeout)

3. **Global state pattern** (src/globals.rs:1-9)
   - Toy 3 pattern validated but not applied
   - **Workaround**: Using locals in main.rs (simpler, works fine)
   - Optional for MVP (can refactor later if multi-module access needed)

## Tests

- Unit tests are colocated via `mod tests` in each file.
- Integration tests under `tests/` include control server JSON-lines behavior.

## Notes

- Prefer Rust `String`/`Vec<u8>` over custom classes unless fidelity reduces complexity.
- Unsafe/FFI localized to termcaps, sockets, and interpreters.
