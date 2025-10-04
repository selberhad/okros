# Development Tools

## faketty - Fake TTY Library for Testing

A `DYLD_INTERPOSE` library that tricks programs into thinking they have a TTY,
allowing TTY-dependent tests to run without using `script` (which breaks llvm-cov).

### How It Works

Uses macOS's `DYLD_INTERPOSE` mechanism to override `isatty()` and related functions:
- `isatty()` always returns 1 (claims we have a TTY)
- `tcgetattr()` / `tcsetattr()` provide fake terminal attributes
- `ioctl(TIOCGWINSZ)` reports 80x24 terminal size

### Building

```bash
gcc -shared -fPIC -o faketty.dylib faketty.c
```

### Usage

```bash
# Run TTY-dependent tests
DYLD_INSERT_LIBRARIES=./tools/faketty.dylib TERM=xterm-256color \
  cargo test --lib curses:: -- --test-threads=1

# Example output:
#   running 3 tests
#   ACS_VLINE: 0x78
#   ACS_HLINE: 0x71
#   test curses::tests::test_get_acs_codes ... ok
#   test curses::tests::test_get_acs_caps ... ok
#   test curses::tests::test_init_curses ... ok
```

### Why Not LD_PRELOAD?

- macOS uses `DYLD_INSERT_LIBRARIES` not `LD_PRELOAD`
- Simple function replacement doesn't work due to symbol precedence
- `DYLD_INTERPOSE` is the macOS-specific mechanism for function interposition

### Limitations

- **macOS only**: Uses macOS-specific `DYLD_INTERPOSE`
- **System Integrity Protection (SIP)**: May not work on SIP-protected binaries
- **Coverage quirks**: Some tests may behave differently when isatty() is faked
- **Serial execution**: ncurses is a global singleton, must use `--test-threads=1`

### Known Issues

- Perl plugin tests may fail when faketty is loaded (symbol conflicts?)
- llvm-cov coverage collection works but needs more validation
- Some ncurses features may not work (uses minimal fake termios)

### Success Criteria

✅ TTY tests run successfully
✅ ncurses initializes properly
✅ ACS codes work (line drawing characters)
⚠️ Coverage collection needs more testing

This proves that **in 2025, yes, we CAN emulate a TTY for testing!**
