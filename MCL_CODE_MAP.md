# MCL C++ Reference — Code Map

## ⚠️ CRITICAL: Virtual Dispatch in C++ → Rust Ports

**READ THIS FIRST:** The single most important lesson from this port.

**The Problem:** Rust composition (HAS-A) doesn't replicate C++ inheritance (IS-A) automatically.

**What Breaks:**
- C++: `output->refresh()` → vtable → `OutputWindow::redraw()` ✅
- Rust: `screen.refresh()` → tree walk → finds `output.win` → `Window::refresh()` → `Window::redraw()` (base class) ❌

**OutputWindow::redraw() is never called!** Data is in the buffer but never copied to canvas.

**The Fix (Manual Virtual Dispatch):**
```rust
// BEFORE screen.refresh(), explicitly call derived methods:
if output.win.dirty { output.redraw(); output.win.dirty = true; }
if input.win.dirty { input.redraw(); input.win.dirty = true; }
screen.refresh(&caps);  // Now tree refresh works correctly
```

**Cost of Missing This:**
- 3 display bugs (invisible prompts, missing splash lines, stuck input)
- Hours debugging buffer logic, scroll calculations, offsets (all correct, but wrong area!)
- 5 lines of code that should have been written on day 1

**See:** `DISPLAY_BUG_POSTMORTEM.md` and `docs/virtual-dispatch-blog.md` for complete analysis.

---

## Overview
Event-loop TUI MUD client with custom window system and embedded scripting. Core architecture: main loop selects on file descriptors → dispatches to sockets/TTY → updates in-memory window canvases → diffs to terminal via ANSI codes.

## TTY Rendering Architecture (CRITICAL FOR PORTING)

### Data Flow: Network → Screen

```
Network Socket
    ↓ read()
Session::inputReady()
    ↓ MCCP decompress → telnet parse → ANSI parse
Session::print(out_buf)  [Session.cc:580]
    ↓ window->print(s)    [Session.cc:290 - virtual dispatch]
Window::print(const char *s)  [Window.cc:169-247]
    ↓ writes to canvas at cursor position
    ↓ advances cursor, handles \n, tabs, scrolling
OutputWindow::scroll()  [OutputWindow.cc:32-61]
    ↓ advances canvas pointer within scrollback
    ↓ advances viewpoint if not frozen
Window::refresh()  [Window.cc:325-350]
    ↓ tree walk: redraw() → refresh children → draw_on_parent()
OutputWindow::draw_on_parent()  [OutputWindow.cc:239-275]
    ↓ parent->copy(viewpoint, ...) - copies VIEWPOINT to parent canvas
Screen::refresh()  [Screen.cc:105-110, 183-299]
    ↓ diffs canvas vs last_screen → generates ANSI
    ↓ writes to stdout (TTY mode) or /dev/vcsa (Linux console mode)
```

### Critical Architecture: OutputWindow Buffer Pointers

**C++ Pattern** (OutputWindow.cc:9-18):
```cpp
class OutputWindow : public Window {
    attrib *scrollback;  // Heap-allocated ring buffer
    attrib *viewpoint;   // What we're viewing (pointer INTO scrollback)
    // NOTE: canvas inherited from Window is ALSO a pointer INTO scrollback!
};

// Constructor (line 9-18):
scrollback = new attrib[width * scrollback_lines];
viewpoint = canvas = scrollback;  // ← ALL THREE point to same buffer!
```

