# Toy 7 — ANSI Canvas Diff: LEARNINGS

## Step 0 — Learning Goals
- Attrib model: Represent `attrib = (color<<8)|byte` in Rust; keep byte‑oriented rendering (avoid unintended UTF‑8).
- Color parity: Reproduce `getColorCode()` mapping, including reverse color table and bold/background behavior; decide on 7‑bit CSI.
- ACS toggling: When to emit `smacs`/`rmacs` around special chars; ensure minimal churn and correct teardown.
- Diff algorithm: Efficiently detect and emit only changed cells; cursor positioning heuristics vs direct VT_GOTO.
- Saved color: Cache last emitted color; confirm behavior for fg_white|bg_black edge case.
- Scrolling region: Decide whether to prototype scroll optimization now or defer; validate basic full‑frame diff first.
- Cursor finalization: Restore cursor to logical position and ensure ACS state reset at frame end.

