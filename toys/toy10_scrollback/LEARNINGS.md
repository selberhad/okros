# Toy 10 — Scrollback Ring + Highlight: LEARNINGS

## Step 0 — Learning Goals
- Ring memory: Model scrollback as contiguous buffer with moving `canvas`/`viewpoint`; validate COPY_LINES‑style compaction.
- Viewport movement: Implement page/line/home navigation with correct clamping and status messaging.
- Highlight overlay: Temporary color swap for matches without corrupting source; restore after copy.
- Performance: Ensure page up/down and redraws remain O(visible) not O(history).
- Integration points: Coordinate with parent copy and cursor placement; ensure no artifacts at edges.
- Save/export: Optional—emit ANSI with minimal color churn when dumping scrollback.

