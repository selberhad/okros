# PLAYBOOK — Doc-Driven Development for C++ to Rust Porting

---

## 1. Purpose

Doc-Driven Development (DocDD) turns ambiguous problems into deterministic, legible systems through lightweight docs, time-boxed toy models, and incremental integrations.

**This playbook is adapted for porting MCL from C++ to Rust with a "safety third" approach** - prioritizing 1:1 fidelity over idiomatic Rust patterns in the first pass.

See: [[DDD]].

---

## 2. Core Principles

- **Docs as control surfaces** — SPEC, PLAN, LEARNINGS, README
- **1:1 fidelity first** — match C++ structure/behavior exactly (class→struct, method→impl, global→static)
- **Safety third** — liberal use of unsafe, raw pointers, FFI; defer idiomatic Rust to second pass
- **Preserve bugs** — match reference behavior including quirks (document with `// FIXME: C++ compat`)
- **Toys for complexity** — build toy implementations to understand tricky C++ before porting
- **Tier-by-tier** — port in dependency order (Foundation → Core → UI → Logic → App)

---

## 3. The DocDD Loop (For Porting Context)

1. Study C++ — understand reference behavior thoroughly
2. SPEC (if toy) — define C++ behavior to replicate
3. PLAN — outline porting steps or toy approach
4. Implementation — port with unsafe/FFI, preserve structure
5. Test — validate against C++ reference
6. LEARNINGS (if toy) — extract FFI/unsafe patterns

---

## 4. Toy Models (For Complex Subsystems)

Small Rust implementations built to understand tricky C++ before porting. Use when:
- C++ code is complex/obscure and needs experimentation
- FFI/unsafe patterns are unclear
- ncurses/interpreter integration needs validation

**Cycle (Learning-First)**:
1. LEARNINGS.md (goals) → define questions to answer
2. Research/implement loop → answer questions, update findings
3. LEARNINGS.md (final) → extract patterns for production port

Toys remain as reference implementations - intermediate artifacts that inform the port.

See: [[TOY_DEV]].

---

## 5. Testing & Validation

- **Unit tests**: Test ported modules against known inputs
- **Integration tests**: Validate ncurses, socket handling, interpreters
- **Manual testing**: Compare against reference C++ MCL
- **Golden tests**: Use C++ as oracle (same input → same output)

Match C++ error behavior exactly (messages, exit codes).

---

## 6. FFI & Unsafe Patterns

Embrace unsafe Rust for 1:1 C++ mapping:

- **Raw pointers**: `*const T`, `*mut T` to match C++ pointers
- **FFI**: ncurses, Python/Perl C APIs
- **Transmute**: For C-style type punning
- **Document**: `// SAFETY:` comments explain C++ correspondence

---

## 7. Porting Guardrails

- Keep C++ reference open, check line-by-line
- Same module boundaries, function signatures (adapted for Rust)
- Use `static`/`lazy_static` for C++ globals
- Port first, optimize later (if ever)
- Mark deviations: `// NOTE: differs from C++`

---

## 8. Roles

- **Agent** — generates docs, toys, integrations; pushes forward.  
- **Human** — spotter: nudges when the agent stalls or drifts, and makes judgment calls the agent cannot.  
