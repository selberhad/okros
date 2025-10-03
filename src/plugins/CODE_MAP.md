# CODE MAP — src/plugins/

Plugin system for embedded interpreters (Python, Perl). All modules implement the `Interpreter` trait defined in `stack.rs`.

## Modules

- `stack.rs` → `EmbeddedInterpreter.h` / `Embedded.cc`
  - Defines `Interpreter` trait (analogous to C++ `EmbeddedInterpreter` virtual class)
  - Implements `StackedInterpreter<I: Interpreter>` (chained interpreter execution; Toy 11 patterns)
  - Methods: `run()`, `run_quietly()`, `load_file()`, `eval()`, `set_int()`, `set_str()`, `get_int()`, `get_str()`
  - Enable/disable functions by name (matches C++ failed/disabled list behavior)

- `python.rs` (feature `python`) → `plugins/PythonEmbeddedInterpreter.cc`
  - Uses `pyo3` crate (simpler than raw C API; Toy 4 patterns)
  - Implements `Interpreter` trait for `PythonInterpreter`
  - Automatic reference counting (no manual INCREF/DECREF)
  - GIL management via `Python::with_gil()`

- `perl.rs` (feature `perl`) → `plugins/PerlEmbeddedInterpreter.cc`
  - Uses raw Perl C API FFI (Toy 5 patterns)
  - Implements `Interpreter` trait for `PerlInterpreter`
  - Requires `PERL_SYS_INIT3` for modern threaded Perl
  - Custom `build.rs` for Perl library linking via `perl -MConfig`

## Integration Notes

- **✅ Wired to main.rs** (main.rs:51-107, 227-271)
- Initialization behind `#[cfg(feature)]` guards (main.rs:51-79)
- Initial variables set: `now`, `VERSION`, `commandCharacter` (main.rs:87-106)
- Runs `sys/init` script on startup (main.rs:95, 105)
- Interpreter hooks in event loop:
  - `sys/postoutput` after I/O events (main.rs:227-240)
  - `sys/idle` on timer tick (main.rs:248-271)
- **Note**: Using separate Python/Perl instances instead of `StackedInterpreter` for MVP simplicity
  - C++ uses `StackedInterpreter` to chain interpreters (Embedded.cc:23-26)
  - Rust can refactor to use `StackedInterpreter<Box<dyn Interpreter>>` later if needed
