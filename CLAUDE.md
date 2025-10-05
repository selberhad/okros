# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **okros** - a Rust port of MCL (MUD Client for Linux) reviving its design for modern use cases. The project translates ~11k LOC of C++ to Rust with a **1:1 porting approach**: port the complete C++ execution path to Rust, including virtual dispatch patterns.

**Philosophy**: okros is a transport layer. Perl/Python scripts handle command logic (aliases, triggers, automation).

## ⚠️ CRITICAL LESSON: No Shortcuts in 1:1 Ports

**Read `DISPLAY_BUG_POSTMORTEM.md` before making ANY code changes.**

**The Iron Rule**: When your goal is a 1:1 port, every shortcut is a bug waiting to happen.

**What happened**: We took a "simplification" shortcut (composition without hooking virtual dispatch), which caused 3 display bugs that took hours to debug. The correct 1:1 port was 5 lines of code that should have been written from the start.

**The Lesson**:
- ✅ **DO**: Port C++ behavior COMPLETELY (including virtual dispatch, call paths, execution order)
- ❌ **DON'T**: "Simplify" or take shortcuts ("composition is close enough to inheritance")
- ❌ **DON'T**: Assume "the data flows right" is sufficient (must port the CALL PATH too)

**Key Pattern**: C++ `class Derived : public Base` with virtual methods → Rust requires manual dispatch hooks (see "C++ Inheritance → Rust Composition Patterns" section below).

---

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

**CRITICAL: 1:1 Port - No Shortcuts**
- **THE IRON RULE**: If C++ does X, Rust must do X (semantically). "Close enough" is a bug.
- ✅ CORRECT: Port the C++ BEHAVIOR completely, including virtual dispatch patterns
- ✅ CORRECT: Use Rust idioms (String, Vec) when they're 1:1 replacements (char*/std::string → String)
- ✅ CORRECT: Unsafe/FFI where needed (ncurses, interpreters, raw pointers)
- ❌ WRONG: "Composition is close enough to inheritance" - NO! Must hook virtual dispatch manually
- ❌ WRONG: "I'll skip this step, the data flows right" - NO! Port the ENTIRE execution path
- ❌ WRONG: "Simplifying" C++ patterns without understanding their purpose
- **Why**: Every shortcut is a bug waiting to happen (see `DISPLAY_BUG_POSTMORTEM.md`)
- **When**: ALL porting work - if unsure, trace C++ execution step-by-step and port each step

**CRITICAL: C++ Inheritance → Rust Composition Patterns**
- **C++ Pattern**: `class Derived : public Base` with virtual methods
- **Rust Pattern**: `struct Derived { base: Box<Base> }` - composition, NOT inheritance
- **THE CATCH**: Virtual dispatch doesn't happen automatically in Rust!
- **Required Fix**: Manually hook derived class methods into refresh/event cycle
- **Example** (see commits 08bcac2, 253c332):
  ```rust
  // C++: OutputWindow::redraw() called via vtable when output->refresh() runs
  // Rust: Must manually call BEFORE tree refresh:
  if output.win.dirty {
      output.redraw();        // Manual "virtual dispatch"
      output.win.dirty = true; // Keep dirty for tree
  }
  screen.refresh(&caps);      // Tree refresh continues
  ```
- **Checklist for every C++ class with virtual methods**:
  - [ ] Identify all virtual methods in C++ (redraw(), keypress(), execute(), etc.)
  - [ ] Find where C++ calls them via base class pointer (vtable dispatch)
  - [ ] Add explicit calls in Rust at those same points
  - [ ] Test that the method actually fires (add debug logging if unsure)
- **Red Flags**: "Why isn't X showing up?" "Data is written but not displayed" → Check virtual dispatch!
- **See**: `DISPLAY_BUG_POSTMORTEM.md` for detailed case study (3 bugs, all same root cause)

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

This project follows **Doc-Driven Development (DDD)** in **Porting Mode** - a reference-driven translation workflow combining Discovery (validate risky patterns) + Execution (systematic translation). See `DDD.md` for full methodology.

