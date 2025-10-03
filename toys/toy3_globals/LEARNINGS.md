# Toy 3: Global State Management - LEARNINGS

## Decision: Use `unsafe static mut` with Helper Functions

**Pattern validated**: Raw pointers with clean accessor functions (Option 2)

**Rationale**: Single-threaded app, 1:1 C++ mapping, zero overhead, clean call sites

---

## Answers to Learning Goals

### 1. Does `unsafe static mut` with helper functions work cleanly?

**ANSWER: YES - Works perfectly ✅**

**Validated pattern**:
```rust
// In src/globals.rs
static mut SCREEN: *mut Screen = ptr::null_mut();
static mut CONFIG: *mut Config = ptr::null_mut();

pub fn screen() -> &'static mut Screen {
    unsafe { SCREEN.as_mut().expect("Screen not initialized") }
}

pub fn config() -> &'static mut Config {
    unsafe { CONFIG.as_mut().expect("Config not initialized") }
}

// Usage anywhere (just like C++!)
screen().refresh();
config().get_option(opt_beep);
```

**Test results**:
- ✅ Clean call sites (no `unsafe` blocks scattered everywhere)
- ✅ Identical ergonomics to C++ global usage
- ✅ Panic on uninitialized access (better than C++ segfault)
- ✅ Zero runtime overhead
- ✅ Works across multiple functions

**No gotchas found** - pattern is straightforward and robust.

---

### 2. Raw pointers vs OnceCell vs lazy_static?

**ANSWER: Raw `static mut *mut T` wins for MCL**

**Comparison**:

| Approach | Pros | Cons | Verdict |
|----------|------|------|---------|
| **Raw `static mut`** | ✅ Simple<br>✅ Zero overhead<br>✅ 1:1 C++ mapping | ⚠️ Requires `unsafe` init | **Winner** |
| **OnceCell + UnsafeCell** | ✅ Double-init protection | ❌ Sync issues (UnsafeCell not Sync)<br>❌ Needs wrapper struct<br>❌ More boilerplate | Too complex |
| **lazy_static + Mutex** | ✅ Thread-safe | ❌ Runtime locking overhead<br>❌ MCL is single-threaded<br>❌ Over-engineered | Not needed |

**OnceCell problem** (discovered in testing):
```rust
// This doesn't compile!
static SCREEN: OnceCell<UnsafeCell<Screen>> = OnceCell::new();
// Error: `UnsafeCell<Screen>` cannot be shared between threads safely
```

You'd need a wrapper struct implementing `Sync`, which defeats the simplicity goal.

**Winner**: Raw `static mut` - embraces "safety third" philosophy

---

### 3. Initialization order and safety?

**ANSWER: Explicit `init_globals()` call in main() ✅**

**Validated pattern**:
```rust
// In src/globals.rs
pub unsafe fn init_globals() {
    if !SCREEN.is_null() || !CONFIG.is_null() {
        panic!("Globals already initialized!");
    }

    SCREEN = Box::leak(Box::new(Screen::new(80, 24)));
    CONFIG = Box::leak(Box::new(Config::new()));
}

// In main.rs
fn main() {
    unsafe {
        init_globals();
    }

    // Now safe to use globals
    run_event_loop();
}
```

**Test results**:
- ✅ Double-init protection (optional, but nice)
- ✅ Panic on uninitialized access (`.expect()` in helper functions)
- ✅ Clear initialization point in code
- ✅ Better than C++ (explicit vs implicit initialization)

**Safety check recommendation**: Keep the double-init check - costs nothing, catches bugs

---

### 4. Nullable vs non-nullable globals?

**ANSWER: Different patterns for each ✅**

**Non-nullable globals** (Screen, Config):
```rust
static mut SCREEN: *mut Screen = ptr::null_mut();

pub fn screen() -> &'static mut Screen {
    unsafe { SCREEN.as_mut().expect("Screen not initialized") }
}

// Usage: always available after init
screen().refresh();
```

