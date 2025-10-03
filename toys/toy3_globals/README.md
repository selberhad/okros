# Toy 3: Global State Management

## Purpose

Validate global state patterns for MCL Rust port. MCL has many globals (Screen, Config, currentSession, etc.) that need Rust equivalents.

## Key Finding

**Use `unsafe static mut` with helper functions** - simple, zero overhead, clean call sites

## Pattern

```rust
// In src/globals.rs
static mut SCREEN: *mut Screen = ptr::null_mut();

pub fn screen() -> &'static mut Screen {
    unsafe { SCREEN.as_mut().expect("Screen not initialized") }
}

pub unsafe fn init_globals() {
    SCREEN = Box::leak(Box::new(Screen::new()));
}

// Usage anywhere (just like C++!)
screen().refresh();
```

## Why This Wins

✅ Clean call sites (no `unsafe` everywhere)
✅ Zero runtime overhead
✅ Perfect 1:1 C++ mapping
✅ Panic on uninitialized access (better than C++ segfault)
✅ Single-threaded app = no data races possible

## Run the Test

```bash
cargo run --bin raw_static_test
```

## See LEARNINGS.md

Complete documentation of:
- Pattern comparison (raw vs OnceCell vs lazy_static)
- Nullable vs non-nullable patterns
- Initialization strategy
- Module organization
- Production-ready code templates

## Status

✅ **Complete** - Pattern validated and ready for production use
