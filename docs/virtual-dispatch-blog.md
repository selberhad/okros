# The $5 Lines That Cost Us Hours: Why "Close Enough" Isn't in 1:1 Ports

**Or: How We Learned That Composition ≠ Inheritance The Hard Way**

## The Problem: Invisible Text and Mysterious Gaps

We were in the final stretch of porting MCL (a MUD client from the early 2000s) from C++ to Rust. Everything worked in headless mode. TTY interactive mode was *mostly* working. But something was wrong with the display:

**Bug #1:** The Nodeka splash screen had a visible gap where lines 6-8 should be:

```
Connected.
            ****''                                  ''****
            ****                                      ****
            ***          **                ***         ***
  ++    ++  ***   +++++   ****++++     ++++***  ++   ++***    ++
  +++   ++  *** ++  +  ++  ****** ++

                                                ← WHERE ARE LINES 6-8?!

                                                ++ ++  ***++++++++++
  ++   +++  *** ++  +  ++   ++ ** ++ ****+  ++  ++  ++ ***++      ++
```

**Bug #2:** The input text wasn't visible in the input box. You could type, but you couldn't see what you were typing.

**Bug #3:** After hitting Enter, the input text didn't clear.

Three bugs, all with the same root cause. We just didn't know it yet.

## First Debugging Attempt: The Data Path

"Let's trace the data," we thought. "It must be a buffer issue."

We added debug logging to track character writes:
- ✅ Characters written to scrollback buffer correctly
- ✅ All 19 lines present in the buffer
- ✅ No characters dropped
- ✅ Viewport calculations correct

The data was flowing perfectly. But it wasn't showing up on screen.

## Second Attempt: The Scroll Logic

"Maybe it's the scrolling?" We checked the C++ reference:

```cpp
// OutputWindow.cc:35
if (canvas == scrollback + width * (scrollback_lines - height)) {
    // At end of buffer - shift everything up
    memmove(scrollback, scrollback + width * COPY_LINES, ...);
    canvas -= width * COPY_LINES;
    viewpoint -= width * COPY_LINES;
}
```

We implemented the memmove logic. We fixed the canvas_off calculations. We added bounds checking.

**Result:** Gap still there. 😢

## Third Attempt: The Canvas Pointer

"Wait," we realized. "C++ uses `canvas` as a pointer INTO scrollback. We're using `canvas_off` as an offset."

We fixed the offset calculations:

```rust
// Write to canvas_off offset (like C++ canvas pointer)
let offset = self.sb.canvas_off + self.cursor_y * self.sb.width + self.cursor_x;
self.sb.buf[offset] = ((color as u16) << 8) | (ch as u16);
```

**Result:** Better! But the gap remained.

## The Breakthrough: "Wait, Where's redraw() Being Called?"

After hours of debugging buffer logic, scroll calculations, and offset math, we asked a different question:

> "The data is in the buffer. The buffer is correct. So why isn't it on the screen?"

We added one more debug line:

```rust
pub fn redraw(&mut self) {
    debug_log!("REDRAW called: viewpoint={}, canvas_off={}",
               self.sb.viewpoint, self.sb.canvas_off);
    // ... actual redraw code
}
```

We ran it. We connected to Nodeka. We saw the gap.

**We checked the debug log.**

**REDRAW was never called.** 😱

## The Root Cause: We Took a Shortcut

Here's what we did when we started the port:

**C++ Code:**
```cpp
class OutputWindow : public Window {
    // OutputWindow IS-A Window
    virtual void redraw();  // Overrides Window::redraw()
};

// Later...
output->refresh();  // Calls Window::refresh() → OutputWindow::redraw() via vtable
```

**Our "Clever" Rust:**
```rust
pub struct OutputWindow {
    pub win: Box<Window>,  // OutputWindow HAS-A Window
    pub sb: Scrollback,
}

impl OutputWindow {
    pub fn redraw(&mut self) {
        // Copy scrollback to canvas...
    }
}
```

We thought: "Rust doesn't have inheritance, so we'll use composition. Close enough."

**It wasn't close enough.**

## What We Missed: Virtual Dispatch Doesn't Happen Automatically

In C++, when you call:

```cpp
Window *output = new OutputWindow(...);
output->refresh();  // Polymorphism!
```

The vtable automatically dispatches to `OutputWindow::redraw()`.

In Rust, when the tree refresh happens:

```rust
// main.rs event loop
screen.refresh(&caps);  // Walks tree, calls output.win.refresh()
```

The tree finds `output.win` (the Window), calls `Window::refresh()`, which calls `Window::redraw()` (base class - just sets dirty=false).

