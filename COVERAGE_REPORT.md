# Test Coverage Report

**Last Updated**: 2025-10-03 02:28
**Tool**: cargo-llvm-cov
**Overall Coverage**: **68.64%** lines | **73.70%** regions | **80.23%** functions

## Summary

```
TOTAL                            4343              1142    73.70%         263                52    80.23%        1604               503    68.64%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `ansi.rs` | 95.65% | 90.32% | 100.00% | 🟢 Excellent |
| `config.rs` | 90.00% | 87.50% | 71.43% | 🟢 Excellent |
| `control.rs` | 40.97% | 35.33% | 46.67% | 🟠 Moderate |
| `curses.rs` | 18.56% | 13.55% | 40.00% | 🔴 Needs Work |
| `engine.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `input_line.rs` | 88.46% | 84.21% | 75.00% | 🟡 Good |
| `input.rs` | 92.50% | 87.03% | 100.00% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `mccp.rs` | 83.33% | 74.85% | 73.33% | 🟡 Good |
| `mud.rs` | 94.44% | 92.68% | 100.00% | 🟢 Excellent |
| `offline_mud/game.rs` | 96.61% | 95.88% | 96.55% | 🟢 Excellent |
| `offline_mud/parser.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `output_window.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `plugins/stack.rs` | 80.90% | 80.39% | 59.46% | 🟡 Good |
| `screen.rs` | 97.78% | 97.07% | 95.24% | 🟢 Excellent |
| `scrollback.rs` | 100.00% | 99.26% | 100.00% | 🟢 Excellent |
| `select.rs` | 100.00% | 96.61% | 100.00% | 🟢 Excellent |
| `selectable.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `session.rs` | 96.15% | 91.43% | 80.00% | 🟢 Excellent |
| `socket.rs` | 92.86% | 92.35% | 100.00% | 🟢 Excellent |
| `status_line.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `telnet.rs` | 100.00% | 99.02% | 100.00% | 🟢 Excellent |
| `tty.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `window.rs` | 89.19% | 84.51% | 85.71% | 🟡 Good |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `ansi.rs` - 95.65%
- `config.rs` - 90.00%
- `engine.rs` - 100.00%
- `input.rs` - 92.50%
- `mud.rs` - 94.44%
- `offline_mud/game.rs` - 96.61%
- `offline_mud/parser.rs` - 100.00%
- `output_window.rs` - 100.00%
- `screen.rs` - 97.78%
- `scrollback.rs` - 100.00%
- `select.rs` - 100.00%
- `selectable.rs` - 100.00%
- `session.rs` - 96.15%
- `socket.rs` - 92.86%
- `status_line.rs` - 100.00%
- `telnet.rs` - 100.00%

### 🟡 Good (70-89% lines)
- `input_line.rs` - 88.46%
- `mccp.rs` - 83.33%
- `plugins/stack.rs` - 80.90%
- `window.rs` - 89.19%

### 🟠 Moderate (40-69% lines)
- `control.rs` - 40.97%

### 🔴 Needs Work (<40% lines)
- `curses.rs` - 18.56% (ncurses FFI - TTY dependent)
- `main.rs` - 0.00% (event loop - needs integration tests)
- `tty.rs` - 0.00% (requires real TTY)

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 68.64% | ⏳ In Progress |
| Critical Paths | ≥95% | Check modules above | ✅ Met |
| New Modules | ≥80% | - | Policy |

## How to Update This Report

```bash
# Regenerate coverage report
./scripts/generate-coverage-report.sh

# Or use shortcuts
make coverage-report
just coverage-report
```

## Quick Commands

```bash
# View interactive HTML report
make coverage              # Generates and opens HTML report

# Update this markdown report
make coverage-report       # Regenerates COVERAGE_REPORT.md

# Run tests with coverage
cargo llvm-cov --html      # Detailed HTML
cargo llvm-cov --summary-only  # Terminal summary
```

---

*This report is auto-generated from `cargo llvm-cov` output.*
*Modules with <40% coverage are expected (TTY/FFI limitations) and documented.*
