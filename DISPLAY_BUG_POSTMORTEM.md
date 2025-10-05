# Display Bug Post-Mortem: The Cost of Taking Shortcuts in a 1:1 Port

## Executive Summary

**Bug**: Lines 6-8 of Nodeka splash screen missing, creating visible gap in ASCII art output
**Root Cause**: Taking a shortcut instead of doing the 1:1 port (composition without proper virtual dispatch equivalent)
**Fix**: Do the 1:1 port - manually hook OutputWindow::redraw() into refresh cycle
**Lesson**: **When your goal is a 1:1 port, don't take shortcuts. Do the 1:1 port.**

---

## What We Did Wrong

### The Shortcut We Took

**C++ Pattern** (OutputWindow.cc:8-22):
```cpp
class OutputWindow : public Window {
    // OutputWindow IS-A Window
    // Virtual dispatch handles rendering
};
```

**Our "Clever" Rust** (output_window.rs:16-23):
```rust
pub struct OutputWindow {
    pub win: Box<Window>,  // OutputWindow HAS-A Window
    pub sb: Scrollback,
    // ...
}
```

**Why we thought it was okay**:
- "Rust doesn't have inheritance, so composition is fine"
- "We can just manually hook things up"
- "It's close enough"

**What actually happened**:
1. `output.win` got added to the window tree (correct)
2. Tree refresh called `output.win.refresh()` → `output.win.redraw()` (wrong - base class method!)
3. `OutputWindow::redraw()` (which copies scrollback→canvas) **was never called**
4. Canvas stayed empty, or had stale data
5. Characters written to scrollback buffer never made it to the screen

---

## How the Bug Manifested

### The Symptoms

```
Connected.
            ****''                                  ''****
            ****                                      ****
            ***          **                ***         ***
  ++    ++  ***   +++++   ****++++     ++++***  ++   ++***    ++
  +++   ++  *** ++  +  ++  ****** ++

                                                ← GAP HERE!

                                                ++ ++  ***++++++++++
  ++   +++  *** ++  +  ++   ++ ** ++ ****+  ++  ++  ++ ***++      ++
  ...
```

Lines 6-8 completely missing. Data was written correctly to the scrollback buffer (debug confirmed this), but never made it to the screen.

### What We Tried (All Band-Aids)

1. ❌ "Maybe it's the scroll logic" - Fixed scrolling, gap remained
2. ❌ "Maybe it's the canvas_off calculation" - Fixed offsets, gap remained
3. ❌ "Maybe we need manual redraw() call" - Added it, gap remained
4. ❌ "Maybe we need to call it at different times" - Tried 5 different places, gap remained

**We kept trying to fix the SYMPTOM instead of fixing the ROOT CAUSE: we didn't do the 1:1 port.**

---

## The Actual Fix

### What C++ Does

**Window.cc:320-351** (refresh cycle):
```cpp
bool Window::refresh() {
    if (dirty) {
        redraw();        // Virtual dispatch - OutputWindow overrides this
    }
    // ... refresh children ...
    draw_on_parent();    // Copy canvas to parent
}
```

In C++, `OutputWindow::redraw()` gets called via **virtual dispatch** when `output->refresh()` is called.

### What Rust Needs

Since we use composition (HAS-A) instead of inheritance (IS-A), we need to **manually hook the virtual dispatch**:

**main.rs:261-271**:
```rust
// OutputWindow composition workaround: manually call redraw before tree refresh
// C++ uses inheritance (OutputWindow IS-A Window), Rust uses composition (OutputWindow HAS-A Window)
// So output.win is in tree, but OutputWindow::redraw() must be called manually
if output.win.dirty {
    output.redraw();           // Manual "virtual dispatch"
    output.win.dirty = true;   // Keep dirty for tree refresh
}

screen.refresh(&caps);         // Tree refresh handles rest
```

**This is the 1:1 port of the virtual dispatch pattern**, just using explicit calls instead of language-level polymorphism.

---

## The Real Lesson

### Timeline of Bad Decisions

1. **Initial port**: "C++ uses inheritance, we'll use composition"
   → *Seems reasonable, Rust doesn't have inheritance*

2. **Debugging starts**: "Gap in output, must be a buffer offset bug"
   → *Didn't check: are we even calling the right rendering code?*

3. **Tried scrolling fixes**: "Maybe it's the scroll logic?"
   → *Fixed scrolling, gap remained*

4. **Tried canvas_off fixes**: "Maybe it's the offset calculations?"
   → *Fixed offsets, gap remained*

