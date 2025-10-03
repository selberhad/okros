# MCL Rust Port - Discovery Phase (Toy Plan)

## Objective

Build minimal toys to validate **genuinely complex** C++ → Rust patterns. Only build toys where Rust approach is unclear or C++ pattern is non-trivial. Skip toys for things that are obviously simpler in Rust.

**Principle**: Use Rust idioms when they simplify the port. Only force C++ fidelity where it reduces complexity.

**Architecture note**: MCL is single-threaded (no threading in C++ code). Multiple sessions handled by tmux/screen, not internal concurrency. This simplifies global state and FFI patterns significantly.

## Scope Analysis

**Source**: `mcl-cpp-reference/` (~11k LOC C++ to port)

**Complex subsystems that might need toys:**
- ncurses FFI integration (terminal UI) - **likely needed**
- Global state management (C++ globals → Rust) - **likely needed**
- Python C API embedding (pyo3 patterns) - **likely needed**
- Perl C API embedding (raw FFI) - **likely needed**
- Custom string/buffer? - **likely skip, use String/Vec**

**Strategy**: Build one toy per complex subsystem, extract patterns, keep as reference artifacts.

---

## Toy Sequence

### Toy 1: String/Buffer Assessment (Maybe Skip)
**C++ Reference**: `String.cc`, `Buffer.cc`, `StaticBuffer.cc`

**Decision point**: Survey C++ implementation first
- If C++ String is simple wrapper → **use Rust `String`/`Vec<u8>` directly, skip toy**
- If C++ has weird invariants/behavior → build toy to understand

**If toy needed:**
- Understand C++ quirks that Rust `String` can't replicate
- Find simplest Rust equivalent (might still be just `String`)
- **Don't build custom unsafe unless C++ behavior requires it**

**Most likely outcome**: Skip this toy, use Rust stdlib

---

### Toy 2: ncurses FFI Integration
**C++ Reference**: `Curses.cc`, `Window.cc`

**Objective**: Validate ncurses FFI approach (`ncurses-rs` vs `pancurses`)

**What to learn:**
- Which Rust ncurses binding works best
- FFI patterns for window management
- Rust wrappers for ncurses state
- Terminal initialization/cleanup sequences

**SPEC.md focus:**
- C++ Curses wrapper behavior
- Window creation, rendering, input handling
- Color/attribute management via FFI

**Success criteria:**
- Basic TUI renders via Rust ncurses bindings
- Wrapper patterns for ncurses API extracted
- Choice: `ncurses-rs` (raw FFI) vs `pancurses` (safe wrapper)

---

### Toy 3: Global State Management
**C++ Reference**: `main.cc` (global variables section)

**Objective**: Validate Rust patterns for C++ global state

**What to learn:**
- `static` vs `lazy_static` vs `OnceCell` for globals
- Mutable global state patterns (unsafe `static mut` vs `Mutex`)
- Initialization order matching C++
- How to replicate C++ extern globals

**SPEC.md focus:**
- C++ globals used in MCL (config, currentSession, etc.)
- Initialization sequence
- Mutation patterns (when/how globals are modified)

**Success criteria:**
- Rust global state matches C++ access patterns
- Decision: which global state approach to use
- Patterns for safe-ish global mutation extracted

---

### Toy 4: Python Interpreter Embedding (pyo3)
**C++ Reference**: `plugins/PythonEmbeddedInterpreter.cc`

**Objective**: Validate pyo3 for Python C API embedding

**What to learn:**
- pyo3 equivalents for C API calls (Py_Initialize, PyImport_AddModule)
- How to match C++ Python integration with pyo3
- State management (globals dict, module setup)
- Conditional compilation patterns (`#[cfg(feature = "python")]`)

**SPEC.md focus:**
- C++ Python initialization sequence
- Variable passing (C++ → Python, Python → C++)
- Error handling (Python exceptions → Rust)

**Success criteria:**
- Rust toy executes Python code via pyo3
- Matches C++ behavior (same scripts work)
- Patterns for pyo3 integration extracted

---