### Document Types & Usage (Porting Context)
- **ORIENTATION.md**: Executive summary (START HERE - what is this, where are we, what's next)
- **PORTING_HISTORY.md**: Historical record of tier-by-tier porting completion
- **FUTURE_WORK.md**: Remaining tasks and post-MVP enhancements
- **TOY_PLAN.md / TOY_PLAN_2.md**: Discovery phase strategy (12 toys completed, including testing infrastructure)
- **SPEC.md**: C++ behavior to replicate (for Discovery mode toys)
- **LEARNINGS.md**: FFI/unsafe patterns discovered (for Discovery mode toys)
- **CODE_MAP.md**: Living map of ported modules (updated before structural commits)
- **README.md**: User-facing project overview

### Universal Principles (Porting)
- **1:1 Port - No Shortcuts**: If C++ does X, Rust must do X (semantically). Every shortcut is a bug.
- **Inheritance → Composition**: Manually hook virtual dispatch (redraw(), keypress(), etc.) - see above
- **"Safety third"**: Liberal use of unsafe, raw pointers, FFI to replicate C++ patterns
- **Tier-by-tier**: Port in dependency order (Foundation → Core → UI → Logic → App)
- **Reference-driven**: Always compare with C++ source, test against C++ MCL behavior
- **Execution path fidelity**: Port the CALL PATH, not just the data flow (C++ vtable → Rust explicit calls)
- **Behavioral equivalence**: Same inputs → same outputs (same execution path → same results)
- **Document deviations**: Mark any differences from C++ with comments
- **Learn from failures**: See `DISPLAY_BUG_POSTMORTEM.md` for case study on composition shortcuts

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
- **1:1 Port**: Trace C++ execution path completely, port every step (no shortcuts!)
- **Virtual Dispatch**: When C++ uses inheritance, manually hook Rust composition (see above)
- **Execution Path**: Port the CALL PATH, not just data flow (vtable → explicit calls)
- Port following PLAN.md phases (Foundation → Core → UI → Logic → App)
- Keep C++ reference open side-by-side during porting
- Match structure: one .rs file per .cc file when feasible
- Preserve C++ behavior exactly (same inputs → same outputs, same call path → same results)
- Use unsafe/FFI freely to replicate C++ patterns
- Test against C++ MCL reference (golden tests)
- **When in doubt**: Add debug logging, verify methods fire, compare with C++ step-by-step

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

### Porting Workflow (Execution Mode)
1. **Choose next module**: Follow PORTING_HISTORY.md tier order (Foundation → Core → UI → Logic → App)
2. **Study C++ source**: Read C++ .cc/.h files thoroughly, understand behavior
3. **Port to Rust**: Create .rs file, use unsafe/FFI freely, preserve structure
4. **Test**: Compare Rust output with C++ MCL (golden tests)
5. **Update CODE_MAP.md**: Document ported module before committing
6. **Commit**: Logical commits matching porting steps (e.g., `feat(foundation): port String - Step 6`)

See ORIENTATION.md for current porting status.

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

**CRITICAL: Pre-commit hooks must always run - NEVER use `git commit --no-verify`**
- Pre-commit hooks format code, generate coverage reports, and update LOC comparisons
- Using `--no-verify` bypasses quality checks and can lead to inconsistent repository state
- If pre-commit hook fails, **fix the underlying issue** instead of bypassing it
- Only exception: Discussed explicitly with user and documented in commit message
- Why: Ensures code quality, consistent formatting, and up-to-date documentation

### Pull Request Guidelines
- Summarize C++ → Rust mapping (which modules ported, approach used)
- List ported files and their C++ origins
- Note any deviations from C++ reference
- Include test status (cargo test results, manual testing notes)

### Next Step Protocol
**Never just say "ready for next step" - always propose the specific next action:**
- Identify the next task from FUTURE_WORK.md
- Propose specific action to take
- Wait for explicit approval before proceeding
- Example: "Next step: Manual testing of --offline mode with real TTY (see FUTURE_WORK.md section 2)"

## Key Files Reference

**File Structure**: See `CODE_MAP.md` for complete project structure
- See `ORIENTATION.md` for current project status
- `./CODE_MAP.md` - Root directory structure, toys status, src/ layout
- `src/CODE_MAP.md` - Ported Rust modules with C++ origins
- `src/plugins/CODE_MAP.md` - Plugin system (Python, Perl)

**Reference Code**:
- `mcl-cpp-reference/` - Original C++ codebase (~11k LOC, 29 .cc files, ~50 headers)
- `mcl-cpp-reference/*.cc` - C++ implementation files to port
- `mcl-cpp-reference/h/*.h` - C++ headers for reference

**Core Documentation**:
- `ORIENTATION.md` - Executive summary (START HERE for quick overview)
- `DISPLAY_BUG_POSTMORTEM.md` - **REQUIRED READING** - Lessons on 1:1 porting (shortcuts = bugs)
- `PORTING_HISTORY.md` - Historical record of C++ → Rust porting (tier-by-tier completion)
- `FUTURE_WORK.md` - Remaining tasks, post-MVP enhancements, deferred features
- `README.md` - User-facing overview (okros MUD client)
- `DDD.md` - Doc-Driven Development methodology (includes Porting Mode)
- `CODE_MAP.md` - Project structure and porting status
- `TOY_PLAN.md` / `TOY_PLAN_2.md` - Discovery phase (12 toys completed, including internal MUD)
- `PLAYBOOK.md` - Condensed porting workflow guide
- `AGENTS.md` - Quick reference summary
