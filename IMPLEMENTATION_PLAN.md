# MCL Rust Port - Execution Phase (Implementation Plan)

## Objective

Port MCL C++ codebase to Rust, applying patterns from Discovery phase. **Use Rust idioms when they simplify the port** - only force C++ fidelity where it actually reduces complexity.

**Principle**: Least complexity to working port. Not fidelity for fidelity's sake.

## Prerequisites

**Assumption**: Discovery phase (TOY_PLAN.md) complete with decisions on:
- String/buffer: Use Rust stdlib or custom? (likely stdlib)
- ncurses FFI: Which binding? (Toy 2)
- Global state: Which pattern? (Toy 3)
- Python embedding: pyo3 patterns (Toy 4)
- Perl embedding: FFI patterns (Toy 5)

**Reference**: `mcl-cpp-reference/` (~11k LOC, 29 .cc files, ~50 headers)

---

## Tier-by-Tier Port Strategy

### Tier 1: Foundation Types
**Files**: `String.cc`, `Buffer.cc`, `StaticBuffer.cc`, `Color.cc`, `List.h`

**Approach**: Use Rust stdlib unless C++ has non-standard behavior

**Steps**:
1. Survey C++ String/Buffer - decide Rust approach:
   - **Likely**: Map to `String`/`Vec<u8>` directly, create thin adapter if needed
   - **Unlikely**: Custom impl only if C++ has weird invariants
2. Port `Color` → `src/color.rs` (straightforward struct/enum)
3. Port `List<T>` → **use `Vec<T>` directly**, no custom wrapper
4. Unit tests for adapted/custom types only (stdlib needs no tests)

**Milestone**: Foundation types available (mostly stdlib), any adapters tested

**Key decision**: Don't reimplement what Rust stdlib provides better

---

### Tier 2: Core Abstractions
**Files**: `Selectable.cc`, `Selection.cc`, `TTY.cc`, `Config.cc`, `MUD.cc`, `Socket.cc`

**Approach**: Direct translation with Toy 3 patterns for any globals

**Steps**:
6. Port `Selectable`/`Selection` → `src/selectable.rs` (trait + structs)
7. Port `TTY` → `src/tty.rs` (unsafe wrapper around terminal FDs)
8. Port `Config` → `src/config.rs` (struct with C-like initialization)
9. Port `MUD` → `src/mud.rs` (struct with socket state)
10. Port `Socket` → `src/socket.rs` (`std::net::TcpStream` wrapper or raw FD)
11. Unit tests for network/config logic

**Milestone**: Core types compile, network/config functional

---

### Tier 3: UI Layer
**Files**: `Curses.cc`, `Window.cc`, `OutputWindow.cc`, `InputLine.cc`, `InputBox.cc`, `StatusLine.cc`, `Screen.cc`

**Approach**: Apply Toy 2 ncurses patterns

**Steps**:
12. Add ncurses dependency (choice from Toy 2: `ncurses-rs` or `pancurses`)
13. Port `Curses` → `src/curses.rs` using Toy 2 wrapper patterns
14. Port `Window` → `src/window.rs` (base widget)
15. Port `OutputWindow` → `src/output_window.rs`
16. Port `InputLine`, `InputBox` → `src/input_line.rs`, `src/input_box.rs`
17. Port `StatusLine` → `src/status_line.rs`
18. Port `Screen` → `src/screen.rs` (screen manager)
19. Integration tests (render basic UI)

**Milestone**: UI compiles, basic rendering works

---

### Tier 4: Logic & Base Interpreter Interface
**Files**: `Session.cc`, `Alias.cc`, `Hotkey.cc`, `Interpreter.cc`, `Pipe.cc`, `Embedded.cc`, `EmbeddedInterpreter.h`, `Chat.cc`, `Borg.cc`, `Group.cc`

**Approach**: Direct translation, prepare plugin trait

**Steps**:
20. Port `Session` → `src/session.rs` (state machine)
21. Port `Alias`, `Hotkey` → `src/alias.rs`, `src/hotkey.rs` (command tables)
22. Port `Interpreter`, `Pipe` → `src/interpreter.rs`, `src/pipe.rs`
23. Port `Embedded`/`EmbeddedInterpreter` → `src/embedded.rs` (base trait)
24. Port `Chat`, `Borg`, `Group` → `src/chat.rs`, `src/borg.rs`, `src/group.rs`
25. Integration tests (command processing)

**Milestone**: Logic layer compiles, base interpreter interface ready

---

### Tier 5a: Python Plugin (Optional Feature)
**Files**: `plugins/PythonEmbeddedInterpreter.cc`

**Approach**: Apply Toy 4 pyo3 patterns

**Steps**:
26. Add `pyo3` dependency behind `python` feature flag
27. Port `PythonEmbeddedInterpreter` → `src/plugins/python.rs` using Toy 4 patterns
28. Implement `EmbeddedInterpreter` trait for Python
29. Conditional compilation: `#[cfg(feature = "python")]`
30. Test Python script execution (compare with C++ MCL)

**Milestone**: Python scripting functional with `--features python`

---

### Tier 5b: Perl Plugin (Optional Feature)
**Files**: `plugins/PerlEmbeddedInterpreter.cc`

**Approach**: Apply Toy 5 Perl FFI patterns

