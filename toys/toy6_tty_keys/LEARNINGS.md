# Toy 6 — TTY + Keys: LEARNINGS

## Step 0 — Learning Goals
- Termios raw mode: Which flags mirror C++ (`ECHO`, `ICANON`, `ISTRIP`, input/output flags), and macOS/Linux portability gotchas?
- Keypad app mode: Do sending "\e=" and "\e>" reliably toggle keypad application mode across common terminals?
- Terminfo caps: Which `tigetstr` names are required and present (e.g., `kcuu1`, `kcud1`, `kend`, `kich1`)? Robust fallback strategy when a cap is missing.
- ALT/meta handling: Normalize ALT+letter and ESC-prefixed sequences; handle 0x80 meta behavior observed in some xterms.
- Buffering limits: Safe max escape buffer length to avoid truncation/DoS; partial-read behavior in non-blocking mode.
- Canonical map: Produce stable key codes/names (arrows, PgUp/Dn, Home/End, F1–F12, Insert/Delete, keypad enter) matching reference semantics.
- Event loop fit: Ensure read path integrates with select/poll without losing partial sequences.

