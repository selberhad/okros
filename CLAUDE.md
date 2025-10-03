# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the MCL Rust Port project - a 1:1 port of MCL (MUD Client for Linux) from C++ to Rust using a "safety third" approach. The goal is maximum fidelity to the reference C++ implementation (~11k LOC), with liberal use of unsafe/FFI to match C++ behavior exactly.

## Tech Stack & Architecture

- **Language**: Rust (unsafe permitted and encouraged)
- **Reference**: C++ codebase in `mcl-cpp-reference/` (~29 .cc files, ~50 headers)
- **Approach**: "Safety third" - 1:1 mapping (class→struct, method→impl, global→static)
- **Architecture**: Single-threaded (no concurrency in C++ code, multiple sessions via tmux/screen)
- **Dependencies**: ncurses (or pancurses), optional pyo3 (Python), Perl FFI (Perl)
- **Features**: Optional cargo features for Python (`--features python`) and Perl (`--features perl`) scripting

### Build & Test Quickstart
- Build: `cargo build` (or `cargo build --release` for optimized)
- Build with features: `cargo build --features python,perl`
- Run tests: `cargo test` (or `cargo test --all-features`)
- Run MCL: `cargo run` (or `cargo run --features python,perl`)
- Check only: `cargo check` (fast, no codegen)

### Development Best Practices

**CRITICAL: Simplicity-first porting approach**
- ✅ CORRECT: Use Rust idioms (String, Vec) when simpler than C++ patterns
- ✅ CORRECT: Match C++ structure when it reduces complexity
- ✅ CORRECT: Unsafe/FFI where needed (ncurses, interpreters, truly complex C++)
- ❌ WRONG: Forcing C++ patterns when Rust stdlib is simpler
- Why: Goal is working port with least complexity, not fidelity for its own sake
- When: All porting work - choose simplest approach per module

**CRITICAL: Compare with C++ reference constantly**
- Keep C++ source files open side-by-side when porting
- Test Rust output against C++ MCL behavior (golden tests)
- Document any deviations with `// NOTE: differs from C++` comments
- Preserve C++ quirks/bugs with `// FIXME: C++ compat` comments
- Why: Ensures behavioral equivalence with reference implementation

**CRITICAL: Perl for development scripting**
- Use Perl for dev-adjacent scripts (build helpers, text processing, code generation, automation)
- Shell: Use only for trivial one-liners or when Perl is overkill
- Python: Use only when clear advantage (e.g., specific library requirement)
- Why: Consistency with MCL codebase context (Perl embedding), powerful text processing

## Operational Modes

The porting workflow operates in two distinct modes:

### Discovery Mode
- **When to use**: Complex C++ subsystems that need understanding before porting
- **Cycle**: SPEC (C++ behavior) → PLAN → TOY RUST → LEARNINGS → Port to src/
- **Focus**: FFI/unsafe pattern validation, ncurses/interpreter integration testing
- **Output**: Toy implementations in `toys/` - kept as reference artifacts

### Execution Mode (Primary)
- **When to use**: Straightforward C++ → Rust translation
- **Cycle**: Compare with C++ → Port → Test against reference → Refactor if needed
- **Focus**: 1:1 fidelity, preserve structure, match behavior exactly
- **Output**: Ported Rust modules in `src/` matching C++ reference

## Core Methodology

This project follows **Doc-Driven Development (DDD)** adapted for C++ → Rust porting with "safety third" approach.

### Document Types & Usage (Porting Context)
- **TOY_PLAN.md**: Discovery phase strategy (toys 1-5, current progress)
- **IMPLEMENTATION_PLAN.md**: Execution phase strategy (tier-by-tier port)
- **SPEC.md**: C++ behavior to replicate (for Discovery mode toys)
- **LEARNINGS.md**: FFI/unsafe patterns discovered (for Discovery mode toys)
- **CODE_MAP.md**: Living map of ported modules (updated before structural commits)
- **README.md**: Project overview and porting approach

### Universal Principles (Porting)
- **1:1 fidelity first**: Match C++ structure/behavior exactly, defer idiomatic Rust to second pass
- **"Safety third"**: Liberal use of unsafe, raw pointers, FFI to replicate C++ patterns
- **Tier-by-tier**: Port in dependency order (Foundation → Core → UI → Logic → App)
- **Reference-driven**: Always compare with C++ source, test against C++ MCL behavior
- **Document deviations**: Mark any differences from C++ with comments

## Discovery Mode Methodology

**Use this methodology for complex C++ subsystems** (build toy to understand before porting):

### When to Use Discovery Mode
- C++ code is obscure/complex and needs experimentation
- FFI patterns unclear (ncurses, Python/Perl C APIs)
- Unsafe Rust patterns need validation
- Novel integration challenges (embedded interpreters)

### Toy Porting Cycle (Learning-First)
1. **LEARNINGS.md (goals)**: Define questions/decisions before any research
   - What do we need to learn?
   - What decisions must be made?
2. **Research/implement loop**: Iterate to answer questions
   - Study C++ patterns
   - Try approaches in Rust
   - Update LEARNINGS.md with findings
