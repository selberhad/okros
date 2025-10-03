# Toy 11 — Plugins Stack (Optional): LEARNINGS

## Step 0 — Learning Goals
- Chaining semantics: Recreate `StackedInterpreter` run/run_quietly behavior passing transformed buffers between layers.
- Enable/disable: Mirror `EmbeddedInterpreter::{enable,disable,isEnabled}` filtering for function names.
- API parity: Validate set/get for ints/strings and `load_file`/`eval` across layers; define error suppression policy.
- Ordering: Confirm deterministic order when multiple interpreters are present (Perl, then Python, etc.).
- Minimal trait: Identify the smallest Rust trait surface needed to keep API parity during the port.

## Findings (Tests & Parity)

- Chaining: `run` composes outputs from each interpreter in order; `run_quietly` can be overridden while `run` returns false.
- Enable/disable: Function-name filtering prevents execution; re-enabling restores chaining output.
- Set/get passthrough: `set_str` broadcasts to all; `get_str` returns from the first interpreter, matching a “primary” semantics.

- Ordering: Insertion order defines execution order (A → B → C). This matches the reference’s stacked plugin behavior.
- Error suppression: `run_quietly` receives a `suppress_error` flag per layer; a layer can choose to succeed only when suppression is enabled without affecting others.
- Defaults: `load_file`/`eval` are no-ops in the toy trait by default; ported implementations (Perl/Python) will override them.

## Open Questions

- Error suppression policy: What constitutes “quiet” errors across Perl/Python layers, and how do we aggregate/short-circuit failures?
- Layer ordering: For features `python, perl`, what is the canonical order for parity with Embedded.cc (Perl first vs Python first)?
- API surface: Do we need `load_file` or dynamic module search paths identical to the reference, or can we unify around Rust-side config?
