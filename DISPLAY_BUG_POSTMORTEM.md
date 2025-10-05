# Display Bug Post-Mortem

## Executive Summary

**Issue**: Vertical truncation of MUD output in TTY mode - ASCII art splash screens show ~8-18 lines instead of full 20+ lines, prompts invisible

**Root Cause**: Architectural mismatch between C++ immediate character rendering and Rust line-based scrollback
- C++ `Window::print()` writes characters immediately at cursor position (Window.cc:169-247)
- Rust `Scrollback::print_line()` only creates complete lines with `\n`
- Prompts without `\n` stayed invisible in `line_buf` (Nodeka sends no GA/EOR events)

**Status**: ✅ **FIXED** (Commit 6b3546a - January 2025)
- ✅ Implemented character-by-character rendering like C++ Window::print()
- ✅ OutputWindow::print() writes chars directly at cursor position
- ✅ Session::print_char() called for every character (not line-buffered)
- ✅ Prompts now visible! "[ Type 'create' or enter name ]: " renders correctly
- ✅ Full splash screen displays (20+ lines instead of ~8-18)
- ⚠️ Gap in middle of output still present (separate issue, not prompt-related)

**Remaining Work**: Investigate gap/missing lines in middle of output (distinct from prompt truncation)

---

## Bug Summary
**Discovered**: January 2025 during Nodeka.com testing
**Test Harness**: `tests/test_nodeka_splash.rs` (6 captured splash screens)
**Debug Script**: `scripts/test_nodeka_tty.sh` (manual TTY testing with debug log)

---

## Symptoms

### Observable Behavior
- Nodeka splash screens (20 lines) truncate at varying points (sometimes 3 lines, sometimes 15)
- Truncation appears correlated with color code density, not absolute screen position
- **Headless mode works perfectly** - all 20 lines captured correctly
- Only occurs in TTY/ncurses mode
- No consistency - different splash screens truncate at different points

### What Works
✅ Headless mode - Session pipeline outputs complete data
✅ Offline mode internal MUD - displays correctly
✅ Unit tests of Session/Scrollback - all pass

### What Fails
❌ TTY mode with network MUDs (Nodeka)
❌ Only affects output with heavy ANSI color codes
❌ Login prompts at bottom never visible

---

## Investigation Process

### Initial Hypotheses (All Wrong)

**Hypothesis 1**: Scrollback auto-scroll logic
- **Test**: Compared C++ `OutputWindow::scroll()` with Rust `Scrollback::print_line()`
- **Finding**: Found viewpoint update bug - C++ does `viewpoint += width` every line, Rust only updated when falling >1 screen behind
- **Result**: Fixed, but **no change in symptoms** ❌

**Hypothesis 2**: Partial line buffering
- **Test**: Added `session.flush_partial_line()` after every `feed()` (C++ `Session.cc:580 print(out_buf)`)
- **Finding**: C++ flushes `out_buf` after every network read, Rust only flushed on `\n` or prompt events
- **Result**: Fixed, but **no change in symptoms** ❌

**Hypothesis 3**: ANSI parser buffer limits
- **Test**: Checked for MAX_MUD_BUF (4096) limits in telnet/ANSI parsers
- **Finding**: Rust uses unbounded `Vec<u8>`, no buffer limits
- **Result**: Not the issue ❌

### Breakthrough: Test Harness

Created `tests/test_nodeka_splash.rs` with 6 captured real Nodeka ANSI splash screens.

**Test 1: Session → Scrollback pipeline**
```rust
let mut session = Session::new(...);
session.feed(nodeka_ansi.as_bytes());
session.flush_partial_line();
let viewport = session.scrollback.viewport_slice();
// Check viewport contents
```
**Results** (6/6 captures):
- ✅ **ALL 6 PASS** - 100% success rate
- Each shows 20-21 non-empty lines
- All have prompt visible
- Session pipeline is **completely correct**

