# Toy 2: ncurses FFI Integration - LEARNINGS

## Decision: Use `ncurses` crate (raw FFI bindings)

**Rationale**: MCL uses ncurses minimally - only for terminal setup and capability queries, NOT for window management. The `ncurses` crate provides exactly what we need: low-level FFI access to setupterm, tigetstr, and ACS codes.

---

## Key Discovery: MCL's Minimal ncurses Usage

**CRITICAL INSIGHT**: The C++ code does NOT use ncurses windows!

### What C++ MCL Actually Uses from ncurses:

1. **Terminal initialization**: `setupterm()`, `newterm()`
2. **Capability queries**: `tigetstr()` for terminal capabilities (smacs, rmacs, key sequences)
3. **ACS character codes**: `ACS_VLINE`, `ACS_HLINE`, border characters
4. **That's it!** No WINDOW* objects, no waddch/wprintw, no ncurses rendering

### What C++ MCL Does Instead:

**Custom Window System**:
- Windows are in-memory buffers (`attrib *canvas`)
- Each `attrib` = 16-bit value (high byte = color, low byte = char)
- Windows manage their own memory, copy to parent buffers

**Direct Terminal Output** (Screen.cc):
- **Virtual Console mode**: Direct writes to `/dev/vcsa` framebuffer
- **TTY mode**: ANSI escape sequences written to stdout
- No ncurses rendering functions used!

**Why this matters for Rust port**:
- We don't need high-level ncurses wrappers (like pancurses)
- We just need FFI access to terminal info functions
- Custom window system → implement in pure Rust
- Direct ANSI output → use Rust `write!` to stdout

---

## Answers to Learning Goals

### 1. Which Rust ncurses binding to use?

**ANSWER: `ncurses` crate (raw FFI bindings)**

**Comparison**:

| Feature | `ncurses` crate | `pancurses` crate |
|---------|----------------|-------------------|
| setupterm/tigetstr | ✅ Direct access | ❌ Not exposed, need manual FFI |
| ACS character codes | ✅ After newterm() | ❌ Requires full window init |
| Low-level access | ✅ All C functions | ❌ Hidden behind abstractions |
| Window management | N/A (not needed) | High-level (not needed) |
| Complexity | Simple for our use | Adds unnecessary layers |

**Decision**: Use `ncurses` crate for low-level FFI, skip pancurses entirely.

---

### 2. How to wrap ncurses state in Rust?

**ANSWER: Minimal global state, no ncurses wrappers needed**

**Pattern** (matching C++ approach):
```rust
// Store capability strings and ACS codes globally
static mut SMACS: Option<Vec<u8>> = None;
static mut RMACS: Option<Vec<u8>> = None;
static mut SPECIAL_CHARS: [u8; MAX_SC] = [0; MAX_SC];
static mut REAL_SPECIAL_CHARS: [char; MAX_SC] = ['\0'; MAX_SC];

// Initialization function
unsafe fn init_curses(virtual_console: bool) {
    // Call setupterm
    // Call newterm
    // Query tigetstr for capabilities
    // Store ACS codes
}
```

**No ncurses window wrappers** - we implement our own window system in Rust like C++ does.

---

### 3. How to handle ncurses initialization/cleanup?

**ANSWER: Manual init/cleanup, matching C++ pattern exactly**

**C++ pattern**:
```c++
void init_curses(bool virtualConsole) {
    setupterm(term, STDOUT_FILENO, &err);
    FILE *fp = fopen("/dev/null", "r+");
    newterm(term, fp, fp);
    // Store ACS codes and capabilities
}
```

**Rust pattern**:
```rust
unsafe fn init_curses(virtual_console: bool) -> Result<(), String> {
    let term = CString::new(env::var("TERM")?)?;
    let mut errret = 0;

    setupterm(term.as_ptr(), STDOUT_FILENO, &mut errret);
    if errret != 1 {
        return Err(format!("setupterm failed: {}", errret));
    }

    let fp = fopen(c"/dev/null".as_ptr(), c"r+".as_ptr());
    let screen = newterm(term.as_ptr() as *mut _, fp, fp);
    if screen.is_null() {
        return Err("newterm failed".into());
    }

    // Now ACS codes are populated
    SPECIAL_CHARS[bc_vertical] = ACS_VLINE() as u8;
    // ... store other codes

    Ok(())
}
```