**Nullable globals** (currentSession):
```rust
static mut CURRENT_SESSION: *mut Session = ptr::null_mut();

pub fn current_session() -> Option<&'static mut Session> {
    unsafe { CURRENT_SESSION.as_mut() }  // No .expect()!
}

pub fn set_current_session(session: Box<Session>) {
    unsafe {
        if !CURRENT_SESSION.is_null() {
            let _ = Box::from_raw(CURRENT_SESSION);  // Drop old
        }
        CURRENT_SESSION = Box::leak(session);
    }
}

pub fn clear_current_session() {
    unsafe {
        if !CURRENT_SESSION.is_null() {
            let _ = Box::from_raw(CURRENT_SESSION);
            CURRENT_SESSION = ptr::null_mut();
        }
    }
}

// Usage: check with if let
if let Some(sess) = current_session() {
    sess.send("hello");
}
```

**Test results**:
- ✅ Non-nullable: `expect()` enforces "must be initialized"
- ✅ Nullable: returns `Option`, clean ergonomics
- ✅ Setter handles cleanup of old value
- ✅ Mirrors C++ semantics perfectly

---

### 5. Module organization?

**ANSWER: Single `globals.rs` module ✅**

**Recommended structure**:
```
src/
  globals.rs       // All global state + accessors
  screen.rs        // Screen implementation
  config.rs        // Config implementation
  session.rs       // Session implementation
  main.rs
```

```rust
// src/globals.rs
static mut SCREEN: *mut Screen = ptr::null_mut();
static mut CONFIG: *mut Config = ptr::null_mut();
static mut CURRENT_SESSION: *mut Session = ptr::null_mut();

pub fn screen() -> &'static mut Screen { ... }
pub fn config() -> &'static mut Config { ... }
pub fn current_session() -> Option<&'static mut Session> { ... }

pub unsafe fn init_globals() { ... }
```

**Rationale**:
- All global state in one place (easy to audit)
- Clear what's global vs what's not
- Single import: `use crate::globals::*;`
- Matches C++ pattern (globals in one header)

**Alternative** (per-global modules) would scatter globals, making them harder to track.

---

### 6. Do we need getter/setter distinction?

**ANSWER: Separate functions for clarity ✅**

**Pattern**:
```rust
// Getter (returns reference)
pub fn current_session() -> Option<&'static mut Session> { ... }

// Setter (takes ownership)
pub fn set_current_session(session: Box<Session>) { ... }

// Clear
pub fn clear_current_session() { ... }
```

**Alternative rejected**:
```rust
// Single function returning mutable Option
pub fn current_session_mut() -> &'static mut Option<Box<Session>> { ... }
// Usage: *current_session_mut() = Some(...); // Ugly!
```

**Rationale**: Separate functions are clearer and more idiomatic

---

## Production Pattern (Final Recommendation)

### File: `src/globals.rs`

