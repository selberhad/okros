# Toy 7 — ANSI Canvas Diff (SPEC)

## Objective
Reproduce MCL’s byte-oriented canvas rendering: maintain an `attrib` grid `(color<<8)|byte`, compute diffs vs. previous frame, and emit minimal ANSI + ACS sequences equivalent to `Screen.cc`/`Curses.cc` behavior.

## Scope
- Data model: `u16` attrib, width/height, current cursor, `last_screen` snapshot, `clearLine`.
- Color mapping: Reverse color table, bold bit, `getColorCode()` parity, 7-bit CSI (`\e[`), fg/bg handling.
- ACS handling: Enable/disable alternate char set using `smacs`/`rmacs` around special chars (SC_BASE..SC_END).
- Diff algorithm: Detect changed cells, choose between printing neighbor char vs VT_GOTO, update `last_screen`.
- Cursor finalization: Move to logical cursor (1-based VT pos).

## Inputs
- Two frames (prev, next): arrays of `attrib` size `w*h`.
- Logical cursor `(x,y)` for final placement.
- Flags: setBackground always vs. smart bg updates (match reference default: setBackground=true).

## Outputs
- Byte stream to stdout: ANSI codes + printable bytes; emits `rmacs` if ACS left enabled.
- Counters (optional): control chars emitted, written bytes (for sanity checks).

## Behavior
- Color: Always emit full color sequence when color changes; special-case fg_white|bg_black|!bold to CSI "0m".
- Goto: Prefer VT_GOTO when skipping across cells unless printing the immediately previous char (same color) is cheaper.
- ACS: On seeing SC_* byte, ensure `smacs` sent; return to normal with `rmacs` before non-ACS; ensure final `rmacs`.
- Diff: Skip bottom-right cell; home first or not needed if first change moves cursor (match reference: VT_HOME at start).
- Scrolling region: Out of scope for minimal spec; can be validated in a follow-up.

## Success Criteria
- Identical output for representative frame transitions from C++ (golden samples) including color toggles and ACS borders.
- Minimal control codes when no changes; stable cursor placement.
- No lingering ACS mode at frame end; `last_screen` updated to match canvas.

## Test Plan
- Unit: Small 4x3 frames; assert ANSI output vs. expected strings (color changes, single-char updates, line wrap).
- ACS: Border-only changes toggle `smacs`/`rmacs` correctly; mixing ASCII/ACS characters.
- Color edge: fg_white|bg_black no-bolding produces CSI "0m"; bold toggles.
- Idempotence: Rendering the same frame twice produces only cursor goto to final pos (or nothing if already there).

## Non-Goals
- Scrolling region optimization and `/dev/vcsa` path (covered by later toy or main port).
- UTF‑8 or wide-char handling (model stays 8-bit like reference).

