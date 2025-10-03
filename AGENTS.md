# Repository Guidelines — Quick Reference

Condensed summary of `CLAUDE.md`. For details, see CLAUDE.md or ORIENTATION.md.

## Project Overview
**okros** - Rust port of MCL (MUD Client for Linux), ~95% complete (validation pending). Transport layer for Perl/Python bots and LLM agents. Simplicity first: use Rust idioms when simpler, preserve C++ patterns when it reduces complexity.

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
**Discovery**: Complex C++ subsystems - build toys, extract patterns (12/12 complete, including internal MUD)
**Execution**: Direct C++ → Rust translation following tier-by-tier approach (~95% done, validation pending)

## Core Methodology (DDD Porting Mode)
**Key Documents**:
- ORIENTATION.md (start here), PORTING_HISTORY.md (porting history), FUTURE_WORK.md (remaining tasks)
- SPEC.md (toys), LEARNINGS.md (toys), CODE_MAP.md (tracking)

**Principles**:
- Simplicity first (Rust idioms when simpler, C++ patterns when useful)
- Behavioral equivalence (not structural)
- MVP philosophy (client = transport, scripts = logic)

## Discovery Mode (12/12 toys complete) ✅
- When: C++ obscure, FFI unclear, integration complex
- Output: Toys in `toys/` (ncurses, Python/Perl FFI, telnet/MCCP, scrollback, plugins, internal MUD)
- Status: All risky patterns validated + built-in test infrastructure
- **Key**: Start with questions, iterate to answers, extract portable patterns

## Execution Mode (~95% complete) ✅
- When: Straightforward C++ → Rust translation
- Output: Ported modules in `src/` (all tiers complete: network, UI, plugins, event loop, headless)
- Next: Real MUD validation, Perl bot integration testing

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
- **ORIENTATION.md**: Executive summary (START HERE)
- **PORTING_HISTORY.md**: Historical record of C++ → Rust porting (tier-by-tier completion)
- **FUTURE_WORK.md**: Remaining tasks, post-MVP enhancements, deferred features
- **README.md**: User-facing overview
- **TOY_PLAN.md / TOY_PLAN_2.md**: Discovery phase (12 toys complete, including internal MUD)
- **DDD.md**: Methodology (includes Porting Mode)
- **CODE_MAP.md**: Project structure and C++ origins
- **mcl-cpp-reference/**: C++ source (~29 .cc, ~50 .h)
