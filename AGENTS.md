# Repository Guidelines — Quick Reference

Condensed summary of `CLAUDE.md`. For details, see CLAUDE.md or ORIENTATION.md.

## Project Overview
**okros** - Rust port of MCL (MUD Client for Linux), ~97% feature-complete. Transport layer for Perl/Python bots and LLM agents. **1:1 porting approach**: Port complete C++ execution path including virtual dispatch patterns. **Every shortcut is a bug** (see DISPLAY_BUG_POSTMORTEM.md).

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
**⚠️ CRITICAL**: Read DISPLAY_BUG_POSTMORTEM.md - lessons on shortcuts vs 1:1 ports

**1:1 Port - No Shortcuts**:
- ✅ Port C++ BEHAVIOR completely (including virtual dispatch, call paths)
- ✅ Use Rust idioms (String, Vec) ONLY when 1:1 replacements
- ✅ Unsafe/FFI freely (ncurses, interpreters, raw pointers)
- ❌ "Composition ≈ inheritance" - NO! Must hook virtual dispatch manually
- ❌ "Data flows right" - NO! Must port CALL PATH too

**C++ Inheritance → Rust Composition**:
- C++: `class Derived : Base` with virtual methods → vtable dispatch
- Rust: `struct Derived { base: Box<Base> }` → NO automatic dispatch
- **Fix**: Manually call derived methods before tree refresh (see commits 08bcac2, 253c332)
- **Pattern**: `if obj.win.dirty { obj.redraw(); obj.win.dirty = true; } screen.refresh();`

## Operational Modes
**Discovery**: Complex C++ subsystems - build toys, extract patterns (12/12 complete, including internal MUD)
**Execution**: Direct C++ → Rust translation following tier-by-tier approach (~95% done, validation pending)

## Core Methodology (DDD Porting Mode)
**Key Documents**:
- ORIENTATION.md (start here), PORTING_HISTORY.md (porting history), FUTURE_WORK.md (remaining tasks)
- SPEC.md (toys), LEARNINGS.md (toys), CODE_MAP.md (tracking)

**Principles**:
- **1:1 Port** - No shortcuts (if C++ does X, Rust must do X semantically)
- **Virtual Dispatch** - Manually hook composition (C++ vtable → Rust explicit calls)
- **Execution Path Fidelity** - Port CALL PATH not just data flow
- MVP philosophy (client = transport, scripts = logic)

## Discovery Mode (12/12 toys complete) ✅
- When: C++ obscure, FFI unclear, integration complex
- Output: Toys in `toys/` (ncurses, Python/Perl FFI, telnet/MCCP, scrollback, plugins, internal MUD)
- Status: All risky patterns validated + built-in test infrastructure
- **Key**: Start with questions, iterate to answers, extract portable patterns

## Execution Mode (~97% complete) ✅
- When: Direct C++ → Rust translation
- **Key**: Trace C++ execution path completely, port every step (no shortcuts!)
- **Watch For**: C++ inheritance → must manually hook Rust composition
- Output: Ported modules in `src/` (all tiers complete: network, UI, plugins, event loop, headless)
- Status: Feature-complete TTY + headless modes

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
- **DISPLAY_BUG_POSTMORTEM.md**: ⚠️ **REQUIRED** - Lessons on 1:1 porting (shortcuts = bugs)
- **PORTING_HISTORY.md**: Historical record of C++ → Rust porting (tier-by-tier completion)
- **FUTURE_WORK.md**: Remaining tasks, post-MVP enhancements, deferred features
- **README.md**: User-facing overview
- **TOY_PLAN.md / TOY_PLAN_2.md**: Discovery phase (12 toys complete, including internal MUD)
- **DDD.md**: Methodology (includes Porting Mode)
- **CODE_MAP.md**: Project structure and C++ origins
- **mcl-cpp-reference/**: C++ source (~29 .cc, ~50 .h)