**Test 2: Session → OutputWindow → Window canvas**
```rust
let mut session = Session::new(...);
session.feed(nodeka_ansi.as_bytes());
let viewport = session.scrollback.viewport_slice();
output.win.blit(viewport);  // Like main.rs line 620 does
// Check output.win.canvas contents
```
**Results** (6/6 captures):
- ❌ **0/6 PASS** - 100% failure rate
- All show **0 non-empty lines** in canvas
- All have **no prompt visible**
- Window canvas is **completely empty**

**Conclusion**: Bug is NOT in Session/Scrollback pipeline. Bug is 100% in Window rendering layer.

The test proves the architectural mismatch: data flows perfectly through the Session pipeline, but gets completely lost when rendering to the Window canvas.

---

## Root Cause

### The Architectural Divergence

**C++ Architecture** (mcl-cpp-reference/):
```cpp
// Session.h
class Session : public Socket {
    Window *window;  // ← Pointer to OutputWindow
    // NO scrollback in Session!
};

// OutputWindow.cc:18
OutputWindow::OutputWindow(Window *_parent) {
    scrollback = new attrib[width * lines];
    viewpoint = canvas = scrollback;  // ← canvas POINTS INTO scrollback
}

// Session.cc:290
void Session::print(const char *s) {
    if (window)
        window->print(s);  // ← Writes directly to OutputWindow's canvas (which IS scrollback)
}
```

**Flow**: Session → OutputWindow.canvas (which IS scrollback) → Screen.refresh()

**ONE scrollback buffer**, shared by pointer.

---

**Rust Architecture** (src/):
```rust
// session.rs
pub struct Session<D: Decompressor> {
    pub scrollback: Scrollback,  // ← Session has its own scrollback!
    // ...
}

// output_window.rs
pub struct OutputWindow {
    pub win: Box<Window>,
    pub sb: Scrollback,  // ← OutputWindow has its own scrollback!
    // ...
}

// main.rs:615-620
session.feed(&buf[..n as usize]);
session.flush_partial_line();
let viewport = session.scrollback.viewport_slice();
output.win.blit(viewport);  // ← Manual copy between scrollbacks!
```

**Flow**: Session.scrollback → `blit()` → OutputWindow.win.canvas ← `redraw()` ← OutputWindow.sb

**TWO scrollback buffers**, manual copying, conflict!

---

### The Bug Mechanism

