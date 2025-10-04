# Port Learnings - DocDD in Practice

This document captures how we applied Doc-Driven Development to the okros C++ → Rust port.

## Overview

**Project**: Port ~11k LOC of C++ MUD client to Rust
**Timeline**: ~20 days (Discovery: 3d, Execution: 17d)
**Result**: 5,550 LOC Rust (-37% code reduction), 134 tests passing, validated on real MUD

## What We Actually Did

### Discovery Mode (12 toys, ~3 days)

Built isolated experiments to validate risky patterns before touching production:

- **toy1**: String handling (skipped - Rust stdlib wins)
- **toy2**: ncurses FFI patterns (raw bindings work)
- **toy3**: Global state (lazy_static + RefCell)
- **toy4**: Python embedding (pyo3 patterns)
- **toy5**: Perl embedding (raw FFI, PERL_SYS_INIT3)
- **toy6-11**: TTY, screen diff, telnet, MCCP, scrollback, interpreter stack
- **toy12**: Internal MUD (bonus e2e test harness)

Each toy: SPEC.md + PLAN.md + code + LEARNINGS.md

**Result**: Validated all risky patterns before touching production. No mid-port rewrites needed.

### Execution Mode (7 tiers, ~17 days)

Systematic tier-by-tier translation applying validated patterns:

- **Tier 1-7**: Foundation → Core → UI → Logic → Plugins → Main → Integration
- Applied toy learnings to production (src/)
- Updated CODE_MAP.md before structural commits
- Mandatory refactoring after each feature
- Golden tests against C++ reference

## Final Stats

| Metric | C++ | Rust | Change |
|--------|-----|------|--------|
| Lines of Code | 8,815 | 5,550 | -37% |
| Files | 79 | 33 | -58% |
| Tests | Manual | 134 passing | +134 |
| Validation | N/A | Real MUD gameplay | ✅ |

**Total time**: Discovery (3d) + Execution (17d) = ~20 days

## What Worked

### Toys Killed Uncertainty Early
- No mid-port architectural rewrites
- FFI patterns validated before production use
- LEARNINGS.md → production was direct path

### Mandatory Refactoring Kept Quality Rising
- Economic inversion is real (AI makes refactoring cheap)
- Code quality improved instead of decaying
- Each feature left codebase cleaner than it found it

### CODE_MAP.md Made Mapping Legible
- Clear C++ → Rust file correspondence
- Easy to find origin of each module
- Living documentation stayed synchronized

### "Simplicity First" Principle
- Use Rust idioms when simpler than C++ fidelity
- Don't force 1:1 mapping when stdlib is better
- Example: String.cc → String, not custom wrapper

## What We Learned

### Porting Mode = Discovery + Execution Hybrid
- Use Discovery for risky/uncertain patterns (FFI, unsafe, complex APIs)
- Use Execution for systematic translation (tier-by-tier)
- Switch modes fluidly based on uncertainty level

### Reference Implementation as Oracle
- Golden tests against C++ MCL prevent behavioral drift
- Side-by-side comparison essential during porting
- "Same inputs → same outputs" is measurable goal

### Scope Evolution is Normal
- MVP philosophy emerged during port
- Deferred chat/borg/group features to scripts
- Focus on transport layer, let scripts handle logic

### Toys are Intermediate Artifacts Worth Keeping
- toy12 became production test MUD (internal offline mode)
- Toys serve as reference for future similar work
- LEARNINGS.md captures portable patterns

## Artifacts You Can Inspect

**Discovery Phase**:
- `/toys/toy*/` - Experiments with SPEC/PLAN/LEARNINGS
- Each toy isolates 1-2 complexity axes
- Validation before production commitment

**Execution Phase**:
- `PORTING_HISTORY.md` - Tier-by-tier completion record
- `CODE_MAP.md` (root + src/ + src/plugins/) - Living architecture map
- `src/*.rs` - Production code with C++ origins documented

**Validation**:
- `MUD_LEARNINGS.md` - Real MUD gameplay validation results
- `FUTURE_WORK.md` - Remaining validation and post-MVP enhancements

## Key Decisions

### When to Preserve C++ Patterns vs Use Rust Idioms

**Use Rust idioms when**:
- Stdlib provides simpler/safer equivalent
- Target language has better abstraction
- Reduces complexity without losing behavior

**Preserve C++ patterns when**:
- Matches behavior more directly
- Reduces translation complexity
- FFI/unsafe boundaries require it

**Examples**:
- String.cc → `String` (Rust idiom wins)
- ncurses → raw FFI (no safe abstraction exists)
- Global state → lazy_static (Rust pattern for C++ globals)

### Toy Complexity Budget

**Axis Principle**:
- Base toy: 1 complexity axis (single invariant/mechanism)
- Integration toy: 2 axes (probe interaction)
- Never exceed 2 axes per toy

**Why this works**:
- Keeps learnings sharp
- Avoids doc bloat
- Mirrors controlled experiments

## Methodology Notes

This port demonstrates **Porting Mode** from DDD.md:
- Not pure Discovery (we had reference implementation)
- Not pure Execution (too much uncertainty for direct port)
- Hybrid: Validate patterns (Discovery) → Apply systematically (Execution)

**Key insight**: Porting is reference-driven translation, not greenfield development.

## Recommended Reading Order

1. `README.md` - Project overview
2. `PORT_LEARNINGS.md` (this file) - How DocDD was applied
3. `DDD.md` - Full methodology (project-agnostic)
4. `PORTING_HISTORY.md` - Detailed tier-by-tier record
5. `/toys/toy*/LEARNINGS.md` - Specific pattern validations

---

**This codebase doubles as DocDD reference implementation. Poke around.**
