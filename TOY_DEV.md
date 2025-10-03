# Toy Model Development for C++ → Rust Porting

_Toys help understand complex C++ before porting. They validate FFI/unsafe patterns and remain in the repo as reference artifacts—intermediate code that informs the final implementation._

---

## What Toy Models Are (In Porting Context)

- **C++ understanding tools**: Build minimal Rust equivalent to comprehend obscure C++ behavior
- **FFI/unsafe validators**: Test ncurses bindings, Python/Perl integration, unsafe patterns
- **Intermediate artifacts**: Code stays in repo as reference; lessons transfer to production
- **Risk reducers**: Validate complex subsystems before porting to main codebase
- **Pattern libraries**: Discover reusable FFI/unsafe idioms for the port

## What Toy Models Are Not

- Not production code (production porting happens in src/)
- Not comprehensive solutions (focus on one tricky subsystem)
- Not deleted after use (kept as reference, allowed dead code)
- Not shortcuts (experiments inform proper porting)  

---

## The Toy Model Cycle (Porting Context)

**CRITICAL**: Every toy starts AND ends with LEARNINGS.md

### 1. Define Learning Goals (LEARNINGS.md - First Pass)
Before any research, write down what you need to learn.
- Questions to answer (e.g., "Which ncurses binding?")
- Decisions to make (e.g., "Wrapper strategy?")
- Success criteria (what patterns must be clear)
- **Start with questions, not answers**

### 2. Research & Implementation Loop
Iterate until learning goals are met:
- Study C++ code for patterns
- Try approaches in Rust (unsafe/FFI as needed)
- Test against C++ reference behavior
- **Update LEARNINGS.md with findings after each cycle**

### 3. Finalize Learnings (LEARNINGS.md - Final Pass)
Extract portable patterns for production port.
- Answer all initial questions
- Document chosen approach and rationale
- FFI/unsafe patterns discovered
- How to port to main codebase

**Key insight**: LEARNINGS.md is both roadmap (goals) and artifact (findings)  

---

## Guiding Principles (Porting Focus)

- **Reference-driven validation**
  Test Rust toy against C++ reference behavior. Same inputs → same outputs.

- **"Safety third" experimentation**
  Use unsafe, raw pointers, FFI freely. This is learning phase, not production.

- **FFI pattern discovery**
  Document which FFI approaches work (ncurses, Python/Perl C APIs).

- **Minimal scope**
  One C++ subsystem per toy. Don't try to port entire system in toy.

- **Reusable idioms**
  Extract patterns that apply to similar C++ code elsewhere in port.  

---

## Patterns That Work (Porting Toys)

- **FFI validation toys**: Test ncurses bindings, Python/Perl C API integration
- **Unsafe pattern toys**: Experiment with raw pointers, transmute, manual memory management
- **Subsystem toys**: Understand one C++ class/module before porting (String, Buffer, etc.)
- **Integration toys**: Test how two ported subsystems interact

---

## Testing Philosophy (Porting Validation)

- **Golden tests**: C++ MCL is the oracle - same input must produce same output
- **Behavioral equivalence**: Rust toy must match C++ reference behavior exactly
- **FFI contract tests**: Ensure ncurses/Python/Perl bindings work as expected
- **Unsafe correctness**: No segfaults, memory leaks, or UB beyond what C++ has  

---

## Strategic Guidance (Porting Context)

- Pivot to simpler FFI approach when patterns become too complex
- Extract reusable unsafe/FFI patterns for main port
- Toys remain in repo as reference - intermediate artifacts that inform production
- Use toys to de-risk complex subsystems before main port

---

## North Star

Toys are **reconnaissance, not construction**.
Scout C++ terrain in Rust without production constraints.
Focus: understanding FFI/unsafe patterns.
Result: lessons applied to src/, toy kept as reference artifact.  