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
 - (Optional) ANSI SGR → attrib parser (colorConverter parity)

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
 - Extension: Scroll-region optimization planner (set region, goto bottom, N newlines) when large diffs in region are detected.

### Toy 8: Telnet + MCCP Fragmentation
C++ Reference: `Session.cc`, `mccpDecompress.c/h`
- Objective: Handle interleaved IAC sequences and compressed chunks splitting across reads.
- What to learn: Carry-over buffer management; prompt detection (GA/EOR); response emission.
- SPEC focus: Test vectors for split IAC; decompress boundaries; error handling.
- Success: Round-trips scripted inputs; emits correct responses; no state loss.
 - Extension: Real MCCP inflate (flate2/miniz) implementing `Decompressor` with v1/v2 handshakes, error/EOS, stats. ANSI SGR → attrib converter integrated for Session color parsing (supports 0/1, 30–37/40–47, and bright 90–97/100–107).

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
 - Extension: Freeze/follow semantics; COPY_LINES block tuned to 250 (capped by buffer).

### (Optional) Toy 11: Plugins Stack
C++ Reference: `Embedded.cc`
- Objective: Chain interpreters and pass transformed buffers.
- Success: run/run_quietly order and enable/disable parity.

---

## Toy Development Workflow
- LEARNINGS.md goals → iterate implementation/tests → finalize findings → template code/FFI patterns for `src/`.
- Keep each toy standalone (no network unless required); prefer deterministic inputs.

## Current Status
- [ ] Toy 6 (TTY + Keys) — SPEC/LEARNINGS staged
- [x] Toy 7 (ANSI Canvas Diff) — base diff+ACS tests complete; extension planned: scroll-region
- [x] Toy 8 (Telnet + MCCP) — telnet pipeline tests complete; extension planned: real MCCP inflate
- [ ] Toy 9 (Nonblocking Connect) — SPEC/LEARNINGS staged
- [x] Toy 10 (Scrollback Ring) — ring/highlight/navigation tests complete; freeze+COPY_LINES tuned
- [x] Toy 11 (Plugins Stack) — chaining/enable/quiet tests complete (optional)

## Next Steps (Extensions)
- Implement Toy 8 real MCCP inflate behind `real_mccp` feature; add streaming inflate tests and EOS/error cases. Add ANSI SGR → attrib converter (basic SGR + bright variants) with fragmentation and reset tests.
- Add Toy 7 scroll-region optimization planner + tests; integrate heuristic thresholds (diff ratio, region bounds).
- Optional new toys (if needed):
  - ANSI SGR → attrib parser (colorConverter parity): SGR runs, resets, malformed/incomplete sequences across chunks.
  - Regex triggers & replacement: case-insensitive patterns, gagging, replacement length changes without losing color.
  - InputLine editor: key-to-state machine using Toy 6 key codes; history/prompt handling.

## Estimated Effort
- Toy 6,7: 0.5–1 day each
- Toy 8,9,10: 1–2 days each
- Toy 11: 0.5 day (if needed)
