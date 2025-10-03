# Toy 10 — Scrollback Ring + Highlight (SPEC)

## Objective
Model MCL’s OutputWindow behavior: a ring-buffer-backed scrollback with an active viewport, page/line navigation, and temporary highlight overlay, matching `OutputWindow.cc` and `Window.cc` semantics.

## Scope
- Memory layout: Single contiguous `attrib` buffer sized `width * scrollback_lines`; `canvas` and `viewpoint` as moving windows; `top_line` tracking.
- Scrolling: On print beyond bottom, advance `canvas` (or compact via COPY_LINES when buffer end reached) and clear bottom line.
- Viewport movement: Page up/down, line up/down, home; clamp to `[scrollback..canvas]`; sticky status messaging.
- Highlight: Temporarily invert/modify color for a span without mutating backing store; restore after copy to parent.

## Inputs
- Rendered lines appended via print interface; search invocation with a string and direction; navigation keys.
- Config: `opt_scrollback_lines`, window width/height.

## Outputs
- Parent copy calls with the current viewport; status messages for scroll state; highlight visual in the copied region.

## Behavior
- Ring end: When `canvas` reaches `scrollback+(lines-height)*width`, memmove up by `COPY_LINES*width`, decrement pointers, clear the newly freed tail.
- Navigation: Adjust `viewpoint` by ±line or ±page (height/2), clamp; set sticky status to show position; exit on end.
- Highlight: For visible match, copy range, swap fg/bg (mask out bold/blink), copy to parent, restore original cells.

## Success Criteria
- Printing and scrolling preserve full history up to capacity; viewport navigation correct at boundaries.
- Highlight overlay does not corrupt backing store; appears only when in view.
- COPY_LINES compaction retains expected relative positions and performance remains acceptable.

## Test Plan
- Unit: Small buffer (e.g., width 10, 50 lines) with controlled prints; assert viewport slices after moves.
- Highlight: Search hit near edges and across lines; confirm overlay and restore.
- Compaction: Force several COPY_LINES events and validate content continuity and `top_line` updates.

## Non-Goals
- Keyboard integration; rely on direct function calls to move viewport and trigger search/highlight.