**No RAII/Drop** - cleanup is manual when app exits, matching C++ pattern.

---

### 4. How to map C++ Window operations?

**ANSWER: Port Window class to Rust struct, no ncurses needed**

**Key insight**: C++ Window class is pure in-memory operations!

```rust
struct Window {
    parent: Option<*mut Window>,
    width: usize,
    height: usize,
    canvas: Vec<u16>,  // attrib array (color << 8 | char)
    cursor_x: usize,
    cursor_y: usize,
    color: u8,
    dirty: bool,
    // ... other fields
}

impl Window {
    fn print(&mut self, s: &str) {
        // Parse string, write to canvas buffer
        // NO ncurses calls!
    }

    fn copy(&mut self, source: &[u16], w: usize, h: usize, x: usize, y: usize) {
        // memcpy-style operation on canvas
        // NO ncurses calls!
    }
}
```

**Rendering happens separately** in Screen class using ANSI escapes.

---

### 5. How to handle ncurses errors?

**ANSWER: Match C++ - minimal error handling**

C++ mostly ignores errors or uses assertions. We should:
- Check setupterm/newterm return values (critical)
- Ignore most other errors (matching C++ behavior)
- Use `unsafe` blocks liberally (matching "safety third" approach)

---

### 6. Color/Attribute handling?

**ANSWER: Use raw u8 values like C++, add Rust constants**

**C++ approach**:
```c++
typedef unsigned short int attrib;  // high byte = color, low byte = char
#define fg_white 7
#define bg_black 0
```

**Rust approach**:
```rust
type Attrib = u16;  // High byte = color, low byte = char

// Color constants (bitflags optional but not required)
pub const FG_BLACK: u8 = 0;
pub const FG_RED: u8 = 1;
// ... etc

pub const BG_BLACK: u8 = 0 << 4;
pub const BG_BLUE: u8 = 4 << 4;
// ... etc

pub const FG_BOLD: u8 = 0x08;

// Combine: color = fg | bg | FG_BOLD
// Store in attrib: attrib = (color as u16) << 8 | (char as u16)
```

**Simple, direct mapping** from C++ constants.

---

## Validated Patterns

### Pattern 1: Terminal Initialization

**Tested in**: `complete_test.rs`

```rust
extern "C" {
    fn setupterm(term: *const c_char, fd: c_int, errret: *mut c_int) -> c_int;
    fn newterm(term: *mut c_char, outfd: *mut FILE, infd: *mut FILE) -> *mut c_void;
    fn tigetstr(capname: *mut c_char) -> *const c_char;
}

unsafe fn init_terminal() -> Result<(), String> {
    // 1. Call setupterm
    let term = CString::new(env::var("TERM")?)?;
    let mut err = 0;
    setupterm(term.as_ptr(), STDOUT_FILENO, &mut err);
    if err != 1 { return Err("setupterm failed".into()); }

    // 2. Open /dev/null for ncurses (it won't write to our terminal)
    let fp = fopen(c"/dev/null".as_ptr(), c"r+".as_ptr());
    if fp.is_null() { return Err("fopen failed".into()); }

    // 3. Call newterm (initializes ncurses, populates ACS codes)
    let screen = newterm(term.as_ptr() as *mut _, fp, fp);
    if screen.is_null() { return Err("newterm failed".into()); }

    Ok(())
}
```

**Result**: ✅ Works perfectly, matches C++ behavior

---

### Pattern 2: Querying Terminal Capabilities

**Tested in**: `ncurses_test.rs`, `complete_test.rs`

```rust
unsafe fn get_capability(name: &str) -> Option<Vec<u8>> {
    let cap = CString::new(name).ok()?;
    let ptr = tigetstr(cap.as_ptr() as *mut _);

    if ptr.is_null() || ptr as isize == -1 {
        return None;
    }

    Some(CStr::from_ptr(ptr).to_bytes().to_vec())
}

// Usage:
let smacs = get_capability("smacs");  // Enable ACS
let rmacs = get_capability("rmacs");  // Disable ACS
let kf1 = get_capability("kf1");      // F1 key sequence
```

**Result**: ✅ All capabilities retrieved correctly

---

### Pattern 3: ACS Character Codes