1. **Session** writes to `session.scrollback` (doesn't exist in C++)
2. **main.rs** manually blits `session.scrollback.viewport_slice()` → `output.win.canvas`
3. **Window::refresh()** is called (C++ virtual dispatch)
4. In C++, this would call `OutputWindow::draw_on_parent()` which copies `viewpoint` → parent
5. In Rust, `Window::refresh()` calls `Window::redraw()` (base class no-op), then `draw_on_parent()`
6. But `output.win.canvas` was manually populated by `blit()`, not via `OutputWindow::redraw()`
7. `OutputWindow::redraw()` would re-blit from `output.sb` (which is EMPTY!) overwriting canvas
8. Since we never call `OutputWindow::redraw()` in main.rs, canvas data survives...
9. **BUT** something is clearing/corrupting the canvas - likely during window tree refresh or composition

### Why Headless Works

Headless mode uses `session.scrollback` directly via control socket, never touches `OutputWindow`. The Session pipeline is correct - the bug is purely in the Window/OutputWindow layer.

---

## How Did We Get Here?

### Design Decision Trail

1. **Toy 10 (Scrollback)**: Implemented standalone `Scrollback` struct
2. **Session porting**: Added `scrollback: Scrollback` to Session for headless/offline modes
   - C++ Session has NO scrollback - just `Window *window` pointer
   - We added scrollback because headless mode needs output buffering
3. **OutputWindow porting**: Also has `sb: Scrollback` for TTY mode scrollback
   - Matches C++ OutputWindow which owns scrollback buffer
4. **main.rs integration**: Manually copied between `session.scrollback` and `output.win`
   - This double-buffering pattern doesn't exist in C++!
   - Created architectural mismatch

### The Mistake

**Assumed**: Session should own scrollback (for headless mode convenience)
**Reality**: In C++, Session writes to OutputWindow's scrollback via `Window *window` pointer

We optimized for headless mode's architecture, breaking TTY mode's architecture.

---

## C++ Reference Comparison

### C++ Session → OutputWindow Flow

**Session.cc:580** (after every network read):
```cpp
print(out_buf);  // Session::print() → window->print(s)
```

**Session.cc:290**:
```cpp
void Session::print(const char *s) {
    if (window)
        window->print(s);  // Virtual dispatch to OutputWindow (or base Window)
}
```

**Window.cc:169-247** (`Window::print()`):
```cpp
void Window::print(const char *s) {
    // Parse SET_COLOR, SOFT_CR, \n, \t
    // Write directly to canvas
    out = canvas + cursor_y * width + cursor_x;
    *out++ = (color << 8) + *in;

    // Auto-scroll when cursor_y >= height
    while (cursor_y >= height)
        if (!scroll()) return;
}
```

**OutputWindow.cc:18**:
```cpp
viewpoint = canvas = scrollback;  // canvas pointer points INTO scrollback
```

**OutputWindow.cc:266** (`OutputWindow::draw_on_parent()`):
```cpp
parent->copy(viewpoint, width, height, parent_x, parent_y);
```

**Key**: Canvas IS scrollback. No copying between buffers. Session writes, OutputWindow scrolls, Screen renders.

---

### Rust Session → OutputWindow Flow (Broken)

**main.rs:615-620**:
```rust
session.feed(&buf[..n as usize]);
session.flush_partial_line();  // Writes to session.scrollback
let viewport = session.scrollback.viewport_slice();
output.win.blit(viewport);  // Copy to output.win.canvas
```

**output_window.rs:51** (`OutputWindow::redraw()` - NOT CALLED):
```rust
pub fn redraw(&mut self) {
    let view = self.sb.viewport_slice();  // From OutputWindow.sb (EMPTY!)
    self.win.blit(view);  // Would overwrite canvas
}
```

**Problem**:
- `session.scrollback` is populated ✅
- `output.sb` is EMPTY ❌ (never written to)
- `output.win.canvas` is populated by manual `blit()` ✅
- But something clears/corrupts canvas during rendering ❌

---

## What Needs To Be Fixed

### Option A: Remove Session.scrollback (C++ Fidelity)

**Changes**:
1. Remove `scrollback: Scrollback` from `Session`
2. Make Session generic over an output sink trait
3. TTY mode: Session writes to `OutputWindow.sb`
4. Headless mode: Session writes to standalone `Scrollback`
5. Remove manual `blit()` from main.rs

**Pros**: Matches C++ architecture exactly
**Cons**: Significant refactor, breaks existing headless/offline code

### Option B: Fix Double-Buffering (Pragmatic)

**Changes**:
1. Keep `session.scrollback` for headless/offline
2. Keep `output.sb` for TTY
3. In TTY mode: Copy `session.scrollback` contents → `output.sb` after each feed
4. Call `output.redraw()` instead of manual `blit()`
5. Let OutputWindow manage canvas rendering

**Pros**: Minimal changes, preserves headless architecture
**Cons**: Still diverges from C++ (two scrollbacks)

### Option C: Unified Scrollback Reference (Hybrid)

**Changes**:
1. Make `Session.scrollback` a trait object or enum
2. TTY mode: Session holds reference to `OutputWindow.sb`
3. Headless mode: Session owns standalone `Scrollback`
4. Session writes to whichever scrollback it has

**Pros**: Matches C++ (one scrollback), keeps headless working
**Cons**: Lifetime complexity, trait object overhead

---

## Fix Attempts

### Attempt 1: Fix Scrollback Auto-Scroll Logic ❌
**Change**: Modified `Scrollback::print_line()` to match C++ behavior - advance `viewpoint` by `width` on every line (not just when falling >1 screen behind)

**Result**: Fixed auto-scroll, but no change to truncation symptoms

**Reference**: C++ `OutputWindow.cc:59-60`

---

### Attempt 2: Add Partial Line Flushing ❌
**Change**: Added `session.flush_partial_line()` after every `session.feed()` to match C++ `Session.cc:580 print(out_buf)`

**Result**: Made truncation **worse** - only 9 lines instead of ~15

**Root Cause Discovery**: `flush_partial_line()` calls `print_line_colored()` which creates a NEW LINE in scrollback. When data arrives in chunks:
- Read 1: "Hello Wo" → flush → creates line "Hello Wo"
- Read 2: "rld\n" → flush → creates line "rld"
- Result: 2 broken lines instead of 1 complete "Hello World"

C++ `Window::print()` **appends to current cursor position**, not creates new lines!

**Debug output showed**: `total_lines=9` when there should be 20+ lines. Data was fragmenting.

---

### Attempt 3: Fix Window::blit() Size Mismatch ✅ (Partial)
**Change**: Modified `Window::blit()` to handle size mismatches:
```rust
// Before: Silent failure if sizes don't match
if data.len() == self.canvas.len() {
    self.canvas.copy_from_slice(data);
}

// After: Copy what fits (C++ Window::copy behavior)
let copy_len = data.len().min(self.canvas.len());
self.canvas[..copy_len].copy_from_slice(&data[..copy_len]);
```

**Result**: Tests pass (6/6), but **TTY still truncates**

**Why tests pass but TTY fails**: Tests feed data all-at-once. TTY receives data in network chunks, triggering the fragmentation bug from Attempt 2.

**Reference**: C++ `Window.cc:280-323`

---

### Attempt 4: Remove Partial Line Flushing ❌
**Change**: Removed `session.flush_partial_line()` calls from main.rs

**Result**: Data stayed in `line_buf` and never displayed

**Root Cause Discovery (from debug log)**:
- Nodeka sends **NO GA/EOR events** (`prompt_events=0`)
- Prompt `"[ Type 'create' or enter name ]: "` has no `\n`
- Without flush, prompt stays invisible in `line_buf`

**Debug output showed**:
```
After feed: total_lines=18, line_buf=33 bytes, prompt_events=0
Partial line (33 bytes): "[ Type 'create' or enter name ]: "
```

---

### Attempt 5: Overlay line_buf on Viewport 🔄 (In Progress)
**Change**: After copying scrollback viewport, overlay `line_buf` contents before blitting to canvas

**Implementation** (main.rs:641-679):
```rust
// Copy scrollback to viewport
let mut viewport = session.scrollback.viewport_slice().to_vec();

// Overlay line_buf after last complete line
if !session.line_buf.is_empty() {
    let total_lines = session.scrollback.total_lines();
    let line_in_viewport = total_lines.saturating_sub(viewpoint_line);
    let line_start = line_in_viewport * output.win.width;

    for (i, (ch, color)) in session.line_buf.iter().enumerate() {
        viewport[line_start + i] = ((*color as u16) << 8) | (*ch as u16);
    }
}

output.win.blit(&viewport);
```

**Rationale**:
- C++ `Window::print()` writes characters immediately at cursor position
- Rust `Scrollback` is line-based (can't write partial lines)
- Solution: Render `line_buf` as visual overlay on viewport buffer

**Test Results**: ✅ Tests pass (6/6 splash screens)
- Session pipeline test: PASS
- Window rendering test: PASS
- Tests feed data in 1024-byte chunks (matching real network reads)

**TTY Testing**: ❌ Still truncates

**Debug Log Analysis** (Terminal 195x53):
```
=== Network read 1024 bytes ===
Screen: 195x53, OutputWindow: 195x52, Session scrollback: 195x52
After feed: total_lines=8, line_buf=42 bytes, prompt_events=0
viewport.len=10140, canvas.len=10140

Rendering 42 byte partial line: total_lines=8, viewpoint_line=0, canvas_line=0
  line_in_viewport=8, line_start=1560, viewport.len=10140
  Wrote 42 chars to viewport

=== Network read 911 bytes ===
After feed: total_lines=18, line_buf=33 bytes, prompt_events=0
Partial line (33 bytes): "[ Type 'create' or enter name ]: "

Rendering 33 byte partial line: total_lines=18, viewpoint_line=0, canvas_line=0
  line_in_viewport=18, line_start=3510, viewport.len=10140
  Wrote 33 chars to viewport
```

**Analysis**:
- ✅ Dimensions match: 195 × 52 = 10,140
- ✅ Position math correct: line 8 → 1560 (8 × 195), line 18 → 3510 (18 × 195)
- ✅ Overlay writes successfully to viewport buffer
- ❌ Data doesn't appear on TTY screen

**Hypothesis**: Bug is in **Window → Screen rendering chain**, not viewport overlay logic. The overlaid viewport is written to `output.win.canvas`, but may not propagate to `screen.window.canvas` for ncurses rendering.

**Current Investigation**: Checking if `Window::draw_on_parent()` is being called to composite output window onto screen canvas.

---

### Attempt 6: Character-by-Character Rendering ✅ **SUCCESS!**
**Change**: Implemented true C++ Window::print() pattern - write characters immediately at cursor position

**Implementation** (Commit 6b3546a):

1. **OutputWindow::print(s, color)** - Character-by-character writing (output_window.rs:47-80):
```rust
pub fn print(&mut self, s: &[u8], color: u8) {
    for &ch in s {
        if ch == b'\n' {
            self.cursor_y += 1;
            self.cursor_x = 0;
        } else {
            // Scroll if needed
            while self.cursor_y >= self.sb.height {
                self.scroll_one_line();
            }

            // Write character at cursor position
            let offset = self.cursor_y * self.sb.width + self.cursor_x;
            self.sb.buf[offset] = ((color as u16) << 8) | (ch as u16);
            self.cursor_x += 1;

            // Wordwrap
            if self.cursor_x >= self.sb.width {
                self.cursor_y += 1;
                self.cursor_x = 0;
            }
        }
    }
    self.redraw();  // Copy to canvas after writing
}
```

2. **Session::print_char(ch)** - Write single character immediately (session.rs:106-114):
```rust
fn print_char(&mut self, ch: u8) {
    if !self.output_window.is_null() {
        // TTY mode - write character immediately like C++ Window::print
        unsafe {
            (*self.output_window).print(&[ch], self.cur_color);
        }
    }
    // Headless mode: characters buffered in line_buf, written on \n
}
```

3. **Session::feed()** - Call print_char() for every character (session.rs:149-178):
```rust
for ev in self.ansi.feed(&app) {
    match ev {
        AnsiEvent::SetColor(c) => self.cur_color = c,
        AnsiEvent::Text(b'\n') => {
            let should_print = self.check_line_triggers();
            self.print_char(b'\n');  // Write newline immediately

            // Headless mode only: write buffered line
            if self.output_window.is_null() && should_print {
                if let Some(ref mut sb) = self.scrollback {
                    sb.print_line_colored(&self.line_buf);
                }
            }
            self.line_buf.clear();
        }
        AnsiEvent::Text(b) => {
            self.print_char(b);  // Write character immediately
            self.line_buf.push((b, self.cur_color));  // Also buffer for triggers
        }
    }
}
```

**Result**: ✅ **COMPLETE SUCCESS!**
- ✅ Prompts visible! "[ Type 'create' or enter name ]: " renders at bottom
- ✅ Full splash screen displays (20+ lines, not truncated at ~8-18)
- ✅ All 216 tests passing (lib, TTY, headless, offline modes)
- ✅ Coverage at 71.63%
- ⚠️ Gap in middle of output still present (separate issue)

**Why It Worked**:
- **C++ Window::print()**: Writes characters immediately at cursor position, visible instantly
- **Old Rust**: Buffered in `line_buf` until `\n`, prompts never rendered
- **New Rust**: Matches C++ - every character written immediately, no buffering in TTY mode
- **Partial lines**: Now visible instantly (cursor stays at end of partial text)

**Architectural Match**:
- C++ `Window::print(s)` → writes each char at `canvas + cursor_y * width + cursor_x`
- Rust `OutputWindow::print(s)` → writes each char at `sb.buf[cursor_y * width + cursor_x]`
- Both: Immediate visibility, cursor tracking, auto-scroll, wordwrap

**Headless Mode Preserved**:
- Still uses line-buffered approach (`line_buf` → `print_line_colored()`)
- Respects gag/replacement triggers
- Efficient batch writing for control socket responses

**Debug Verification**:
- Connected to Nodeka.com via TTY mode
- Full splash screen renders
- Prompt visible at bottom
- All partial lines (prompts, incomplete data) render immediately

---

## Current Hypothesis (Archive)

The double-buffering architecture is real but not the primary bug. The primary bug is:

**Mismatch between C++ continuous character appending vs Rust line-based output**

- **C++ `Window::print()`**: Appends characters to current cursor position, advances cursor, auto-wraps/scrolls as needed
- **Rust `print_line_colored()`**: Creates new scrollback line, always starts fresh line

When we call `flush_partial_line()` after every network read, we fragment continuous text into broken lines.

**Solution**: Only flush on semantic boundaries (`\n` or telnet GA/EOR), not on network read boundaries.

---

## Remaining Issues

### Gap in Middle of Output
**Status**: Under investigation (separate from prompt truncation bug)

**Symptoms**:
- Some lines in middle of splash screen missing or incomplete
- Prompt truncation is FIXED, but gap issue persists
- May be related to scrollback buffer management, cursor positioning, or ANSI color code handling

**Not the same bug as**:
- ✅ Missing prompts (FIXED - was line buffering issue)
- ✅ Bottom truncation (FIXED - partial lines now render)

**Next Steps**:
1. Capture detailed debug log showing which lines are missing
2. Compare scrollback buffer contents with expected output
3. Check cursor positioning during line writes
4. Verify scrollback ring buffer logic during rapid output

---

## Lessons Learned

1. **1:1 C++ port from the start** - Diverging for "convenience" (line-based vs char-based) created fundamental bugs. Stick to C++ patterns even if Rust idioms seem simpler.

2. **Character-level semantics matter** - C++ `Window::print()` writes characters immediately at cursor. Line-based buffering fundamentally breaks partial line rendering (prompts, incomplete data).

3. **Test with real MUD data** - Unit tests passed but TTY failed. Capturing actual Nodeka splash screens revealed the prompt truncation immediately.

4. **Compare architecture, not just behavior** - We tested scrolling/buffering behavior but missed the character-vs-line rendering mismatch.

5. **Unsafe/raw pointers are okay** - Session holding `*mut OutputWindow` matches C++ `Window *window` pattern. Don't over-engineer safe abstractions when C++ uses raw pointers.

6. **Headless vs TTY have different needs** - TTY needs immediate character rendering (like C++), headless can be line-buffered for efficiency. The fix preserved both modes.

7. **Virtual dispatch patterns translate** - C++ `Session::print() → window->print()` became Rust `Session::print_char() → (*output_window).print()`. Similar pattern, different syntax.

---

## References

- C++ Session: `mcl-cpp-reference/Session.cc:290, 580`
- C++ OutputWindow: `mcl-cpp-reference/OutputWindow.cc:9-18, 239-275`
- C++ Window: `mcl-cpp-reference/Window.cc:169-247`
- Rust Session: `src/session.rs:36-40, 64-70`
- Rust OutputWindow: `src/output_window.rs:17-41, 49-86`
- Test harness: `tests/test_nodeka_splash.rs`
- Captured data: `test_captures/nodeka/*.txt.json`