### Toy 5: Perl Interpreter Embedding (Raw FFI)
**C++ Reference**: `plugins/PerlEmbeddedInterpreter.cc`

**Objective**: Validate raw Perl C API FFI

**What to learn:**
- Perl C API bindings in Rust (perl_alloc, perl_construct, etc.)
- XS initialization (boot_DynaLoader)
- Variable passing (C++ → Perl, Perl → C++)
- Conditional compilation (`#[cfg(feature = "perl")]`)

**SPEC.md focus:**
- C++ Perl initialization sequence
- XS bootstrap requirements
- Variable access (perl_get_sv, etc.)

**Success criteria:**
- Rust toy executes Perl code via raw C API
- Matches C++ behavior (same scripts work)
- Patterns for Perl FFI extracted

---

## Toy Development Workflow

**CRITICAL**: Start with learning goals, iterate to answer them

For each toy:

1. **Define learning goals** - Write `LEARNINGS.md` with questions/decisions
   - What do we need to learn?
   - What decisions must be made?
   - What patterns must be clear?
2. **Research & implement loop** - Iterate until goals met:
   - Study C++ reference
   - Try approaches in Rust
   - Test against C++ behavior
   - Update `LEARNINGS.md` with findings
3. **Finalize learnings** - Complete `LEARNINGS.md`:
   - Answer all questions
   - Document chosen approach
   - Extract portable patterns
4. **Optional: README.md** - Quick reference (only if toy is complex)

**Directory structure:**
```
toys/
  toy1_string_buffer/
    SPEC.md
    PLAN.md
    LEARNINGS.md
    README.md
    src/lib.rs
    tests/
  toy2_ncurses/
    ...
  toy3_globals/
    ...
  toy4_python/
    ...
  toy5_perl/
    ...
```

---

## Current Status

**Progress**: 5/5 toys complete ✅

- [x] **Toy 1 (String/Buffer)**: SKIPPED - Use Rust `String`/`Vec<u8>` with minimal wrappers
- [x] **Toy 2 (ncurses)**: COMPLETE - Use `ncurses` crate (raw FFI), skip `pancurses`
- [x] **Toy 3 (Globals)**: COMPLETE - Use `unsafe static mut` with helper functions
- [x] **Toy 4 (Python)**: COMPLETE - Use `pyo3` (simpler than C API)
- [x] **Toy 5 (Perl)**: COMPLETE - Use raw FFI with `PERL_SYS_INIT3` for modern Perl

## Success Criteria (Discovery Phase Complete)

- [x] Complex subsystems validated (only toys that were actually needed) ✅
- [x] Key decisions made:
  - [x] String/buffer: Rust stdlib sufficient ✅
  - [x] ncurses: `ncurses` crate (raw FFI) ✅
  - [x] Globals: `unsafe static mut` with helpers ✅
  - [x] Python: `pyo3` (simpler than C API) ✅
  - [x] Perl: Raw FFI with `PERL_SYS_INIT3` for modern Perl ✅
- [x] Built toys remain as reference artifacts ✅
- [x] **Ready to begin Execution phase (PORTING_HISTORY.md)** ✅

**Key principle applied**: Only built toys where Rust approach was unclear or C++ was genuinely complex

**Discovery phase: COMPLETE!** All patterns validated, ready for production porting.

---

## Estimated Effort (Discovery Phase)

**Optimistic** (String/Buffer skipped):
- Toy 2 (ncurses): 2-3 days
- Toy 3 (Globals): 1 day
- Toy 4 (Python): 2-3 days
- Toy 5 (Perl): 2-3 days
- **Total**: 7-10 days

**Pessimistic** (String/Buffer needed):
- Toy 1 (String/Buffer): 1-2 days
- Others: 7-10 days
- **Total**: 8-12 days

**Output**: Minimal validated patterns for genuinely complex subsystems

---

## Next Steps

1. Survey C++ String/Buffer implementation
2. **Decision**: Skip toy if Rust `String`/`Vec` is simpler
3. If skipping: Move to Toy 2 (ncurses)
4. If not: Build minimal String/Buffer toy
