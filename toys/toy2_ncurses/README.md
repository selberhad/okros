# Toy 2: ncurses FFI Integration

## Purpose

Validate ncurses FFI patterns for MCL Rust port. Key discovery: **MCL doesn't use ncurses windows** - only terminal setup and capability queries!

## Key Findings

- ✅ **Use `ncurses` crate** (raw FFI bindings)
- ✅ **Skip `pancurses`** (unnecessary abstraction)
- ✅ **Custom window system** (port Window class to pure Rust)
- ✅ **Direct ANSI output** (no ncurses rendering needed)

## Tests

Run the tests to see validated patterns:

```bash
# Basic terminfo access
cargo run --bin ncurses_test

# Complete initialization (matches C++ pattern)
cargo run --bin complete_test

# Pancurses comparison (shows why we skip it)
cargo run --bin pancurses_test
```

## What MCL Actually Uses from ncurses

1. Terminal initialization: `setupterm()`, `newterm()`
2. Capability queries: `tigetstr()` for smacs/rmacs/keys
3. ACS character codes: `ACS_VLINE`, `ACS_HLINE`, etc.
4. **Nothing else!** No WINDOW*, no waddch/wprintw

## Implementation Pattern

```rust
// 1. Initialize terminal
unsafe {
    setupterm(term.as_ptr(), STDOUT_FILENO, &mut errret);
    let fp = fopen(c"/dev/null".as_ptr(), c"r+".as_ptr());
    newterm(term.as_ptr() as *mut _, fp, fp);
}

// 2. Get ACS codes (now available)
let vline = (ncurses::ACS_VLINE() & 0xFF) as u8;

// 3. Query capabilities
let smacs = tigetstr(c"smacs".as_ptr() as *mut _);

// 4. Direct ANSI output (no ncurses rendering)
write!(stdout, "\x1b[31mRed text\x1b[0m")?;
```

## See LEARNINGS.md

Comprehensive documentation of all validated patterns and decisions.

## Status

✅ **Complete** - Ready for production port (Tier 3 - UI Layer)
