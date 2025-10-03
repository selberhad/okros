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

### Toy 12: Internal MUD (Testing Infrastructure)
C++ Reference: None (okros innovation)
- Objective: Single-player text adventure for e2e testing without external MUD server.
- What to learn: Can we test full pipeline (ANSI → Session → Scrollback) with zero deps?
- SPEC focus: Deterministic command sequences; headless mode via control server; LLM automation.
- Success: Automated tests via JSON API; perfect for CI/CD and bot validation.
- Extension: Headless mode integration (control server drives internal MUD).

---

## Toy Development Workflow
- LEARNINGS.md goals → iterate implementation/tests → finalize findings → template code/FFI patterns for `src/`.
- Keep each toy standalone (no network unless required); prefer deterministic inputs.

## Current Status (All Complete) ✅
- [x] Toy 6 (TTY + Keys) — COMPLETE: Raw mode, keypad app mode, key normalization
- [x] Toy 7 (ANSI Canvas Diff) — COMPLETE: Diff+ACS tests, scroll-region optimization
- [x] Toy 8 (Telnet + MCCP) — COMPLETE: Telnet pipeline, real MCCP inflate, ANSI parser
- [x] Toy 9 (Nonblocking Connect) — COMPLETE: EINPROGRESS flow, socket state machine
- [x] Toy 10 (Scrollback Ring) — COMPLETE: Ring/highlight/navigation, freeze+COPY_LINES
- [x] Toy 11 (Plugins Stack) — COMPLETE: Chaining/enable/disable/quiet parity
- [x] Toy 12 (Internal MUD) — COMPLETE: E2E testing infrastructure, headless automation

**Discovery Phase 2: COMPLETE!** All 12 toys validated (11 from plan + 1 testing innovation)

## Extensions Completed ✅
- ✅ Toy 8: Real MCCP inflate with flate2, v1/v2 handshakes, streaming tests
- ✅ Toy 8: ANSI SGR → attrib converter (SGR 0/1, 30-37/40-47, bright 90-97/100-107)
- ✅ Toy 7: Scroll-region optimization planner with heuristic thresholds
- ✅ Toy 12: Internal MUD for comprehensive e2e testing without external deps

**All discovery work complete** - ready for production porting in src/

## Estimated Effort
- Toy 6,7: 0.5–1 day each
- Toy 8,9,10: 1–2 days each
- Toy 11: 0.5 day (if needed)
