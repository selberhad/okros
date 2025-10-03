# MCL Rust - Just Task Runner
# Install: cargo install just
# Run: just <task>
# List tasks: just --list

# Default recipe (show help)
default:
    @just --list

# === Building ===

# Build debug binary
build:
    cargo build

# Build release binary
release:
    cargo build --release

# Build with Python plugin
build-python:
    cargo build --features python

# Build with Perl plugin
build-perl:
    cargo build --features perl

# Build with all features
build-all:
    cargo build --all-features

# === Testing ===

# Run all tests
test:
    cargo test

# Run tests with pseudo-TTY (ncurses tests execute)
test-tty:
    ./test-with-tty.sh

# Run only library tests
test-lib:
    cargo test --lib

# Run integration tests
test-int:
    cargo test --tests

# Run tests with all features
test-all:
    cargo test --all-features

# Generate coverage report (HTML)
coverage:
    #!/usr/bin/env bash
    if ! command -v cargo-llvm-cov &> /dev/null; then
        echo "Installing cargo-llvm-cov..."
        cargo install cargo-llvm-cov
    fi
    cargo llvm-cov --html --open
    echo ""
    echo "Coverage report: target/llvm-cov/html/index.html"

# Update COVERAGE_REPORT.md from coverage data
coverage-report:
    ./scripts/generate-coverage-report.sh

# Install git hooks for auto-coverage updates
install-hooks:
    ./scripts/install-git-hooks.sh install

# Uninstall git hooks
uninstall-hooks:
    ./scripts/install-git-hooks.sh uninstall

# === Running ===

# Run client (basic, no plugins)
run *ARGS:
    cargo run {{ARGS}}

# Run with Python plugin
run-python *ARGS:
    cargo run --features python {{ARGS}}

# Run with Perl plugin
run-perl *ARGS:
    cargo run --features perl {{ARGS}}

# Run with all features
run-all *ARGS:
    cargo run --all-features {{ARGS}}

# Run headless mode (requires instance name)
headless instance='default':
    cargo run -- --headless --instance {{instance}}

# Attach to headless instance
attach instance='default':
    cargo run -- --attach {{instance}}

# === Code Quality ===

# Fast syntax check
check:
    cargo check --all-features

# Run clippy lints
lint:
    cargo clippy --all-features -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt -- --check

# Security audit
audit:
    #!/usr/bin/env bash
    if ! command -v cargo-audit &> /dev/null; then
        echo "Installing cargo-audit..."
        cargo install cargo-audit
    fi
    cargo audit

# === Documentation ===

# Generate and open docs
doc:
    cargo doc --open --no-deps

# Generate docs with private items
doc-private:
    cargo doc --open --no-deps --document-private-items

# === Cleanup ===

# Remove build artifacts
clean:
    cargo clean

# Remove all generated files
clean-all:
    cargo clean
    rm -rf target/llvm-cov target/doc
    find . -name "*.profraw" -delete
    find . -name "*.profdata" -delete

# === CI/CD ===

# Run all CI checks
ci: fmt-check lint test-all release
    @echo "✅ All CI checks passed"

# === Development Workflow ===

# Pre-commit checks
pre-commit: fmt lint test
    @echo "✅ Pre-commit checks passed"

# Watch mode (auto-test on file changes)
watch:
    #!/usr/bin/env bash
    if ! command -v cargo-watch &> /dev/null; then
        echo "Installing cargo-watch..."
        cargo install cargo-watch
    fi
    cargo watch -x 'test --lib' -x clippy

# Install all development tools
install-tools:
    @echo "Installing development tools..."
    cargo install cargo-llvm-cov
    cargo install cargo-audit
    cargo install cargo-watch
    cargo install cargo-outdated
    cargo install just
    @echo "✅ Development tools installed"

# Update dependencies
update:
    cargo update
    cargo outdated
