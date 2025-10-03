# Development Guide

## Quick Start

### One-Time Setup

Install development tools:
```bash
# Using Make
make install-tools

# Or manually
cargo install cargo-llvm-cov    # Coverage reports
cargo install cargo-audit        # Security auditing
cargo install cargo-watch        # Auto-rebuild on file changes
cargo install just               # Modern task runner (optional)
```

## Task Automation

**You have 3 options** (pick your preference):

### 1. **Makefile** (Recommended - No Extra Install)

```bash
make help              # Show all commands
make test              # Run tests
make coverage          # Generate coverage report
make run-python        # Run with Python plugin
make pre-commit        # Format + lint + test (before commits)
```

### 2. **`just` Task Runner** (Modern Alternative)

Install: `cargo install just`

```bash
just                   # Show all commands
just test              # Run tests
just coverage          # Generate coverage report
just run-python        # Run with Python plugin
just pre-commit        # Format + lint + test
```

### 3. **Cargo Aliases** (Built-in, Simple)

```bash
cargo t                # cargo test
cargo ttt              # Run with TTY (./test-with-tty.sh)
cargo cov              # Coverage report
cargo bp               # Build with Python
```

Defined in `.cargo/config.toml`

## Testing

### Run Tests

```bash
# Standard test suite (59 tests)
cargo test             # Or: make test / just test

# Specific test types
cargo test --lib       # Unit tests only (57 tests)
cargo test --tests     # Integration tests only (2 tests)
cargo test --features python   # With Python plugin

# With pseudo-TTY (ncurses tests execute)
./test-with-tty.sh     # Or: make test-tty / just test-tty
```

### Coverage Reports

#### **Using cargo-llvm-cov** (Recommended for macOS)

```bash
# Install
cargo install cargo-llvm-cov

# Generate HTML report
cargo llvm-cov --html --open

# Or use shortcuts
make coverage          # Generates and opens HTML report
just coverage          # Same

# Update COVERAGE_REPORT.md (markdown report)
make coverage-report   # Auto-generated from llvm-cov data
just coverage-report
```

#### **Auto-Update Coverage on Git Push** (Recommended)

Install a git hook to automatically keep `COVERAGE_REPORT.md` up-to-date:

```bash
# Install git hooks
make install-hooks     # Or: just install-hooks

# Now when you push, the hook will:
# 1. Run coverage analysis
# 2. Update COVERAGE_REPORT.md if needed
# 3. Remind you to commit the changes

# Uninstall hooks
make uninstall-hooks   # Or: just uninstall-hooks

# Bypass hook on a specific push
git push --no-verify
```

The hook ensures your coverage report is always current and diffs nicely in git!

#### **Using Tarpaulin** (Linux only)

```bash
# Install
cargo install cargo-tarpaulin

# Generate report
cargo tarpaulin --out Html --output-dir target/tarpaulin
```

#### **Coverage Targets**

Current baseline (post-event loop implementation):
- **Target**: 80%+ overall coverage
- **Critical paths**: 95%+ (network, session, event loop)
- **Exclude**: FFI wrappers, unsafe blocks (tested via integration tests)

### Watch Mode (Auto-test on file changes)

```bash
# Install
cargo install cargo-watch

# Run watch mode
cargo watch -x 'test --lib' -x clippy

# Or use shortcuts
make watch
just watch
```

## Building

```bash
# Debug build
cargo build            # Or: make build / just build

# Release build (optimized)
cargo build --release  # Or: make release / just release

# With features
cargo build --features python       # Python plugin
cargo build --features perl         # Perl plugin
cargo build --all-features          # All plugins
```

## Running

### Standard Mode

```bash
cargo run              # Basic client (no plugins)
cargo run --features python         # With Python
cargo run --features perl           # With Perl

# Or shortcuts
make run-python
just run-python
```

### Headless Mode

```bash
# Start headless instance
cargo run -- --headless --instance mybot

# Attach to instance from another terminal
cargo run -- --attach mybot

# Or shortcuts
just headless mybot
just attach mybot
```

### Demo with MUD Connection

```bash
# Connect to MUD on startup
MCL_CONNECT=127.0.0.1:4000 cargo run

# Or
MCL_CONNECT=mud.example.com:4000 make demo
```