3. **LEARNINGS.md (final)**: Extract portable patterns for production
4. **Port to src/**: Apply validated patterns to main codebase

**Key**: Start with questions, end with answers. LEARNINGS.md is both roadmap and artifact.

### Discovery Mode Principles
- **One subsystem per toy**: Focus experiments narrowly
- **Compare with C++**: Test Rust toy against C++ reference
- **Extract patterns**: Document reusable FFI/unsafe idioms
- **Retain as reference**: Toys live in repo as intermediate artifacts

## Execution Mode Methodology

**Use this methodology for direct C++ → Rust translation** (primary porting mode):

### Execution Mode Principles (Porting)
- Port following PLAN.md phases (Foundation → Core → UI → Logic → App)
- Keep C++ reference open side-by-side during porting
- Match structure: one .rs file per .cc file when feasible
- Preserve C++ behavior exactly (same inputs → same outputs)
- Use unsafe/FFI freely to replicate C++ patterns
- Test against C++ MCL reference (golden tests)

## Documentation Structure

### CODE_MAP.md Convention
**CRITICAL: Keep CODE_MAP.md files up-to-date on every structural commit**

- **Scope**: One CODE_MAP.md per directory containing significant files/structure
- **Content**: Each CODE_MAP.md describes only files/folders in its own directory, not subdirectories
- **Location**:
  - `./CODE_MAP.md` - Root directory (documentation, toys, future src/)
  - `src/CODE_MAP.md` - Ported Rust modules (create when porting begins)
  - `tests/CODE_MAP.md` - Test organization (create when tests added)
- **Purpose**: Track project structure, porting status, and C++ origins
- **Update trigger**: Before any commit that changes structure or adds/removes/renames files
- **Porting notes**: Include which C++ file each .rs file was ported from

**Current status**: Root CODE_MAP.md exists, tracks toys and overall structure

### Porting Workflow (Execution Mode - Primary)
1. **Choose next module**: Follow IMPLEMENTATION_PLAN.md tier order (Foundation → Core → UI → Logic → App)
2. **Study C++ source**: Read C++ .cc/.h files thoroughly, understand behavior
3. **Port to Rust**: Create .rs file, use unsafe/FFI freely, preserve structure
4. **Test**: Compare Rust output with C++ MCL (golden tests)
5. **Update CODE_MAP.md**: Document ported module before committing
6. **Commit**: One commit per IMPLEMENTATION_PLAN.md step (e.g., `feat(foundation): port String - Step 6`)

**Policy**: Port one module at a time following tier order. Update CODE_MAP.md before structural commits. Preserve C++ behavior exactly.

### Toy Development Workflow (Discovery Mode - As Needed)
1. **Identify complex subsystem**: C++ code needs understanding before porting
2. **Create toy directory**: `toys/toyN_subsystem_name/`
3. **Write SPEC.md**: Define C++ behavior to replicate
4. **Write PLAN.md**: How to build Rust equivalent
5. **Implement**: Build minimal Rust version, test against C++
6. **Extract LEARNINGS.md**: Document FFI/unsafe patterns discovered
7. **Port to src/**: Apply lessons to main codebase; toy remains as reference

**Policy**: Use toys sparingly for complex subsystems (ncurses, interpreters). Most porting is direct.

### Commit Guidelines
**Use conventional commit format for all commits:**
- **Format**: `type(scope): subject` with optional body/footer
- **Types**: `feat` (port), `fix`, `docs`, `chore`, `refactor`, `test`
- **Port discipline**: Commit after logical milestones (module complete, tests pass, etc.)
- **Descriptive commits**: Include C++ origin (e.g., "feat(foundation): port String class from String.cc")
- **History**: Keep linear history (prefer rebase; avoid merge commits)
- **Footer**: Note C++ source file if helpful (e.g., `Ported-From: mcl-cpp-reference/String.cc`)
- **Documentation updates**: Update affected CODE_MAP.md files BEFORE committing structural changes

### Pull Request Guidelines
- Summarize C++ → Rust mapping (which modules ported, approach used)
- List ported files and their C++ origins
- Note any deviations from C++ reference
- Include test status (cargo test results, manual testing notes)

### Next Step Protocol
**Never just say "ready for next step" - always propose the specific next action:**
- Identify the next module to port from IMPLEMENTATION_PLAN.md
- Propose which C++ file to port next
- Wait for explicit approval before proceeding
- Example: "Next step: Port String.cc to src/string.rs following Tier 1 (Foundation) of IMPLEMENTATION_PLAN.md"

## Key Files Reference

**Current Status**: Discovery phase COMPLETE (5/5 toys validated) — Execution phase scaffolded (Cargo.toml, src/ initialized)

**File Structure**: See `CODE_MAP.md` for complete project structure and status
- `./CODE_MAP.md` - Root directory structure, toys status, future src/ layout
- `src/CODE_MAP.md` - Ported Rust modules (create when porting begins)
- `tests/CODE_MAP.md` - Test organization (create when tests added)

**Reference Code**:
- `mcl-cpp-reference/` - Original C++ codebase (~11k LOC, 29 .cc files, ~50 headers)
- `mcl-cpp-reference/*.cc` - C++ implementation files to port
- `mcl-cpp-reference/h/*.h` - C++ headers for reference

**Core Documentation**:
- `CODE_MAP.md` - Project structure and status (START HERE for orientation)
- `TOY_PLAN.md` - Discovery phase strategy and progress (toys 1-5)
- `IMPLEMENTATION_PLAN.md` - Execution phase strategy (tier-by-tier port)
- `DDD.md` - Methodology adapted for Rust porting
- `PLAYBOOK.md` - Condensed porting workflow guide
- `AGENTS.md` - Quick reference summary