**`OutputWindow::redraw()` is never called.**

The data is in the scrollback buffer. But it's never copied to the canvas. So it never appears on screen.

## The Fix: 5 Lines of Code

```rust
// main.rs event loop, BEFORE screen.refresh()
if output.win.dirty {
    output.redraw();           // Manual "virtual dispatch"
    output.win.dirty = true;   // Keep dirty for tree
}

screen.refresh(&caps);         // Tree refresh continues
```

That's it. Five lines.

**Those five lines should have been there from day one.**

## The Second Bug: Same Pattern

Once we understood the issue, we immediately checked InputLine:

```rust
pub struct InputLine {
    pub win: Box<Window>,  // HAS-A Window, not IS-A
    // ...
}

impl InputLine {
    pub fn redraw(&mut self) {
        // Writes prompt + input text to canvas...
    }
}
```

Same bug! InputLine::redraw() was never being called!

**The fix:** Same pattern, 4 more lines:

```rust
if input.win.dirty {
    input.redraw();
    input.win.dirty = true;
}

screen.refresh(&caps);
```

**Bug #2 fixed.**

## The Third Bug: A Different Lesson

Bug #3 (input not clearing after Enter) had a different root cause:

**C++ Code:**
```cpp
// Enter key handler
max_pos = 0;  // Setting to 0 "hides" old data in char array
cursor_pos = 0;
// redraw() only reads input_buf[0..max_pos] = input_buf[0..0] = empty
```

**Our Rust:**
```rust
// Enter key handler
self.max_pos = 0;
self.cursor_pos = 0;
// redraw() reads input_buf[0..max_pos], but Vec still contains old bytes!
```

In C++, setting `max_pos=0` is enough because the old data stays in the char array but is "hidden" by the zero-length slice.

In Rust with `Vec<u8>`, the old bytes are still there and can show up in the next render.

**The fix:** One line:

```rust
self.input_buf.clear();  // Actually clear the Vec
```

**Bug #3 fixed.**

## The Postmortem: Every Shortcut is a Bug

We wrote a detailed postmortem (`DISPLAY_BUG_POSTMORTEM.md`) with one key lesson:

> **When your goal is a 1:1 port, every shortcut is a bug waiting to happen.**

### What We Did Wrong

We said: "Composition is close enough to inheritance."

**It wasn't.** Composition gives you the structure, but you lose the polymorphism.

We said: "The data flows right, so it's fine."

**It wasn't.** We ported the DATA FLOW but forgot the CALL PATH.

We said: "We'll hook it up manually somewhere."

**We didn't.** And we spent hours debugging the wrong things.

### What We Should Have Done

**Day 1 of the port:**

1. Identify all C++ classes with virtual methods
2. For each one, ask: "Where does C++ call this via a base pointer?"
3. In Rust, add explicit calls at those exact points
4. Test that the methods actually fire

**The 1:1 port would have been:**

```rust
// C++ uses virtual dispatch here, so we need manual dispatch
if output.win.dirty { output.redraw(); output.win.dirty = true; }
if input.win.dirty { input.redraw(); input.win.dirty = true; }
screen.refresh(&caps);
```

**Nine lines. Day 1. Done.**

