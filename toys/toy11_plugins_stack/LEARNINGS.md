# Toy 11 — Plugins Stack (Optional): LEARNINGS

## Step 0 — Learning Goals
- Chaining semantics: Recreate `StackedInterpreter` run/run_quietly behavior passing transformed buffers between layers.
- Enable/disable: Mirror `EmbeddedInterpreter::{enable,disable,isEnabled}` filtering for function names.
- API parity: Validate set/get for ints/strings and `load_file`/`eval` across layers; define error suppression policy.
- Ordering: Confirm deterministic order when multiple interpreters are present (Perl, then Python, etc.).
- Minimal trait: Identify the smallest Rust trait surface needed to keep API parity during the port.

