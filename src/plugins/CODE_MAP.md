# CODE MAP — src/plugins/

- `stack.rs` → Stacked interpreter utility (consolidates behavior from `EmbeddedInterpreter` layering in C++).

Planned (feature-gated)
- `python.rs` (feature `python`) → `plugins/PythonEmbeddedInterpreter.cc` via pyo3.
- `perl.rs` (feature `perl`) → `plugins/PerlEmbeddedInterpreter.cc` via raw FFI.
