# Toy 1: String/Buffer Analysis - LEARNINGS

## Decision: TOY SKIPPED - Use Rust Stdlib

After surveying C++ String/Buffer implementations, determined that **building a toy is unnecessary**. Rust stdlib (`String`, `Vec<u8>`) with minimal adapters is simpler than porting C++ patterns.

---

## C++ Implementation Analysis

### String Class (`String.h`, `String.cc`)
**Purpose**: Simple `char*` wrapper with reference semantics

**Implementation**:
- Manual memory management (`new char[]`, `delete[]`)
- Empty string optimization (static `""` pointer, not allocated)
- Printf formatting method
- Copy constructor/assignment

**Key Quirk**:
- ⚠️ **Case-insensitive comparisons**: `operator==` uses `strcasecmp` (ignores case)
- This is the ONLY non-standard behavior that matters

**Rust Approach**:
```rust
// Just use String, add custom comparison when needed
impl PartialEq<str> for CaseInsensitiveString {
    fn eq(&self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}
```

**No toy needed**: Trivial wrapper, no unsafe required

---

### Buffer Class (`Buffer.h`, `Buffer.cc`)
**Purpose**: Growable text buffer with queue operations

**Implementation**:
- Growable buffer with size tracking (power-of-2 resizing)
- len (used) vs size (allocated)
- Operations: strcat, printf, shift (dequeue front), unshift (insert front)
- Overflow flag
- Virtual `flush()` method (for subclasses)

**Rust Approach**:
```rust
struct Buffer {
    data: Vec<u8>,
    overflowed: bool,
}

impl Buffer {
    fn shift(&mut self, count: usize) {
        self.data.drain(0..count);
    }

    fn unshift(&mut self, bytes: &[u8]) {
        self.data.splice(0..0, bytes.iter().copied());
    }

    // strcat → extend_from_slice
    // printf → write!(self.data, ...)
}
```

**No toy needed**: Straightforward `Vec<u8>` wrapper, no unsafe required

---

### StaticBuffer Class (`StaticBuffer.h`, `StaticBuffer.cc`)
**Purpose**: Global circular buffer pool for temporary strings

**Implementation**:
- 64KB static circular buffer
- Stack-like allocation (getstatic/shrinkstatic)
- Used for temporary formatted strings (like `sprintf`)
- Tracks stats (gets, shrinks, fails, size)

**Design Pattern**:
```c++
const char* Sprintf(const char *fmt, ...) {
    StaticBuffer s;  // Allocates from circular pool
    vsnprintf(s, s.size(), fmt, va);
    return s;  // Destructor shrinks if possible
}
```

**Rust Approach**:
```rust
// Just use String/format! - Rust allocator is fine
fn sprintf(fmt: &str, args: ...) -> String {
    format!(fmt, args)
}
```

**Skip optimization**: C-era memory pooling trick. Rust allocator is modern and efficient, no need for custom pool.

---

## Key Learnings

### 1. Case-Insensitive String Comparison
**C++ behavior**: `String::operator==` uses `strcasecmp` (case-insensitive)

**Porting strategy**:
- Option A: Wrapper struct with custom `PartialEq`
- Option B: Use `.eq_ignore_ascii_case()` at call sites
- **Decision**: Evaluate usage during port, likely Option B (explicit at call sites)

### 2. Buffer Operations Map Cleanly to Vec
**C++ → Rust mapping**:
- `strcat(text)` → `data.extend_from_slice(text.as_bytes())`
- `shift(n)` → `data.drain(0..n)`
- `unshift(text, len)` → `data.splice(0..0, text.iter().copied())`
- `printf(fmt, ...)` → `write!(data, fmt, ...)`

**No unsafe needed** - Vec handles resizing, bounds checking optional

### 3. StaticBuffer Optimization Unnecessary
**C++ rationale**: Reduce malloc overhead for temporary strings (2000s-era optimization)

**Rust reality**: Modern allocator + compiler optimizations make this premature optimization

**Skip pattern**: Use `String`/`format!` directly

### 4. No Toy Implementation Needed
**Why skip**:
- String quirks easily handled with stdlib + traits
- Buffer is straightforward Vec wrapper
- StaticBuffer optimization is obsolete
- **Building toy would be more complex than direct port**

**Port strategy**:
- Create thin `Buffer` wrapper around `Vec<u8>` during Tier 1
- Handle case-insensitive string comparison as needed
- Skip StaticBuffer entirely, use `String`/`format!`

---

## Porting Decisions

### String Handling
✅ **Use Rust `String` directly**
- Add case-insensitive comparison where needed
- Use `format!` for printf-style operations
- No custom memory management

### Buffer Handling
✅ **Create minimal `Buffer` wrapper around `Vec<u8>`**
- Implement shift/unshift operations
- Track overflow flag
- Map strcat/printf to Vec operations

### StaticBuffer Handling
✅ **Skip entirely, use regular `String` allocation**
- Replace `Sprintf()` calls with `format!()`
- Rely on Rust allocator efficiency

---

## Time Saved

**Toy implementation avoided**: 1-2 days
**Reason**: Rust stdlib is simpler than porting C++ patterns

**Outcome**: Move directly to Toy 2 (ncurses FFI) where validation is actually needed

---

## References

- `mcl-cpp-reference/String.h` - Simple char* wrapper (20 LOC)
- `mcl-cpp-reference/String.cc` - Printf implementation (20 LOC)
- `mcl-cpp-reference/Buffer.h` - Growable buffer interface (50 LOC)
- `mcl-cpp-reference/Buffer.cc` - Buffer implementation (116 LOC)
- `mcl-cpp-reference/StaticBuffer.h` - Circular pool interface (28 LOC)
- `mcl-cpp-reference/StaticBuffer.cc` - Pool implementation (111 LOC)
