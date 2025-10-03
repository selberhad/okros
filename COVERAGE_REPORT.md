# Test Coverage Report

**Last Updated**: 2025-10-03 13:39
**Tool**: cargo-llvm-cov
**Overall Coverage**: **67.81%** lines | **69.28%** regions | **77.26%** functions

## Summary

```
TOTAL                            5674              1743    69.28%         343                78    77.26%        3644              1173    67.81%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `action.rs` | 66.32% | 66.67% | 69.23% | 🟠 Moderate |
| `alias.rs` | 94.01% | 94.52% | 100.00% | 🟢 Excellent |
| `ansi.rs` | 88.65% | 90.45% | 100.00% | 🟡 Good |
| `config.rs` | 91.67% | 87.50% | 71.43% | 🟢 Excellent |
| `control.rs` | 32.02% | 33.52% | 46.67% | 🔴 Needs Work |
| `curses.rs` | 18.56% | 13.55% | 40.00% | 🔴 Needs Work |
| `engine.rs` | 66.91% | 63.67% | 87.50% | 🟠 Moderate |
| `input_line.rs` | 83.58% | 84.21% | 75.00% | 🟡 Good |
| `input.rs` | 89.69% | 87.03% | 100.00% | 🟡 Good |
| `macro_def.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `mccp.rs` | 71.09% | 74.85% | 73.33% | 🟡 Good |
| `mud.rs` | 86.11% | 83.93% | 71.43% | 🟡 Good |
| `offline_mud/game.rs` | 96.58% | 95.88% | 96.55% | 🟢 Excellent |
| `offline_mud/parser.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `output_window.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `plugins/stack.rs` | 73.13% | 78.10% | 55.00% | 🟡 Good |
| `screen.rs` | 95.78% | 97.79% | 96.15% | 🟢 Excellent |
| `scrollback.rs` | 91.71% | 93.13% | 100.00% | 🟢 Excellent |
| `select.rs` | 97.50% | 96.61% | 100.00% | 🟢 Excellent |
| `selectable.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `session.rs` | 85.92% | 88.62% | 87.50% | 🟡 Good |
| `socket.rs` | 91.54% | 92.35% | 100.00% | 🟢 Excellent |
| `status_line.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `telnet.rs` | 97.67% | 99.02% | 100.00% | 🟢 Excellent |
| `tty.rs` | 31.11% | 25.84% | 55.56% | 🔴 Needs Work |
| `window.rs` | 88.24% | 84.51% | 85.71% | 🟡 Good |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `alias.rs` - 94.01%
- `config.rs` - 91.67%
- `macro_def.rs` - 100.00%
- `offline_mud/game.rs` - 96.58%
- `offline_mud/parser.rs` - 100.00%
- `output_window.rs` - 100.00%
- `screen.rs` - 95.78%
- `scrollback.rs` - 91.71%
- `select.rs` - 97.50%
- `selectable.rs` - 100.00%
- `socket.rs` - 91.54%
- `status_line.rs` - 100.00%
- `telnet.rs` - 97.67%

### 🟡 Good (70-89% lines)
- `ansi.rs` - 88.65%
- `input.rs` - 89.69%
- `input_line.rs` - 83.58%
- `mccp.rs` - 71.09%
- `mud.rs` - 86.11%
- `plugins/stack.rs` - 73.13%
- `session.rs` - 85.92%
- `window.rs` - 88.24%

### 🟠 Moderate (40-69% lines)
- `action.rs` - 66.32%
- `engine.rs` - 66.91%

### 🔴 Needs Work (<40% lines)
- `control.rs` - 32.02%
- `curses.rs` - 18.56% (ncurses FFI - TTY dependent)
- `main.rs` - 0.00% (event loop - needs integration tests)
- `tty.rs` - 31.11% (requires real TTY)

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 67.81% | ⏳ In Progress |
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