```rust
use std::ptr;
use crate::{Screen, Config, Session};

// =============================================================================
// Global state
// =============================================================================

static mut SCREEN: *mut Screen = ptr::null_mut();
static mut CONFIG: *mut Config = ptr::null_mut();
static mut CURRENT_SESSION: *mut Session = ptr::null_mut();

// =============================================================================
// Non-nullable global accessors
// =============================================================================

/// Get mutable reference to global Screen
pub fn screen() -> &'static mut Screen {
    unsafe {
        SCREEN.as_mut().expect("Screen not initialized! Call init_globals() first")
    }
}

/// Get mutable reference to global Config
pub fn config() -> &'static mut Config {
    unsafe {
        CONFIG.as_mut().expect("Config not initialized! Call init_globals() first")
    }
}

// =============================================================================
// Nullable global accessors
// =============================================================================

/// Get mutable reference to current session (if any)
pub fn current_session() -> Option<&'static mut Session> {
    unsafe { CURRENT_SESSION.as_mut() }
}

/// Set the current session (takes ownership, drops old session if present)
pub fn set_current_session(session: Box<Session>) {
    unsafe {
        // Clean up old session if exists
        if !CURRENT_SESSION.is_null() {
            let _ = Box::from_raw(CURRENT_SESSION);
        }
        CURRENT_SESSION = Box::leak(session);
    }
}

/// Clear the current session (drops it)
pub fn clear_current_session() {
    unsafe {
        if !CURRENT_SESSION.is_null() {
            let _ = Box::from_raw(CURRENT_SESSION);
            CURRENT_SESSION = ptr::null_mut();
        }
    }
}

// =============================================================================
// Initialization
// =============================================================================

/// Initialize global state once at startup
///
/// # Safety
/// Must be called exactly once before any global access
/// Must be called from single-threaded context (before spawning any threads)
pub unsafe fn init_globals() {
    // Double-init check
    if !SCREEN.is_null() || !CONFIG.is_null() {
        panic!("Globals already initialized!");
    }

    // Create and leak (never deallocated)
    SCREEN = Box::leak(Box::new(Screen::new()));
    CONFIG = Box::leak(Box::new(Config::new()));

    // currentSession starts as null (created later)
}
```

### File: `src/main.rs`

```rust
mod globals;
use globals::*;

fn main() {
    // Initialize globals first thing
    unsafe {
        init_globals();
    }

    // Now use globals anywhere
    screen().refresh();
    config().set_option(opt_beep, true);

    run_event_loop();
}

fn run_event_loop() {
    loop {
        handle_input();
        screen().refresh();
    }
}

fn handle_input() {
    if let Some(sess) = current_session() {
        sess.send("look");
    }
}
```

---

## Test Results Summary

**From `raw_static_test.rs`**:

```
Test 1: Initialization ✓
Test 2: Basic global access ✓
Test 3: Global mutation ✓
Test 4: Nullable global (no session) ✓
Test 5: Create session ✓
Test 6: Use session ✓
Test 7: Multiple operations ✓
Test 8: Clear nullable global ✓
Test 9: Reconnect ✓
```

All patterns work as expected!

---

## Key Takeaways

1. **`unsafe static mut` with helper functions is the winner**
   - Clean call sites (no scattered `unsafe` blocks)
   - Zero overhead
   - Perfect 1:1 C++ mapping

2. **Helper functions hide all the unsafe**
   - One `unsafe` block per accessor
   - User code looks safe and clean

3. **Nullable vs non-nullable distinction is important**
   - Non-nullable: `expect()` enforces initialization
   - Nullable: return `Option`, natural Rust idiom

4. **OnceCell adds complexity with no benefit**
   - Sync issues with UnsafeCell
   - MCL is single-threaded, doesn't need it

5. **Single `globals.rs` module is cleanest**
   - All global state in one place
   - Easy to audit and understand

6. **Double-init check is worth keeping**
   - Costs nothing
   - Catches initialization bugs

---

## Files in This Toy

- `src/raw_static_test.rs` - Validated pattern (WORKS ✅)
- `src/oncecell_test.rs` - OnceCell comparison (Sync issues, complex)

---

## Next Steps (Moving to Production)

Ready to implement in main port:

1. Create `src/globals.rs` with pattern above
2. Add globals as needed:
   - `screen()`
   - `config()`
   - `current_session()`
   - Others as discovered during port

3. Call `unsafe { init_globals() }` in `main()`
4. Use globals freely with clean syntax

**Estimated complexity**: Low - pattern is proven and straightforward

---

## Philosophy Note

This pattern embraces **"safety third"**:
- MCL is single-threaded → no data races
- Globals init once at startup → no threading issues
- Pattern matches C++ exactly → behavioral equivalence
- `unsafe` is contained → not scattered through codebase

For a 1:1 port of single-threaded C++ code, this is the pragmatic choice.
