# src/ — Code Map

Initial scaffold for Execution Phase per IMPLEMENTATION_PLAN.md.

## Files

- `main.rs` — Entry point (stub). Prints feature flags; TODO: init globals/ncurses.
- `globals.rs` — Placeholder for Toy 3 globals pattern.
- `color.rs` — Placeholder color/attribute constants (bitflags).

## Planned Structure (excerpt)

- `string.rs`, `buffer.rs` — Thin adapters over `String`/`Vec<u8>`
- `curses.rs`, `window.rs`, `screen.rs` — UI layer using `ncurses`
- `config.rs`, `socket.rs`, `mud.rs` — Core layer
- `session.rs`, `alias.rs`, `interpreter.rs` — Logic layer

