# Toy 6 — TTY + Keys (SPEC)

## Objective
Validate terminal raw mode, keypad application mode, and terminfo-driven key decoding to produce stable, normalized key events matching the C++ reference (`TTY.cc`, `Curses.cc`).

## Scope
- Termios raw setup/teardown (disable ECHO/ICANON/ISTRIP, input/output flags).
- Send "\e=" on init and "\e>" on restore (keypad app mode).
- Initialize terminfo and query key capability strings (`tigetstr`).
- Parse ESC-prefixed sequences and ALT/meta keys into canonical codes.
- Non-blocking read loop compatible with select/poll; handle partial sequences.

## Inputs
- Terminal: `$TERM`-driven capabilities; typical targets: `xterm`, `rxvt`, `linux`, `screen`, `tmux`, `iterm2`.
- User key presses: arrows, PgUp/Dn, Home/End, Insert/Delete, F1–F12, keypad Enter, ALT+letter.

## Outputs
- Line-delimited normalized events: `KEY <name> <code>` (e.g., `KEY arrow_up 1001`).
- Init/restore logs: `INIT done`, `RESTORE done`.

## Behavior
- On start: setupterm, enable keypad app mode, set raw termios, build key table:
  - Prefer `tigetstr("kcuu1"|...)`; fallback to related cap (e.g., `cuu1`) or hardcoded `"[A"` forms when missing.
  - Strip the initial ESC (`\e`) in capability strings to align with reference handling.
- Read loop: accumulate bytes up to `MAX_ESCAPE_BUF` (e.g., 64). If timeout/no match, return `escape` or raw byte.
- ALT/meta: interpret ESC + `<letter>` as `alt_<letter>`; handle 0x80 meta if present by normalizing to the same event.
- On exit: restore cooked termios and keypad numeric mode.

## Success Criteria
- All targeted keys produce stable names/codes across common terminals (xterm/rxvt/iTerm2/screen/tmux).
- No input loss when ESC sequences arrive fragmented across reads.
- App mode toggles do not leave the terminal in a bad state on crash (manual restore path tested).

## Test Plan
- Manual: Press each target key; verify printed name/code; test ALT+[a..z]; keypad Enter; Insert/Delete; F1–F12.
- Fragmentation: Simulate ESC sequence delivery in chunks (inject via a test harness or mocked reader) and assert normalization.
- Cap fallback: Temporarily ignore a cap name to force fallback path; confirm expected mapping still produced.

## Non-Goals
- Full UI integration, ncurses windows, or application event routing.
- Platform portability beyond Linux/macOS terminals referenced above.

