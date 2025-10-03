# Toy 11 — Plugins Stack (SPEC, Optional)

## Objective
Mirror `Embedded.cc`’s stacked interpreter behavior in Rust: chain multiple interpreters (e.g., Perl, Python) so `run`/`run_quietly` pass transformed buffers through in order, with enable/disable controls.

## Scope
- Trait surface: `run`, `run_quietly`, `load_file`, `eval`, `set`(int/str), `get_int`, `get_string`, `match_prepare`, `substitute_prepare`, `match`.
- Stacking: Maintain ordered list; for `run(_quietly)`, feed output of interpreter i to i+1 when previous returned true.
- Enable/disable: Track disabled function names; short-circuit calls when disabled.
- Version/info: Optional per-plugin version string display.

## Inputs
- Interpreters: two mock interpreter impls with simple transforms (e.g., append markers).
- Calls: `run("sys/test", "in", out)`, `set("var", 42)`, `load_file("file", suppress)`.

## Outputs
- Chained result: `out` reflects last interpreter’s transformation when any returned true; boolean result reflects if any ran.
- State: Variables set propagate to all interpreters.

## Behavior
- run: For each interpreter e in order: if `e.run(f,arg,tmp)` returned true, set `arg=tmp` and set `res=true`.
- run_quietly: Same as run but honor `suppress_error` flag; treat failures as no-ops unless all fail noisily (configurable).
- set/get: `set` broadcasts; `get_*` read from first interpreter (parity with reference).
- load_file/eval: Map directly to first interpreter (parity).

## Success Criteria
- Deterministic chaining order; toggling one interpreter changes output accordingly.
- Disabled functions are skipped.
- set/get parity matches spec.

## Test Plan
- Unit: Two fake interpreters A,B with predictable transforms; verify chained output and boolean result.
- Disable: Disable a function and assert it is skipped; re-enable and confirm chaining restored.
- get/set: Ensure set broadcasts and get reads from first.

## Non-Goals
- Actual Perl/Python FFI; use mocks to validate stacking semantics only.