## Code Quality

### Linting

```bash
# Run Clippy (Rust linter)
cargo clippy --all-features -- -D warnings

# Or shortcuts
make lint
just lint
```

### Formatting

```bash
# Format code
cargo fmt              # Or: make fmt / just fmt

# Check formatting (CI)
cargo fmt -- --check   # Or: make fmt-check / just fmt-check
```

### Security Auditing

```bash
# Install
cargo install cargo-audit

# Run audit
cargo audit            # Or: make audit / just audit
```

## Documentation

```bash
# Generate and open docs
cargo doc --open --no-deps

# Include private items (for development)
cargo doc --open --document-private-items

# Or shortcuts
make doc
just doc-private
```

## Common Workflows

### Before Committing

```bash
# Run all pre-commit checks
make pre-commit        # Formats, lints, tests
just pre-commit

# Or manually
cargo fmt
cargo clippy --all-features -- -D warnings
cargo test
```

### Adding a New Feature

1. **Create feature branch**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Develop with watch mode**
   ```bash
   make watch          # Auto-test on save
   ```

3. **Run coverage before PR**
   ```bash
   make coverage
   # Check coverage report, aim for 80%+
   ```

4. **Pre-commit checks**
   ```bash
   make pre-commit
   ```

5. **Create PR**
   ```bash
   git add .
   git commit -m "feat: my feature"
   git push origin feature/my-feature
   ```

### Debugging

```bash
# Verbose test output
cargo test -- --nocapture

# Run tests serially (better for debugging)
cargo test -- --test-threads=1

# With backtrace
RUST_BACKTRACE=1 cargo test

# Single test
cargo test socket::tests::nonblocking_connect_loopback
```

## CI/CD

### GitHub Actions Setup

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Lint
        run: cargo clippy --all-features -- -D warnings

      - name: Test
        run: cargo test --all-features

      - name: Build release
        run: cargo build --release --all-features

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Generate coverage
        run: cargo llvm-cov --lcov --output-path lcov.info

      - name: Upload to codecov.io
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
```

### Using Make/Just in CI

```yaml
# In CI job
- name: Run all checks
  run: make ci          # Or: just ci
```

## Dependency Management

### Update Dependencies

```bash
# Install cargo-outdated
cargo install cargo-outdated

# Check outdated deps
cargo outdated

# Update Cargo.lock
cargo update

# Or shortcut
just update
```

### Add Dependency

```bash
# Add to Cargo.toml
cargo add <crate-name>

# Add dev dependency
cargo add --dev <crate-name>

# Add optional dependency (for features)
cargo add <crate-name> --optional
```

## Troubleshooting

### Python/Perl Dynamic Library Issues

**Problem**: `cargo test --all-features` fails with library not found

**Solution**:
```bash
# macOS - Add to shell rc file
export DYLD_LIBRARY_PATH="/opt/homebrew/opt/python@3.10/lib:$DYLD_LIBRARY_PATH"

# Or use without features
cargo test              # Works fine, skips plugin tests
```

### ncurses Tests Failing

**Problem**: `curses::tests::*` fail in CI or non-TTY environment

**Solution**: Tests auto-skip gracefully. To run them:
```bash
./test-with-tty.sh     # Provides pseudo-TTY
# Or from real terminal:
cargo test --lib curses::tests -- --nocapture
```

### Coverage Not Generating

**Problem**: `cargo llvm-cov` reports 0% coverage

**Solution**:
```bash
# Clean and rebuild
cargo clean
cargo llvm-cov --html

# Ensure tests are running
cargo llvm-cov --html -- --nocapture
```

## Performance Profiling

### Using `cargo-flamegraph`

```bash
# Install
cargo install flamegraph

# Generate flamegraph
cargo flamegraph

# Output: flamegraph.svg
```

### Using `perf` (Linux)

```bash
# Record
cargo build --release
perf record --call-graph dwarf ./target/release/okros

# Report
perf report
```

## References

- **Cargo Book**: https://doc.rust-lang.org/cargo/
- **cargo-llvm-cov**: https://github.com/taiki-e/cargo-llvm-cov
- **just**: https://github.com/casey/just
- **Testing in Rust**: https://doc.rust-lang.org/book/ch11-00-testing.html