**Tested in**: `complete_test.rs`

```rust
// After calling newterm(), ACS codes are available
unsafe fn get_acs_codes() -> [char; MAX_SC] {
    let mut codes = ['\0'; MAX_SC];

    // Extract low byte (actual character code)
    codes[BC_VERTICAL] = (ncurses::ACS_VLINE() & 0xFF) as u8 as char;
    codes[BC_HORIZONTAL] = (ncurses::ACS_HLINE() & 0xFF) as u8 as char;
    codes[BC_UPPER_LEFT] = (ncurses::ACS_ULCORNER() & 0xFF) as u8 as char;
    // ... etc

    codes
}
```

**Result**: ✅ ACS codes: VLINE=0x78, HLINE=0x71, ULCORNER=0x6c, etc.

---

### Pattern 4: Direct ANSI Output (No ncurses rendering)

**Tested in**: All test files

```rust
use std::io::{self, Write};

fn render_to_terminal(canvas: &[Attrib], width: usize, height: usize) {
    let mut stdout = io::stdout();

    // Move cursor to home
    write!(stdout, "\x1b[H").unwrap();

    let mut current_color = 0xFF; // Invalid to force first set

    for y in 0..height {
        for x in 0..width {
            let attrib = canvas[y * width + x];
            let color = (attrib >> 8) as u8;
            let ch = (attrib & 0xFF) as u8 as char;

            // Set color if changed
            if color != current_color {
                let fg = color & 0x0F;
                let bg = color >> 4;
                write!(stdout, "\x1b[0;{};{}m", 30 + fg, 40 + bg).unwrap();
                current_color = color;
            }

            // Write character
            write!(stdout, "{}", ch).unwrap();
        }
    }

    stdout.flush().unwrap();
}
```

**Result**: ✅ Direct ANSI output works, no ncurses rendering needed

---

## Implementation Recommendations

### For Production Port (Tier 3 - UI Layer)

1. **Create minimal ncurses wrapper** (`src/curses.rs`):
   - `init_curses()` - setupterm + newterm
   - `get_capability()` - tigetstr wrapper
   - Store ACS codes and capabilities in global statics

2. **Port Window class** (`src/window.rs`):
   - Pure Rust, no ncurses dependencies
   - In-memory canvas operations
   - Parent/child relationships
   - Dirty tracking

3. **Port Screen class** (`src/screen.rs`):
   - Inherit from Window
   - Implement `refresh()` with ANSI output (skip /dev/vcsa for now)
   - Direct writes to stdout
   - Diff-based rendering

4. **Global state pattern**:
```rust
// Use lazy_static or once_cell for safe initialization
static SPECIAL_CHARS: Lazy<[u8; MAX_SC]> = Lazy::new(|| {
    // Initialize after newterm()
    get_acs_codes()
});
```

---

## Test Results Summary

### ✅ ncurses crate capabilities:
- setupterm: Works
- tigetstr: Works
- ACS codes: Works after newterm()
- Low-level FFI: Complete access
- **Verdict**: Sufficient for MCL port

### ❌ pancurses crate:
- No low-level terminfo access without manual FFI
- ACS codes hidden behind window init
- Unnecessary abstraction for our needs
- **Verdict**: Skip, not needed

---

## Key Takeaways

1. **MCL uses ncurses minimally** - only for terminal info, not rendering
2. **Custom window system** - port Window class to pure Rust
3. **Direct ANSI output** - no ncurses window functions needed
4. **ncurses crate is perfect** - provides exact FFI we need
5. **Skip pancurses** - too much abstraction, not helpful

---

## Files in This Toy

- `src/ncurses_test.rs` - Basic terminfo access test
- `src/complete_test.rs` - Full initialization pattern (matching C++)
- `src/pancurses_test.rs` - Comparison test (shows pancurses limitations)

---

## Next Steps (Moving to Production)

Ready to port Tier 3 (UI Layer):
1. Port `Curses.cc` → `src/curses.rs` (minimal wrapper)
2. Port `Window.cc` → `src/window.rs` (pure Rust)
3. Port `Screen.cc` → `src/screen.rs` (ANSI output)
4. Use `ncurses = "5.101"` dependency
5. Apply patterns validated in this toy

**Estimated complexity**: Low - patterns are clear, no surprises expected.
