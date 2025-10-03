# CODE MAP — src/

Brief index mapping Rust modules to their C++ counterparts (when applicable). Update before structural commits.

- `ansi.rs` → ANSI SGR/attrib conversion (from rendering logic in `OutputWindow.cc`/`Screen.cc`).
- `telnet.rs` → `Telnet.cc` (IAC parsing, SB handling).
- `mccp.rs` → `Mccp.cc` (decompressor; v1 path implemented, gated by `mccp` feature).
- `scrollback.rs` → Scrollback/ring buffer (from `OutputWindow.cc` + helpers).
- `selectable.rs` → `Selectable.cc` (trait) and `Selection.cc` helpers.
- `select.rs` → poll wrapper analogous to `Selection.cc`.
- `socket.rs` → `Socket.cc` (nonblocking IPv4 socket over raw fd).
- `tty.rs` → `TTY.cc` (raw mode + keypad app mode; Toy 2 patterns).
- `config.rs` → `Config.cc` (basic config; loopback helpers for tests).
- `mud.rs` → `MUD.cc` (socket/config wiring; small helper tests).
- `session.rs` → `Session.cc` (pipeline MCCP→Telnet→ANSI→Scrollback).
- `engine.rs` → Headless engine (no strict C++ analog; extraction from `main.cc` event loop).
- `control.rs` → New (Unix domain control server; headless/attach support).
- `screen.rs` → `Screen.cc` (renderer + scroll region planner).
- `window.rs` → `Window.cc` (base widget).
- `output_window.rs` → `OutputWindow.cc` (rendering and color attrs).
- `input_line.rs` → `InputLine.cc` (line editor basics).
- `status_line.rs` → `StatusLine.cc` (status UI stripe).

Tests
- Unit tests are colocated via `mod tests` in each file.
- Integration tests under `tests/` include control server JSON-lines behavior.

Notes
- Prefer Rust `String`/`Vec<u8>` over custom classes unless fidelity reduces complexity.
- Unsafe/FFI localized to termcaps, sockets, and interpreters.
