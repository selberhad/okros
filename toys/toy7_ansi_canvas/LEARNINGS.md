# Toy 7 — ANSI Canvas Diff: LEARNINGS

## Step 0 — Learning Goals
- Attrib model: Represent `attrib = (color<<8)|byte` in Rust; keep byte‑oriented rendering (avoid unintended UTF‑8).
- Color parity: Reproduce `getColorCode()` mapping, including reverse color table and bold/background behavior; decide on 7‑bit CSI.
- ACS toggling: When to emit `smacs`/`rmacs` around special chars; ensure minimal churn and correct teardown.
- Diff algorithm: Efficiently detect and emit only changed cells; cursor positioning heuristics vs direct VT_GOTO.
- Saved color: Cache last emitted color; confirm behavior for fg_white|bg_black edge case.
- Scrolling region: Decide whether to prototype scroll optimization now or defer; validate basic full‑frame diff first.
- Cursor finalization: Restore cursor to logical position and ensure ACS state reset at frame end.

## Findings (Tests & Parity)

- Control chars (<32) render as spaces; byte‑oriented rendering confirmed.
- Color mapping/parity matches reference, including white-on-black collapsing to `CSI 0m` and bold emitting `CSI 1;...m`.
- Saved color behaves as expected: single color code reused for adjacent and separated writes across rows.
- Bottom-right cell is intentionally skipped; no writes or ACS toggles when only that cell changes.
- ACS toggling: `smacs` begins runs; `rmacs` emitted on transition back to normal and guaranteed at frame end if enabled.
- Cursor finalization precedes ACS reset when both occur at the end of a frame.
- Minimal cursoring: adjacent writes and simple line-wrapping avoid redundant `VT_GOTO` sequences.

## Open Questions

- Scroll-region optimization: Implement reference-style scroll region and newline optimization or keep simpler VT_GOTO-only diff?
- Background emission: Always emit bg like the reference, or optimize to omit bg when unchanged (we prototyped `set_bg_always=false`). Which path for parity/perf?
- ACS glyphs: Replace toy `#` placeholder with real `real_special_chars` mapping; validate terminals that differ in ACS handling.
- Non-ASCII policy: Reference is 8-bit; clarify behavior for UTF‑8 input in the ported code (pass-through vs. sanitize).