**Steps**:
31. Create Perl C API bindings behind `perl` feature flag
32. Port `PerlEmbeddedInterpreter` → `src/plugins/perl.rs` using Toy 5 patterns
33. Implement `EmbeddedInterpreter` trait for Perl
34. Conditional compilation: `#[cfg(feature = "perl")]`
35. Test Perl script execution (compare with C++ MCL)

**Milestone**: Perl scripting functional with `--features perl`

---

### Tier 6: Application & Main Loop
**Files**: `main.cc`

**Approach**: Apply Toy 3 global patterns, wire everything together

**Steps**:
36. Port `main.cc` → `src/main.rs` (initialization sequence)
37. Set up global state using patterns from Toy 3
38. Implement main event loop
39. Conditional interpreter loading:
    - `#[cfg(feature = "python")]` → load Python
    - `#[cfg(feature = "perl")]` → load Perl
40. Wire up all subsystems (UI, networking, commands, interpreters)

**Milestone**: Binary compiles and runs

---

### Tier 7: Integration & Validation
**Steps**:
41. Manual smoke tests against C++ MCL reference
42. Fix segfaults, panics, undefined behavior
43. Validate core workflows:
    - Launch MCL
    - Connect to MUD server
    - Send/receive text
    - Execute commands (aliases, hotkeys)
    - Run scripts (Python/Perl if features enabled)
44. Test feature combinations:
    - Base (no features): `cargo run`
    - Python only: `cargo run --features python`
    - Perl only: `cargo run --features perl`
    - Both: `cargo run --features python,perl`
45. Golden tests (same inputs to Rust vs C++ → same outputs)

**Milestone**: Full MCL port operational with behavioral equivalence to C++

---

## Implementation Guidelines

### Port Discipline
- **Simplicity first**: Use Rust idioms when they're simpler than C++ patterns
- **Fidelity when useful**: Match C++ structure where it reduces complexity
- **Side-by-side comparison**: Keep C++ file open when porting
- **Smart mapping**:
  - C++ `std::string` → Rust `String` (not custom type)
  - C++ `std::vector` → Rust `Vec` (not custom type)
  - Unsafe only where FFI or C++ quirks require it
- **Document choices**: `// NOTE: Using Rust X instead of C++ Y (simpler)`

### Testing Strategy
- **Unit tests**: Per module, compare outputs with C++ reference
- **Integration tests**: Per tier milestone
- **Golden tests**: Use C++ MCL as oracle
- **Manual testing**: Side-by-side behavior validation

### Build Configuration

**Cargo.toml features:**
```toml
[features]
python = ["pyo3"]
perl = []  # Custom FFI bindings

[dependencies]
libc = "0.2"
ncurses = "5.101"  # or pancurses = "0.17" based on Toy 2
bitflags = "2.4"

[dependencies.pyo3]
version = "0.20"
optional = true
features = ["auto-initialize"]
```

---

## Success Criteria

- [ ] All 29 .cc files ported to .rs modules
- [ ] All 5 tiers complete with milestones achieved
- [ ] Binary compiles without errors (all feature combinations)
- [ ] Behavioral equivalence with C++ MCL:
  - [ ] Can connect to MUD server
  - [ ] Send/receive text
  - [ ] UI renders correctly (output, input, status)
  - [ ] Commands work (aliases, hotkeys, interpreter)
  - [ ] Python scripts work (`--features python`)
  - [ ] Perl scripts work (`--features perl`)
  - [ ] Both interpreters work together (`--features python,perl`)
- [ ] No crashes on startup or during basic operation
- [ ] CODE_MAP.md updated for all directories

---

## Out of Scope (First Pass)

- Idiomatic Rust refactoring (save for second pass)
- Memory safety improvements beyond C++ reference
- Automated test suite (beyond basic unit/integration)
- Cross-platform support (Linux only)
- Performance optimization
- Comprehensive documentation

---

## Risk Mitigation

**From Discovery Phase:**
- ✅ ncurses FFI: Validated in Toy 2
- ✅ Global state: Validated in Toy 3
- ✅ Python integration: Validated in Toy 4
- ✅ Perl integration: Validated in Toy 5

**Remaining Risks:**
- **Integration complexity**: Mitigate with tier-by-tier milestones
- **Edge case bugs**: Mitigate with thorough C++ reference comparison
- **Feature flag combinations**: Mitigate with explicit test matrix

---

## Estimated Effort (Execution Phase)

**Using Rust idioms where simpler:**

- Tier 1 (Foundation): 1-2 days (mostly stdlib mapping)
- Tier 2 (Core): 2-3 days
- Tier 3 (UI): 3-4 days (FFI validated in Discovery)
- Tier 4 (Logic): 2-3 days
- Tier 5a (Python): 1-2 days (patterns from Toy 4)
- Tier 5b (Perl): 1-2 days (patterns from Toy 5)
- Tier 6 (Main): 1-2 days (patterns from Toy 3)
- Tier 7 (Integration): 2-3 days

**Total**: 13-21 days

**Combined (Discovery + Execution)**:
- Optimistic: 20-31 days (String toy skipped, Rust idioms used)
- Pessimistic: 21-33 days (String toy needed, more C++ fidelity)

---

## Next Steps

1. Review Toy LEARNINGS.md files for patterns
2. Initialize Cargo workspace with dependencies
3. Begin Tier 1: Port String.cc using Toy 1 patterns
4. Follow tier-by-tier sequence with milestone validation