Instead, we:
- Spent hours debugging buffer logic ✅ (made it better, but didn't fix the bug)
- Fixed scrollback calculations ✅ (needed anyway, but didn't fix the bug)
- Implemented memmove logic ✅ (matched C++, but didn't fix the bug)
- Added canvas_off support ✅ (correct, but didn't fix the bug)

All good work. None of it addressed the root cause.

## The Checklist We Wish We Had

For anyone porting C++ inheritance to Rust composition:

### When You Port a C++ Class with Virtual Methods

- [ ] List ALL virtual methods in the C++ class
- [ ] Find where C++ calls them via base class pointers (vtable dispatch)
- [ ] In Rust, add explicit calls at those same points
- [ ] Add debug logging to verify the methods fire
- [ ] Test with real data (not just synthetic tests)

### Red Flags That You Missed Virtual Dispatch

- "Why isn't X showing up?" (data written but not displayed)
- "The buffer is correct but the screen is wrong"
- "It works in one mode but not another" (headless vs TTY)
- "I can see it in debug logs but not on screen"

If you see these, check: **Is a derived class method being called?**

## Lessons Learned

### 1. "Close Enough" Isn't Close Enough

If C++ does X and you do "something similar to X," you are NOT doing a 1:1 port. You're doing a reimplementation. Reimplementations have bugs.

### 2. Port the Execution Path, Not Just the Data Flow

It's not enough for the data to flow correctly through the system. You must also replicate HOW and WHEN methods are called.

C++ vtable → Rust explicit calls. Same behavior, different syntax.

### 3. The Shortcut Paradox

Taking shortcuts to "save time" cost us hours of debugging.

The correct 1:1 port took 5 lines and would have taken 5 minutes.

**The shortcut was more expensive.**

### 4. Document Your Failures

We wrote a comprehensive postmortem and updated our development guidelines. Now `CLAUDE.md` has:

- ⚠️ CRITICAL LESSON section at the top (impossible to miss)
- Detailed "C++ Inheritance → Rust Composition Patterns" section
- Checklist for identifying virtual methods
- Red flags for debugging missing dispatch

**The goal:** Make it impossible for future work to make the same mistake.

## The Technical Deep Dive

### Why Virtual Dispatch Matters

C++ classes with inheritance use virtual method tables (vtables):

```cpp
class Base {
    virtual void foo();  // Entry in vtable
};

class Derived : public Base {
    virtual void foo();  // Overrides vtable entry
};

Base *ptr = new Derived();
ptr->foo();  // Runtime lookup: calls Derived::foo()
```

At runtime:
1. `ptr->foo()` looks up the vtable
2. Finds `Derived::foo()` entry
3. Calls it automatically

### Rust Composition Doesn't Have Vtables

```rust
struct Base { /* ... */ }
struct Derived { base: Box<Base> }

impl Derived {
    fn foo(&mut self) { /* ... */ }
}

// Later...
screen.refresh(&caps);  // Walks tree, finds `derived.base`
// Calls `Base::refresh()` → `Base::foo()`
// NEVER calls `Derived::foo()`!
```

**No automatic dispatch.** You must hook it manually.

### The Correct Pattern

```rust
// BEFORE tree refresh
if derived.base.needs_work {
    derived.foo();              // Explicit dispatch
    derived.base.needs_work = true;  // Keep flag for tree
}

screen.refresh(&caps);          // Tree processes base
```

This replicates C++'s vtable behavior using explicit calls.

## Try It Yourself

The complete fix is in our repo:

- [Commit 08bcac2](https://github.com/selberhad/okros/commit/08bcac2) – OutputWindow virtual dispatch
- [Commit 253c332](https://github.com/selberhad/okros/commit/253c332) – InputLine virtual dispatch
- [Commit aa51b66](https://github.com/selberhad/okros/commit/aa51b66) – Input buffer clear
- [`DISPLAY_BUG_POSTMORTEM.md`](../DISPLAY_BUG_POSTMORTEM.md) – Full technical analysis

```bash
# Clone and see the pattern
git clone https://github.com/selberhad/okros.git
cd okros

# The fix is in main.rs:261-272
grep -A 12 "Composition workaround" src/main.rs
```

You'll see the pattern:
```rust
if output.win.dirty { output.redraw(); output.win.dirty = true; }
if input.win.dirty { input.redraw(); input.win.dirty = true; }
screen.refresh(&caps);
```

Nine lines that should have been there from day one.

## Conclusion

We set out to do a 1:1 port of C++ to Rust. We took what seemed like a reasonable shortcut: "composition is basically the same as inheritance."

It cost us:
- 3 display bugs
- Hours of debugging
- False leads on buffer logic, scrolling, and offsets
- Lots of good debug infrastructure (silver lining!)

The correct approach would have taken:
- 9 lines of code
- 5 minutes of work
- Zero debugging

**The moral?** When you're doing a 1:1 port, there are no shortcuts. "Close enough" is a bug. Port the complete execution path, including the parts that seem obvious.

And when C++ uses a vtable, Rust needs explicit calls. Always.

---

*This post documents our work on [okros](https://github.com/selberhad/okros), a Rust port of MCL (MUD Client for Linux). We learned the hard way that composition ≠ inheritance, so you don't have to.*

**Tech Stack:** Rust, ncurses, C++ inheritance patterns, composition patterns
**Time to introduce bug:** 30 seconds (skipping virtual dispatch on day 1)
**Time to fix bug:** 5 minutes (once we found it)
**Time to find bug:** Hours (debugging the wrong things)
**Lines of fix:** 9
**Lines of documentation:** This blog post + comprehensive postmortem

*Sometimes the best lessons come from our mistakes. This was one of those times.*

**Update (Oct 2025):** okros is now feature-complete! Both headless automation and TTY interactive modes working. All 203 tests passing. The virtual dispatch pattern is now baked into our development guidelines.