5. **Tried manual redraw**: "Maybe we need to call redraw() manually?"
   → *Getting warmer, but kept trying different places*

6. **Finally**: "Wait, we never set up the virtual dispatch equivalent"
   → *Should have done this from the start!*

### What We Should Have Done

**From the start**:
```rust
// C++ uses virtual dispatch: OutputWindow::redraw() is called via output->refresh()
// Rust doesn't have virtual dispatch with composition
// THEREFORE: We must manually hook OutputWindow::redraw() into the refresh cycle
// This is the 1:1 port of the C++ pattern
if output.win.dirty {
    output.redraw();  // Explicit call = Rust's version of virtual dispatch
    output.win.dirty = true;
}
screen.refresh(&caps);
```

**One function, added at the start, would have prevented hours of debugging.**

---

## Core Principle

### When Your Goal Is a 1:1 Port

**DON'T**:
- ❌ "I'll use composition instead of inheritance, close enough"
- ❌ "I'll refactor this to be more Rust-like"
- ❌ "I'll optimize this later, let's get it working first"
- ❌ "The data flows right, rendering is a separate problem"

**DO**:
- ✅ **Trace the C++ execution path completely**
- ✅ **Port every step of that path, even if Rust syntax differs**
- ✅ **Inheritance → explicit dispatch = 1:1 port**
- ✅ **Virtual calls → manual calls = same semantics, different syntax**

### The Iron Rule

> If C++ does X, and you do "something close to X", you are NOT doing a 1:1 port.
> You are doing a reimplementation.
> Reimplementations have bugs.
> 1:1 ports copy bugs, not create them.

---

## What "1:1 Port" Actually Means

### Not This (Syntax-Level Copy)
```rust
// "I copied the C++ structure exactly!"
pub struct OutputWindow : pub Window { // ERROR: Rust has no inheritance
```

### This (Semantic-Level Copy)
```rust
// "I ported the C++ behavior exactly using Rust's tools"

// C++: OutputWindow::redraw() called via virtual dispatch in Window::refresh()
// Rust: OutputWindow::redraw() called via explicit dispatch before Window::refresh()

if output.win.dirty {
    output.redraw();        // Rust's way of doing virtual dispatch
    output.win.dirty = true;
}
screen.refresh(&caps);      // Continues the C++ refresh cycle
```

**Same behavior, different syntax = 1:1 port ✅**

---

## Checklist for Future Ports

When porting C++ to Rust:

- [ ] Identify C++ virtual dispatch patterns
- [ ] Map C++ vtable calls → Rust explicit calls
- [ ] If C++ calls `base->method()`, Rust must call equivalent
- [ ] Don't assume "close enough" composition is the same as inheritance
- [ ] Test the CALL PATH, not just the data flow
- [ ] When in doubt: trace C++ execution step-by-step, port each step

---

## Conclusion

**Hours of debugging, hundreds of lines of diagnostic code, 6 different attempted fixes.**

**All because we took a shortcut in the initial port.**

**The 1:1 port would have been:**
```rust
if output.win.dirty {
    output.redraw();
    output.win.dirty = true;
}
```

**Five lines. From the start. Done.**

### Remember

When your goal is a 1:1 port, **every shortcut is a bug waiting to happen**.

Don't be clever. Don't be efficient. Don't be Rusty.

**Be C++, in Rust syntax.**

---

## Technical Context (For Reference)

### The Character-by-Character Rendering Fix (Prerequisite)

Before this gap bug, there was a separate issue where prompts weren't visible at all. That was fixed by implementing character-by-character rendering (matching C++ `Window::print()`):

**Previous Bug**: Prompts invisible because Rust buffered in `line_buf` until `\n`, but prompts have no newline
**Fix**: `Session::print_char()` writes every character immediately to `OutputWindow::print()` in TTY mode
**Result**: Prompts visible, but gap bug still present

See commit 6b3546a for the character-by-character rendering implementation.

### Why Both Bugs Existed

1. **Character buffering bug**: Wrong approach (line-based vs char-based) - didn't match C++ semantics
2. **Gap bug**: Right approach (composition) but incomplete port - didn't hook virtual dispatch

Both were shortcuts. Both caused bugs. Both required going back and doing the 1:1 port properly.

---

## References

- C++ Window::refresh(): `mcl-cpp-reference/Window.cc:320-351`
- C++ virtual dispatch: `OutputWindow : public Window`
- Rust fix: `src/main.rs:261-271`
- Character rendering fix: Commit 6b3546a
- Gap bug fix: Manual virtual dispatch workaround
