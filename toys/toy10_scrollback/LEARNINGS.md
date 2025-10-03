# Toy 10 — Scrollback Ring + Highlight: LEARNINGS

## Step 0 — Learning Goals
- Ring memory: Model scrollback as contiguous buffer with moving `canvas`/`viewpoint`; validate COPY_LINES‑style compaction.
- Viewport movement: Implement page/line/home navigation with correct clamping and status messaging.
- Highlight overlay: Temporary color swap for matches without corrupting source; restore after copy.
- Performance: Ensure page up/down and redraws remain O(visible) not O(history).
- Integration points: Coordinate with parent copy and cursor placement; ensure no artifacts at edges.
- Save/export: Optional—emit ANSI with minimal color churn when dumping scrollback.

## Findings (Tests & Parity)

- Ring buffer model works with a large compaction block; adopted `COPY_LINES = 250` (capped by `lines - height`) similar to the reference.
- Printing a short line clears the tail cells to spaces (with the current color), matching reference behavior.
- Default viewing follows the tail; additional prints keep the latest lines visible unless frozen.
- Freeze semantics implemented: when frozen, `viewpoint` does not auto-follow new prints.
- Viewpoint movement clamps correctly for both page and single-line moves; mixed movement sequences preserve invariants.
- Highlight view swaps fg/bg over the requested span and clips to viewport bounds.

## Open Questions

- COPY_LINES size: Adopted 250 as default; revisit if performance characteristics suggest tuning per terminal/screen size.
- Freeze/follow semantics: Reference toggles “frozen” mode via scrollback controller. Define the exact follow-tail rules during long prints.
- Search integration: Reference supports case-insensitive search with highlight and navigation. When we port, where should this logic live (OutputWindow vs. higher layer)?
- Save/export semantics: For `#save -c`, ensure window diff/ANSI output aligns with Screen’s color handling to avoid redundant sequences.
