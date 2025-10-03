# MCL Rust - Development Makefile
# Provides common development tasks (like npm scripts in Node.js)

.PHONY: help build test clean run dev check coverage lint fmt doc

# Default target
help:
	@echo "MCL Rust Development Commands"
	@echo ""
	@echo "Building:"
	@echo "  make build          - Build debug binary"
	@echo "  make release        - Build optimized release binary"
	@echo "  make build-python   - Build with Python plugin"
	@echo "  make build-perl     - Build with Perl plugin"
	@echo "  make build-all      - Build with all features"
	@echo ""
	@echo "Testing:"
	@echo "  make test           - Run all tests (59 tests, ncurses skip gracefully)"
	@echo "  make test-tty       - Run tests with pseudo-TTY (ncurses tests execute)"
	@echo "  make test-lib       - Run only library unit tests"
	@echo "  make test-int       - Run integration tests only"
	@echo "  make test-python    - Run tests with Python feature"
	@echo "  make test-perl      - Run tests with Perl feature"
	@echo "  make coverage       - Generate HTML coverage report (opens browser)"
	@echo "  make coverage-report - Update COVERAGE_REPORT.md from coverage data"
	@echo ""
	@echo "Running:"
	@echo "  make run            - Run client (basic, no plugins)"
	@echo "  make run-python     - Run with Python plugin"
	@echo "  make run-perl       - Run with Perl plugin"
	@echo "  make run-all        - Run with all features"
	@echo "  make demo           - Run with demo MUD connection (MCL_CONNECT env var)"
	@echo ""
	@echo "Code Quality:"
	@echo "  make check          - Fast syntax check (no codegen)"
	@echo "  make lint           - Run clippy lints"
	@echo "  make fmt            - Format code"
	@echo "  make fmt-check      - Check formatting without changes"
	@echo "  make audit          - Security audit (requires cargo-audit)"
	@echo ""
	@echo "Documentation:"
	@echo "  make doc            - Generate and open documentation"
	@echo "  make doc-private    - Generate docs including private items"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean          - Remove build artifacts"
	@echo "  make clean-all      - Remove all generated files (coverage, docs, etc.)"
	@echo ""
	@echo "Git Hooks:"
	@echo "  make install-hooks  - Install git hooks (auto-update coverage on push)"
	@echo "  make uninstall-hooks - Remove git hooks"

# === Building ===

build:
	cargo build

release:
	cargo build --release

build-python:
	cargo build --features python

build-perl:
	cargo build --features perl

build-all:
	cargo build --all-features

# === Testing ===

test:
	cargo test

test-tty:
	./test-with-tty.sh

test-lib:
	cargo test --lib

test-int:
	cargo test --tests

test-python:
	cargo test --features python

test-perl:
	cargo test --features perl

test-all:
	cargo test --all-features

# Coverage (requires: cargo install cargo-llvm-cov)
coverage:
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { echo "Error: cargo-llvm-cov not installed. Run: cargo install cargo-llvm-cov"; exit 1; }
	cargo llvm-cov --html --open
	@echo ""
	@echo "Coverage report generated: target/llvm-cov/html/index.html"

# Generate markdown coverage report
coverage-report:
	./scripts/generate-coverage-report.sh

# Install git hooks for automatic coverage updates
install-hooks:
	./scripts/install-git-hooks.sh install

# Uninstall git hooks
uninstall-hooks:
	./scripts/install-git-hooks.sh uninstall

# === Running ===

run:
	cargo run

run-python:
	cargo run --features python

run-perl:
	cargo run --features perl

run-all:
	cargo run --all-features

# Demo with connection (set MCL_CONNECT=127.0.0.1:4000)
demo:
	@if [ -z "$$MCL_CONNECT" ]; then \
		echo "Error: Set MCL_CONNECT env var (e.g., MCL_CONNECT=127.0.0.1:4000 make demo)"; \
		exit 1; \
	fi
	cargo run

# === Code Quality ===

check:
	cargo check --all-features

lint:
	cargo clippy --all-features -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

# Security audit (requires: cargo install cargo-audit)
audit:
	@command -v cargo-audit >/dev/null 2>&1 || { echo "Error: cargo-audit not installed. Run: cargo install cargo-audit"; exit 1; }
	cargo audit

# === Documentation ===

doc:
	cargo doc --open --no-deps

doc-private:
	cargo doc --open --no-deps --document-private-items

# === Cleanup ===

clean:
	cargo clean

clean-all: clean
	rm -rf target/llvm-cov
	rm -rf target/doc
	find . -name "*.profraw" -delete
	find . -name "*.profdata" -delete

# === CI/CD Tasks ===

ci-test:
	cargo test --all-features

ci-lint:
	cargo fmt -- --check
	cargo clippy --all-features -- -D warnings

ci-build:
	cargo build --all-features --release

ci: ci-lint ci-test ci-build
	@echo "✅ All CI checks passed"

# === Development Workflow ===

# Pre-commit checks
pre-commit: fmt lint test
	@echo "✅ Pre-commit checks passed"

# Watch mode (requires: cargo install cargo-watch)
watch:
	@command -v cargo-watch >/dev/null 2>&1 || { echo "Error: cargo-watch not installed. Run: cargo install cargo-watch"; exit 1; }
	cargo watch -x 'test --lib' -x 'clippy'

# Install development tools
install-tools:
	@echo "Installing Rust development tools..."
	cargo install cargo-llvm-cov      # Coverage
	cargo install cargo-audit         # Security auditing
	cargo install cargo-watch         # File watching
	cargo install cargo-outdated      # Dependency updates
	@echo "✅ Development tools installed"
