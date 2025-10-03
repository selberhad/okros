# MCL Rust Port — Discovery Phase 2 (Risk-Focused Toys)

## Objective
De-risk key C++→Rust pitfalls found during reference review without pulling in full app complexity. Keep toys small, single-purpose, and fast to iterate.

Principle: Validate behaviors and edge cases first; codify patterns we can lift directly into `src/` during Execution.

## Scope Analysis
Risky subsystems to validate:
- TTY raw mode + terminfo key decode (ALT/meta, keypad app mode)
- Byte-oriented canvas diff + ANSI/ACS rendering (no curses windows)
- Telnet IAC + MCCP decompression across fragmented reads
- Nonblocking connect() + EINPROGRESS and readiness transitions
- Scrollback ring buffer + viewport/highlight overlay
- (Optional) Stacked interpreter chaining parity

---

## Toy Sequence

### Toy 6: TTY + Keys
C++ Reference: `TTY.cc`, `Curses.cc` (tigetstr)
- Objective: Raw mode, "\e="/"\e>" keypad app mode, terminfo lookup, escape parsing.
- What to learn: Normalize key events to stable codes; ALT/meta edge cases.
- SPEC focus: termios flags, tigetstr vs hardcoded fallbacks, buffer size limits.
- Success: Prints normalized key names for arrows, PgUp/Dn, Home/End, F-keys, ALT-letters.

### Toy 7: ANSI Canvas Diff
C++ Reference: `Window.cc`, `Screen.cc`, `Color.h`, `Curses.cc`
- Objective: `attrib`=u16 grid, color mapping, ACS enable/disable, diff frames to ANSI.
- What to learn: Efficient diffing; background updates; ACS toggling boundaries.
- SPEC focus: `getColorCode()` parity, 7-bit CSI, saved vs current color.
- Success: Visual diff fidelity across frames; minimal control codes when unchanged.

### Toy 8: Telnet + MCCP Fragmentation
C++ Reference: `Session.cc`, `mccpDecompress.c/h`
- Objective: Handle interleaved IAC sequences and compressed chunks splitting across reads.
- What to learn: Carry-over buffer management; prompt detection (GA/EOR); response emission.
- SPEC focus: Test vectors for split IAC; decompress boundaries; error handling.
- Success: Round-trips scripted inputs; emits correct responses; no state loss.

### Toy 9: Nonblocking Connect semantics
C++ Reference: `Socket.cc`
- Objective: Reproduce EINPROGRESS flow and readiness-driven establishment.
- What to learn: `getpeername`/`getsockname` checks; error path coverage.
- SPEC focus: Select/poll loop; reconnect edge cases.
- Success: Deterministic transitions and error texts match table.

### Toy 10: Scrollback Ring + Highlight
C++ Reference: `OutputWindow.cc`, `Window.cc`
- Objective: Ring movement, viewport paging, highlight overlay without corrupting source.
- What to learn: Copy-on-write overlay; border cases at buffer ends.
- SPEC focus: COPY_LINES behavior; sticky status messaging.
- Success: Paging and highlighting behave as reference without artifacts.

### (Optional) Toy 11: Plugins Stack
C++ Reference: `Embedded.cc`
- Objective: Chain interpreters and pass transformed buffers.
- Success: run/run_quietly order and enable/disable parity.

---

## Toy Development Workflow
- LEARNINGS.md goals → iterate implementation/tests → finalize findings → template code/FFI patterns for `src/`.
- Keep each toy standalone (no network unless required); prefer deterministic inputs.

## Current Status
- [ ] Toy 6 (TTY + Keys)
- [ ] Toy 7 (ANSI Canvas Diff)
- [ ] Toy 8 (Telnet + MCCP)
- [ ] Toy 9 (Nonblocking Connect)
- [ ] Toy 10 (Scrollback Ring)
- [ ] Toy 11 (Plugins Stack) — optional

## Estimated Effort
- Toy 6,7: 0.5–1 day each
- Toy 8,9,10: 1–2 days each
- Toy 11: 0.5 day (if needed)