**Key Insight**: In C++, `canvas`, `viewpoint`, and `scrollback` are **pointers into the same memory**.
- `scrollback`: Start of ring buffer
- `canvas`: Current write position (advances as lines are printed)
- `viewpoint`: Current display position (what's visible on screen)

When scrolling (OutputWindow.cc:55):
```cpp
canvas += width;  // Advance write position by one line
if (!fFrozen)
    viewpoint += width;  // Follow canvas unless user scrolled back
```

**Porting Impact**: Rust cannot replicate this pointer-into-buffer pattern safely. The Rust port uses separate buffers with manual copying, creating architectural mismatch.

### Window Class Hierarchy & Virtual Dispatch

**Base Class: Window** (Window.h, Window.cc)
- Owns `attrib *canvas` - in-memory character buffer (color + char packed in u16)
- Tree structure: parent, child_first, child_last, next, prev pointers
- Virtual methods:
  - `print(const char *s)` - writes to canvas at cursor position [Window.cc:169-247]
  - `scroll()` - base implementation shifts canvas up [Window.cc:149-167]
  - `redraw()` - no-op in base class (some subclasses override)
  - `keypress(int key)` - event dispatch [Window.cc:352-372]

**Subclass: OutputWindow** (OutputWindow.h, OutputWindow.cc)
- Overrides `scroll()` to advance canvas pointer within scrollback ring buffer
- Overrides `draw_on_parent()` to copy from VIEWPOINT (not canvas) to parent
- Has separate `viewpoint` pointer for scrollback navigation
- Key difference: `canvas` and `viewpoint` can diverge when user scrolls back

**Subclass: Screen** (Screen.h, Screen.cc)
- Top-level window (no parent)
- Overrides `refresh()` to render to terminal via ANSI diff
- Has `last_screen` buffer for delta detection

**Virtual Dispatch Critical Points**:
1. `Session::print()` calls `window->print(s)` [Session.cc:290]
   - In C++: Dispatches to OutputWindow instance if set
   - OutputWindow inherits Window::print() (doesn't override it)
   - So it uses base Window::print() which writes to `canvas`

2. `Window::refresh()` calls tree walk [Window.cc:325-350]:
   ```cpp
   if (dirty) {
       redraw();  // Virtual - most classes don't override
   }
   // Refresh children recursively
   for (Window *w = child_first; w; w = w->next) {
       w->refresh();
   }
   draw_on_parent();  // Virtual - OutputWindow DOES override this
   ```

3. `OutputWindow::draw_on_parent()` [OutputWindow.cc:239-275]:
   ```cpp
   // Copies from VIEWPOINT, not canvas!
   parent->copy(viewpoint, width, height, parent_x, parent_y);
   ```

### Window::print() Character-by-Character Writing

**Critical Method**: `Window::print(const char *s)` [Window.cc:169-247]

This is the heart of how text gets rendered. It writes characters ONE AT A TIME to the canvas at the current cursor position.

```cpp
void Window::print(const char *s) {
    const unsigned char *in;
    attrib *out;

    dirty = true;

    // Position output pointer at cursor
    out = canvas + cursor_y * width + cursor_x;

    for (in = (unsigned char *)s; *in; in++) {
        if (*in == SET_COLOR) {
            // Handle color change (color = *++in)
        } else if (*in == SOFT_CR) {
            // Soft carriage return (move to next line if not already)
            if (cursor_x) {
                cursor_x = 0;
                cursor_y++;
                out = canvas + cursor_y * width + cursor_x;
            }
        } else if (*in == '\n') {
            // Hard newline
            cursor_x = 0;
            cursor_y++;
            out = canvas + cursor_y * width + cursor_x;
        } else {
            // Regular character - write and advance
            while (cursor_y >= height)
                if (!scroll())  // Virtual dispatch!
                    return;

            out = canvas + cursor_y * width + cursor_x;  // Recalc after scroll
            *out++ = (color << 8) + *in;  // Write packed attrib
            cursor_x++;
        }

        // Wordwrap
        if (cursor_x == width) {
            cursor_y++;
            cursor_x = 0;
        }
    }
}
```

**Key Points**:
1. **Cursor tracking**: `cursor_x`, `cursor_y` track write position
2. **Direct memory writes**: Characters go directly into `canvas` at cursor offset
3. **Immediate visibility**: No line buffering - partial lines are visible immediately
4. **Scrolling**: Calls virtual `scroll()` when cursor_y >= height
5. **Color handling**: SET_COLOR byte followed by color code

**Porting Impact**: Rust port uses line-based `Scrollback::print_line()` instead of character-by-character appending. This breaks partial line rendering (prompts without \n stay invisible in `line_buf`).

### OutputWindow::scroll() - Ring Buffer Management

**Method**: `OutputWindow::scroll()` [OutputWindow.cc:32-61]

Handles advancing the write position within the scrollback ring buffer.

```cpp
bool OutputWindow::scroll() {
    // Check if at end of ring buffer
    if (canvas == scrollback + width * (scrollback_lines - height)) {
        // At end - copy older lines up, reuse space
        memmove(scrollback, scrollback + width * COPY_LINES,
                (scrollback_lines - COPY_LINES) * width * sizeof(attrib));
        canvas -= width * COPY_LINES;
        viewpoint -= width * COPY_LINES;
        top_line += COPY_LINES;

        if (viewpoint < scrollback)
            viewpoint = scrollback;

        // Clear newly available space
        canvas += width * height;
        clear();
        canvas -= width * height;
    } else {
        // Not at end - just advance canvas pointer
        canvas += width;
        clear_line(height-1);  // Clear bottom line
        cursor_y--;

        if (!fFrozen)  // Track canvas unless user scrolled back
            viewpoint += width;
    }

    return true;
}
```

**Key Points**:
1. **Ring buffer recycling**: When reaching end, memmove() copies lines to beginning
2. **Pointer arithmetic**: `canvas` advances by `width` (one line) each scroll
3. **Viewpoint tracking**: `viewpoint` follows `canvas` unless frozen (user scrolled back)
4. **top_line counter**: Tracks absolute line number for search/highlighting

**Porting Impact**: Rust port has similar logic in `Scrollback::print_line()` but operates on line boundaries, not character boundaries.

### OutputWindow::draw_on_parent() - The Rendering Gap

**Method**: `OutputWindow::draw_on_parent()` [OutputWindow.cc:239-275]

This is where C++ copies the visible portion of scrollback to the parent window (Screen).

```cpp
void OutputWindow::draw_on_parent() {
    if (parent) {
        // Handle search highlighting if active
        if (highlight.line >= 0 && highlight in viewport) {
            // Temporarily invert colors in highlight region
            attrib old[highlight.len];
            memcpy(old, viewpoint + offset, ...);
            // Invert colors
            for (b = a; b < a+highlight.len; b++) {
                unsigned char color = ((*b & 0xFF00) >> 8);
                unsigned char bg = (color & 0x0F) << 4;
                unsigned char fg = (color & 0xF0) >> 4;
                *b = (*b & 0x00FF) | ((bg|fg) << 8);
            }
            // Copy to parent
            parent->copy(viewpoint, width, height, parent_x, parent_y);
            // Restore original colors
            memcpy(viewpoint + offset, old, ...);
        } else {
            // Normal case - copy viewpoint to parent canvas
            parent->copy(viewpoint, width, height, parent_x, parent_y);
        }
    }
}
```

**CRITICAL**: This copies from `viewpoint`, NOT from `canvas`!
- `canvas` is where Session writes new data
- `viewpoint` is what the user sees (can lag behind if scrolled back)

**Porting Impact**: Rust port's `Window::draw_on_parent()` copies from `self.canvas`, missing the viewpoint/canvas distinction. But in Rust, OutputWindow has its own `sb: Scrollback` which is separate from `session.scrollback`, creating double-buffering.

### Screen::refresh() - Terminal Rendering

**Method**: `Screen::refresh()` [Screen.cc:105-110, 183-299]

Top-level refresh that renders to terminal.

```cpp
bool Screen::refresh() {
    // Call base Window::refresh() to composite tree
    bool changed = Window::refresh();  // [Screen.cc:84]

    if (changed) {
        if (using_virtual)
            refreshVirtual();  // Linux /dev/vcsa direct write
        else
            refreshTTY();      // ANSI escape codes
    }

    return changed;
}
```

**refreshTTY() Detail** [Screen.cc:183-299]:
```cpp
void Screen::refreshTTY() {
    // Detect scrolling opportunity (optimize bulk line moves)
    int scroll_lines = planScrollUp(last_screen, canvas, ...);
    if (scroll_lines) {
        // Use terminal scroll region to shift lines
        emitScrollAnsi(scroll_lines);
        // Update last_screen to reflect scroll
        adjustLastScreenForScroll(scroll_lines);
    }

    // Diff canvas vs last_screen, emit only changes
    for (y = 0; y < height; y++) {
        for (x = 0; x < width; x++) {
            if (canvas[y*width + x] != last_screen[y*width + x]) {
                // Emit color change if needed
                // Emit cursor positioning if needed
                // Emit character
            }
        }
    }

    // Position final cursor
    gotoXY(cursor_x, cursor_y);

    // Update last_screen for next diff
    memcpy(last_screen, canvas, width * height * sizeof(attrib));
}
```

**Key Points**:
1. **Delta rendering**: Only emits changes since last frame
2. **Scroll optimization**: Detects line shifts and uses ANSI scroll regions
3. **Color optimization**: Only emits color codes when color changes
4. **Cursor optimization**: Minimizes cursor movement commands

**Porting Status**: Rust has equivalent `diff_to_ansi()` in screen.rs - this part works correctly.

### Session::print() - The Entry Point

**Method**: `Session::print(const char *s)` [Session.cc:290]

Simple virtual dispatch to window:
```cpp
void Session::print(const char *s) {
    if (window)
        window->print(s);
}
```

**Called From**: `Session::inputReady()` after telnet/ANSI parsing [Session.cc:580]:
```cpp
print(out_buf);  // Writes parsed output to window
```

**Porting Impact**: In C++, Session doesn't own scrollback - it just calls window->print(). In Rust, Session OWNS `pub scrollback: Scrollback`, creating architectural mismatch.

## File-by-File Reference

### Window.cc (Base Canvas & Tree)
- **Lines 10-60**: Constructor - allocates canvas, handles sizing/positioning
- **Lines 62-83**: Destructor, insert/remove from parent tree
- **Lines 85-148**: clear(), clear_line(), gotoxy() - canvas manipulation
- **Lines 149-167**: scroll() - base implementation shifts canvas up
- **Lines 169-247**: **print(const char *s)** - character-by-character writing ⚠️ CRITICAL
- **Lines 249-323**: copy() - copies source buffer to canvas region
- **Lines 325-350**: refresh() - tree walk: redraw() → children → draw_on_parent()
- **Lines 352-372**: keypress() - event dispatch
- **Lines 374-423**: bordered(), setf(), printf() - utilities

### OutputWindow.cc (Scrollback & Viewpoint)
- **Lines 9-22**: **Constructor** - allocates scrollback, sets viewpoint = canvas = scrollback ⚠️ CRITICAL
- **Lines 24-28**: Destructor
- **Lines 32-61**: **scroll()** - ring buffer management, advances canvas/viewpoint ⚠️ CRITICAL
- **Lines 65-172**: moveViewpoint() - Page Up/Down/Home navigation
- **Lines 174-236**: search() - find text in scrollback, highlight
- **Lines 239-275**: **draw_on_parent()** - copies viewpoint to parent ⚠️ CRITICAL
- **Lines 277-279**: move() - repositions window
- **Lines 301-339**: saveToFile() - export scrollback to file

### Screen.cc (Terminal Rendering)
- **Lines 39-69**: Constructor - allocates last_screen, sets up /dev/vcsa or TTY mode
- **Lines 71-102**: Destructor, initialization
- **Lines 105-110**: **refresh()** - calls Window::refresh() then refreshTTY/Virtual ⚠️ CRITICAL
- **Lines 112-181**: refreshVirtual() - direct /dev/vcsa write (Linux console)
- **Lines 183-299**: **refreshTTY()** - ANSI diff rendering ⚠️ CRITICAL
- **Lines 301-337**: keypress() - dispatches to focused window

### Session.cc (Network → Window Bridge)
- **Lines 27-92**: Constructor/Destructor
- **Lines 287-293**: **print()** - window->print(s) virtual dispatch ⚠️ CRITICAL
- **Lines 295-309**: open() - connect to MUD
- **Lines 311-372**: close() - disconnect, cleanup
- **Lines 374-615**: **inputReady()** - MCCP decompress → telnet parse → ANSI parse → print() ⚠️ CRITICAL
- **Lines 580**: **print(out_buf)** - writes parsed data to window ⚠️ CRITICAL
- **Lines 617-755**: outputReady(), write_mud() - send data to MUD

### Window.h (Interface Definitions)
- **Lines 30-120**: Window class declaration
  - **Line 105**: `attrib *canvas` - the in-memory character buffer
  - **Lines 82-91**: Virtual methods: redraw(), scroll(), keypress(), draw_on_parent()
  - **Lines 56-74**: Tree structure: parent, child_first/last, next/prev

### OutputWindow.h (Scrollback Interface)
- **Lines 4-46**: OutputWindow class declaration
  - **Line 33**: `attrib *scrollback` - ring buffer start
  - **Line 34**: `attrib *viewpoint` - current display position
  - **Line 26**: `virtual void draw_on_parent()` - override declaration

### Screen.h (Terminal Interface)
- **Lines 8-36**: Screen class declaration
  - Inherits from Window
  - Overrides refresh(), keypress()
  - Has last_screen, using_virtual (mode flag)

## Porting Pitfalls (Lessons Learned The Hard Way)

### 1. ⚠️ VIRTUAL DISPATCH - The Bug That Cost Us Hours

**THE CORE LESSON:** Every C++ virtual method needs explicit Rust dispatch.

**The Bug Pattern:**
```cpp
// C++ (works automatically via vtable):
class OutputWindow : public Window {
    virtual void redraw();  // Overrides base
};
output->refresh();  // → vtable → OutputWindow::redraw() ✅
```

```rust
// Rust composition (BREAKS without manual dispatch):
pub struct OutputWindow {
    pub win: Box<Window>,  // HAS-A, not IS-A
}
impl OutputWindow {
    pub fn redraw(&mut self) { /* ... */ }
}
screen.refresh();  // → tree → output.win → Window::redraw() (base!) ❌
// OutputWindow::redraw() NEVER CALLED!
```

**Symptoms When You Miss This:**
- "Data is in buffer but not on screen" ← CLASSIC RED FLAG
- "Works in one mode, broken in another" (headless vs TTY)
- "I see it in debug logs but not displayed"
- Debug logging shows correct data flow, but display is wrong

**The Fix (Always!):**
```rust
// BEFORE tree refresh, manually dispatch derived methods:
if output.win.dirty { output.redraw(); output.win.dirty = true; }
if input.win.dirty { input.redraw(); input.win.dirty = true; }
screen.refresh(&caps);
```

**Checklist for EVERY C++ class with virtual methods:**
- [ ] List all virtual methods (redraw, scroll, keypress, etc.)
- [ ] Find where C++ calls them via base pointer (vtable points)
- [ ] Add explicit Rust calls at those same points
- [ ] Verify with debug logging that methods fire
- [ ] Test with real data, not just synthetic tests

**This bug caused 3 display issues and hours of wasted debugging. Don't skip this step!**

### 2. Pointer-Into-Buffer Architecture

**C++ Pattern**:
```cpp
attrib *scrollback = new attrib[width * lines];
attrib *canvas = scrollback;      // Points into scrollback
attrib *viewpoint = scrollback;   // Points into scrollback
// Later:
canvas += width;                   // Advance by one line
viewpoint += width;                // Follow canvas
```

**Rust Solution**: Use offsets instead of pointers
```rust
pub struct Scrollback {
    pub buf: Vec<u16>,        // The buffer
    pub canvas_off: usize,    // Offset (replaces canvas pointer)
    pub viewpoint: usize,     // Offset (replaces viewpoint pointer)
}
// Later:
canvas_off += width;  // Advance by one line
```

**Impact**: Requires careful offset arithmetic but avoids lifetime/aliasing issues.

### 3. Character-by-Character vs Line-by-Line

**C++ Pattern**: `Window::print()` writes characters immediately at cursor position
- Partial lines visible instantly
- Prompts without \n appear immediately

**Rust Pattern**: `Scrollback::print_line()` only creates complete lines
- Partial data stays in `line_buf` until \n or prompt event
- Prompts without \n (and no GA/EOR) stay invisible

**Fix**: Flush `line_buf` on prompt events (GA/EOR telnet codes)

### 4. Tree Walk & Compositing ✅

**C++ Pattern**: `Window::refresh()` recursively composites children onto parents
**Rust Status**: ✅ Correctly ported (Window::refresh() in window.rs)

**Works Because**: Tree structure uses raw pointers (`*mut Window`), mimics C++ exactly.

### 5. Double-Buffering Diff Rendering ✅

**C++ Pattern**: Screen stores last_screen, diffs against canvas, emits ANSI
**Rust Status**: ✅ Correctly ported (Screen::refresh_tty() in screen.rs)

**Works Because**: This is pure algorithm - no architectural dependency on pointers.

## Critical Porting Workflow

**Day 1 of ANY class port:**

1. **Identify inheritance**: Does C++ use `virtual` methods? If yes → ALERT!
2. **Trace vtable calls**: Where does base pointer call derived methods?
3. **Add manual dispatch**: Insert explicit calls in Rust at those points
4. **Verify execution**: Debug log to prove methods fire
5. **Test with real data**: Synthetic tests hide timing/dispatch bugs

**The Iron Rule**: If C++ does X, Rust must do X (semantically). "Close enough" is a bug.

## References

**MUST READ for Porting**:
- `DISPLAY_BUG_POSTMORTEM.md` - Complete debugging history (virtual dispatch lesson)
- `docs/virtual-dispatch-blog.md` - Public blog post on the virtual dispatch bug
- `CLAUDE.md` - Updated dev guidelines with virtual dispatch patterns

**Key Commits (Virtual Dispatch Fixes)**:
- [08bcac2](https://github.com/selberhad/okros/commit/08bcac2) – OutputWindow::redraw() manual dispatch
- [253c332](https://github.com/selberhad/okros/commit/253c332) – InputLine::redraw() manual dispatch
- [aa51b66](https://github.com/selberhad/okros/commit/aa51b66) – Input buffer clear fix
- [6b3546a](https://github.com/selberhad/okros/commit/6b3546a) – Character-by-character rendering

**Porting History**:
- Session porting: Phases 1-3 complete (see PORTING_HISTORY.md)
- Window/OutputWindow: Toy 10 (scrollback), direct ports
- Screen: screen.rs (diff_to_ansi working)

**Testing Artifacts**:
- `tests/test_nodeka_splash.rs` - Real MUD data test harness
- `test_captures/nodeka/*.json` - Captured Nodeka splash screens
- `scripts/test_nodeka_tty.sh` - TTY mode validation script
