# MUD Learnings: Nodeka Integration

Lessons learned from integrating okros with Nodeka MUD (nodeka.com:23).

## Test Methodology

**Goal**: Turn real MUD bugs into reproducible test cases.

### Process

1. **Enable debug logging** - Add `eprintln!()` to capture problematic bytes
2. **Interact with MUD** - Note what looks wrong (missing text, wrong colors, etc.)
3. **Extract bytes from logs** - Copy exact sequences that triggered bug
4. **Create test case** - Use real MUD bytes in unit test
5. **Fix bug** - Verify test passes
6. **Re-test with real MUD** - Confirm fix works in production

### Example

```rust
#[test]
fn nodeka_menu_colors() {
    // Real Nodeka output from debug logs
    let nodeka_line = b"\x1b[41m \x1b[0m \x1b[1;37mWelcome\x1b[0m\n\r";

    let mut ses = Session::new(PassthroughDecomp::new(), 80, 3, 100);
    ses.feed(nodeka_line);

    let v = ses.scrollback.viewport_slice();
    let text: String = v[0..80].iter().map(|a| (a & 0xFF) as u8 as char).collect();
    assert!(text.contains("Welcome"));
}
```

**Benefits**: Fast, reproducible, no network required, prevents regressions.

## Key Findings

### 1. Per-Character Color Storage

**Problem**: Nodeka menus showed black-on-black.

**Root Cause**: We stored one color per line, but Nodeka uses multiple colors per line.

**Example** (single line with 5 color changes):
```
\x1b[41m \x1b[0m \x1b[1;37mWelcome to Nodeka\x1b[0m: \x1b[41m \x1b[0m\n\r
```

**Fix**: Store `(char, color)` pairs like C++ MCL's SET_COLOR stream:
```rust
// Before: line_buf: Vec<u8>, cur_color: u8
// After:  line_buf: Vec<(u8, u8)>
```

**Commit**: `fix(session): store per-character colors like C++ MCL`

### 2. Circular Buffer Flattening

**Problem**: Menu text showed as NULL bytes in headless mode.

**Root Cause**: `recent_lines()` read from offset 0 (old data) instead of following `canvas_off` pointer.

**Fix**: Rewrote `recent_lines()` to flatten circular buffer, handle wraparound correctly.

**Result**: Welcome screen now works (was NULL bytes).

**Commit**: `fix(scrollback): flatten circular buffer for headless mode`

### 3. Hex Dump Debug Tool

Added `hex` command to control protocol for low-level debugging:

```bash
echo '{"cmd":"hex","lines":20}' | nc -U /tmp/okros/instance.sock
```

**Output**:
```json
{
  "event": "Hex",
  "lines": [{
    "hex": "48:07 65:07 6c:07",
    "text": "Hel",
    "colors": "07 07 07"
  }]
}
```

Reveals actual scrollback storage (char + color per cell). Helped find circular buffer bug.

**Commit**: `feat(headless): add hex dump mode for debugging`

## ANSI Sequence Handling

### Supported (Line-Oriented)

| Sequence | Purpose | Status |
|----------|---------|--------|
| `ESC[Nm` | Color codes | ✅ Full support |
| `ESC[0m` | Reset | ✅ Works |
| `ESC[1m` | Bold | ✅ Works |
| `ESC[30-37m` | Foreground | ✅ Works |
| `ESC[40-47m` | Background | ✅ Works |
| `IAC GA/EOR` | Telnet prompts | ✅ Works |
| `\r` | Carriage return | ✅ Discarded (C++ compat) |

### Ignored (Not Line-Oriented)

| Sequence | Purpose | Why Ignored |
|----------|---------|-------------|
| `ESC[H` | Cursor home | Line-oriented scrollback |
| `ESC[row;colH` | Cursor position | Line-oriented scrollback |
| `ESC[J` | Clear screen | May need handling (TODO) |
| `ESC[K` | Clear line | May need handling (TODO) |

**Note**: okros is line-oriented, not 2D canvas. Cursor positioning used for full-screen menus cannot be captured in headless mode (fundamental limitation).

## C++ MCL vs Rust okros

| Aspect | C++ MCL | Rust okros | Match? |
|--------|---------|------------|--------|
| Color storage | SET_COLOR markers | `(char, color)` pairs | ✅ Equivalent |
| Line buffering | `out_buf` with markers | `Vec<(u8, u8)>` | ✅ Equivalent |
| ANSI parsing | `colorConverter.convert()` | `AnsiConverter::feed()` | ✅ Works |
| Prompt detection | IAC GA/EOR | TelnetParser + GA/EOR | ✅ Works |
| `\r` handling | Discarded | Discarded | ✅ Match |

## Current Status

### Working
- ✅ Welcome screen (ASCII art, colors)
- ✅ Per-character color changes
- ✅ Circular buffer flattening
- ✅ Prompts (with/without GA/EOR)
- ✅ Login flow

### Test Character
- **Username**: `Locus` (stored in `.env`)
- **Password**: `SecurePass2024!@#` (stored in `.env`)

## Helper Tools

### Debug Logging
```rust
// src/session.rs
eprintln!("SESSION feed: {} bytes: {:?}", data.len(), String::from_utf8_lossy(data));
```

### Hex Dump
```bash
echo '{"cmd":"hex","lines":10}' | nc -U /tmp/okros/instance.sock | jq -r '.lines[0].hex'
```

### MUD Command Script
```bash
./scripts/mud_cmd.sh /tmp/okros/instance.sock "look"
```

## References

- C++ MCL: `mcl-cpp-reference/Session.cc` (lines 524-580 for ANSI processing)
- Tests: `src/session.rs::tests::nodeka_menu_colors`
- Related: `AGENT_GUIDE.md` for MUD interaction best practices
