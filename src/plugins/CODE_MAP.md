# CODE MAP — src/plugins/

- `stack.rs` → Stacked interpreter utility (consolidates behavior from `EmbeddedInterpreter` layering in C++; Toy 11).
- `python.rs` (feature `python`) → `plugins/PythonEmbeddedInterpreter.cc` via pyo3 (Toy 4).
- `perl.rs` (feature `perl`) → `plugins/PerlEmbeddedInterpreter.cc` via raw FFI (Toy 5).
