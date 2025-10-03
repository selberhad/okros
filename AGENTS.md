# Repository Guidelines

Condensed 1:1 summary of `CLAUDE.md`. For details, see CLAUDE.md.

## Project Overview
1:1 port of MCL (MUD Client for Linux) from C++ to Rust with "safety third" approach - liberal unsafe/FFI for maximum fidelity to reference implementation.

## Tech Stack & Architecture
- **Language**: Rust (unsafe permitted), **Reference**: `mcl-cpp-reference/` (~11k LOC)
- **Dependencies**: ncurses/pancurses, optional pyo3 (Python), Perl FFI
- **Features**: `--features python,perl` for scripting support

### Build & Test Quickstart
```bash
cargo build                    # Base build
cargo build --features python  # With Python
cargo test --all-features      # All tests
```

### Development Best Practices
- ✅ Use Rust idioms (String, Vec) when simpler than C++
- ✅ Match C++ structure when it reduces complexity
- ✅ Unsafe/FFI only where needed (ncurses, interpreters, complex C++)
- ✅ Compare with C++ reference constantly
- ✅ Perl for dev scripts (unless shell/Python has clear advantage)

## Operational Modes
**Discovery**: Complex C++ subsystems - build toys, extract patterns
**Execution**: Direct C++ → Rust translation following PLAN.md tiers

## Core Methodology
**Document Types**: PLAN.md (strategy), SPEC.md (toys), LEARNINGS.md (toys), CODE_MAP.md (tracking)
**Principles**: Rust idioms when simpler, C++ structure when useful, tier-by-tier (Foundation → App)

## Discovery Mode
- When: C++ obscure, FFI unclear, integration complex
- Output: Toys in `toys/` as reference artifacts
- Cycle: LEARNINGS.md (goals) → research/impl loop → LEARNINGS.md (findings) → port to src/
- **Key**: Start with questions, iterate to answers

## Execution Mode
- When: Straightforward C++ → Rust translation
- Output: Ported modules in `src/`
- Discipline: Side-by-side comparison, preserve structure

## Documentation Structure
**CODE_MAP.md**: One per directory with .rs files - update BEFORE structural commits
**Porting Workflow**: Study C++ → Port → Test → Update CODE_MAP → Commit
**Toys**: `toys/toyN_name/` with SPEC, PLAN, LEARNINGS, kept as reference

## Commit Guidelines
- Format: `type(scope): subject` (feat, fix, docs, chore, refactor, test)
- Port cadence: One commit per PLAN.md step
- Example: `feat(foundation): port String class - Step 6`
- Footer: `Ported-From: mcl-cpp-reference/String.cc` (optional)

## Next Step Protocol
Propose specific next action, wait for approval.
Example: "Next step: Port String.cc to src/string.rs following Step 6 of PLAN.md"

## Key Files
- **PLAN.md**: Master strategy (Discovery + Execution phases)
- **TOY_PLAN.md**: Discovery phase (5 toys)
- **IMPLEMENTATION_PLAN.md**: Execution phase (tier-by-tier)
- **DDD.md**: Methodology (project-agnostic)
- **PLAYBOOK.md**: Condensed workflow
- **mcl-cpp-reference/**: C++ source (~29 .cc, ~50 .h)
