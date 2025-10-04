# Testing Guide

## Running Tests

### Standard Test Suite
```bash
cargo test              # Run all tests
cargo test --lib        # Run only library unit tests
cargo test --tests      # Run integration tests
```

All tests should pass in any environment. Tests that require special conditions (like a TTY) will skip gracefully with a message.

**See [COVERAGE_REPORT.md](COVERAGE_REPORT.md) for current test counts and pass rates**

### Test Architecture

**In-Process Tests** (Preferred for Coverage):
- Unit tests run library functions directly (same process)
- Integration tests start servers in background threads (same process)
- Coverage tools (llvm-cov) track all code execution
- Example: `tests/control_inprocess.rs` - starts ControlServer in thread

**Subprocess Tests** (Legacy, E2E only):
- Some tests spawn `cargo run` as separate process
- Do NOT contribute to coverage metrics (different process)
- Kept for end-to-end validation only
- Example: `tests/offline_tty_smoke.rs` - validates binary works

### Test Results

All tests pass in any environment:
- **✅ 0 failures** - Tests skip gracefully when preconditions not met
- **✅ In-process tests** - Contribute to coverage metrics
- **✅ TTY tests** - Run when real terminal available (pseudo-TTY via `script`)

**For current test counts and coverage stats, see [COVERAGE_REPORT.md](COVERAGE_REPORT.md)**

## Special Test Requirements

### ncurses Tests

Three tests in `src/curses.rs` require a real TTY with terminfo database access:
- `curses::tests::test_init_curses`
- `curses::tests::test_get_acs_caps`
- `curses::tests::test_get_acs_codes`

**Behavior:**
- **No TTY**: Tests skip gracefully with message `"SKIP: ... requires a TTY"`
- **With TTY**: Tests run and validate ncurses functionality

**To run with fake TTY** (recommended - works with coverage):
```bash
# Build the faketty shim (one-time setup)
gcc -shared -fPIC -o tools/faketty.dylib tools/faketty.c

# Run tests with fake TTY (serial execution required for ncurses singleton)
DYLD_INSERT_LIBRARIES=./tools/faketty.dylib TERM=xterm-256color \
  cargo test --lib curses:: -- --test-threads=1 --nocapture
```

This uses DYLD_INTERPOSE to override `isatty()` and related functions, tricking ncurses into thinking it has a real TTY. Unlike `script`, this works in the same process and **does not break llvm-cov coverage**!

See [`tools/README.md`](tools/README.md) for technical details.

**Alternative: pseudo-TTY via `script`** (works but breaks coverage):
```bash
# Fallback if faketty.dylib doesn't work
env TERM=xterm-256color script -q /dev/null \
  cargo test --lib curses:: -- --test-threads=1 --nocapture
```

**Coverage Limitation (script only)**: The `script` command creates a child process, which breaks llvm-cov instrumentation. Tests pass but don't contribute to coverage metrics.

Expected output when tests run:
```
ACS_VLINE: 0x78
ACS_HLINE: 0x71
test curses::tests::test_get_acs_codes ... ok
```

### Python/Perl Plugin Tests

Feature-gated plugin tests require libraries to be in the dynamic linker path:

```bash
# Python (requires libpython3.x.dylib accessible)
cargo test --features python

# Perl (requires libperl.dylib accessible)
cargo test --features perl

# All features
cargo test --all-features
```

**Known Issue**: `cargo test --all-features` may fail with `dyld: Library not loaded` if Python/Perl shared libraries aren't in the runtime library path. This is an environment configuration issue, not a code issue.

**Workaround**: Tests work in `cargo run --features python/perl` because the runtime environment differs.

## Test Organization

### Unit Tests
- **Location**: `#[cfg(test)] mod tests` in each source file
- **Coverage**: 57 tests across all modules
- **Style**: Colocated with implementation

### Integration Tests
- **Location**: `tests/*.rs`
- **Coverage**:
  - `control_integration.rs` - Unix socket control server
  - `pipeline.rs` - MCCP→Telnet→ANSI→Scrollback pipeline
  - `pipeline_mccp.rs` - MCCP decompression (requires `mccp` feature)

## CI/CD Considerations

For automated CI environments:
1. **Base tests**: `cargo test` always passes (ncurses tests skip automatically)
2. **Feature tests**: May need library path configuration for Python/Perl
3. **No manual intervention required**: All tests are defensive and skip when preconditions aren't met

## Debugging Tests

### Verbose output
```bash
cargo test -- --nocapture           # Show println! output
cargo test -- --test-threads=1      # Run tests serially
```

### Specific test
```bash
cargo test socket::tests::nonblocking_connect_loopback
cargo test --lib curses::
```

### With backtrace
```bash
RUST_BACKTRACE=1 cargo test
```

## Test Coverage Summary

| Module | Tests | Status | Notes |
|--------|-------|--------|-------|
| ansi | 4 | ✅ | SGR parsing, bright colors |
| color | 2 | ✅ | Attribute constants |
| config | 1 | ✅ | Basic config |
| control | 4 | ✅ | JSON Lines protocol |
| **curses** | **3** | **✅ (skip)** | **Requires TTY** |
| engine | 1 | ✅ | Detach/attach |
| input | 5 | ✅ | Key decoding, ESC sequences |
| input_line | 4 | ✅ | Line editor |
| mccp | 2 | ✅ | Passthrough + flate2 |
| mud | 1 | ✅ | Socket wiring |
| output_window | 2 | ✅ | Rendering |
| screen | 5 | ✅ | Diff renderer, ACS |
| scrollback | 4 | ✅ | Ring buffer |
| select | 1 | ✅ | poll wrapper |
| selectable | 1 | ✅ | Interest flags |
| session | 1 | ✅ | Pipeline integration |
| socket | 2 | ✅ | Nonblocking connect |
| status_line | 1 | ✅ | Status rendering |
| telnet | 8 | ✅ | IAC, EOR, GA, SB |
| window | 1 | ✅ | Base widget |
| **Total** | **57** | **✅** | **All pass** |

## Adding New Tests

Follow these patterns:

### Unit test with TTY requirement
```rust
#[cfg(test)]
mod tests {
    fn has_tty() -> bool {
        unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 }
    }

    #[test]
    fn test_something_with_tty() {
        if !has_tty() {
            eprintln!("SKIP: requires TTY");
            return;
        }
        // ... test code
    }
}
```

### Integration test
```rust
// tests/my_integration_test.rs
use okros::*;

#[test]
fn test_end_to_end_flow() {
    // ... integration test
}
```

### Feature-gated test
```rust
#[cfg(feature = "python")]
#[test]
fn test_python_plugin() {
    // ... Python-specific test
}
```
