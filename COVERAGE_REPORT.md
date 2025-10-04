# Test Coverage Report

**Last Updated**: 2025-10-04 01:11
**Tool**: cargo-llvm-cov
**Overall Coverage**: **73.99%** lines | **76.27%** regions | **79.62%** functions

## Summary

```
TOTAL                            9067              2152    76.27%         530               108    79.62%        5383              1400    73.99%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `action.rs` | 68.93% | 68.05% | 71.43% | 🟠 Moderate |
| `alias.rs` | 94.01% | 94.52% | 100.00% | 🟢 Excellent |
| `ansi.rs` | 88.65% | 90.45% | 100.00% | 🟡 Good |
| `config.rs` | 94.67% | 96.30% | 77.42% | 🟢 Excellent |
| `control.rs` | 72.75% | 74.25% | 87.50% | 🟡 Good |
| `curses.rs` | 18.56% | 13.55% | 40.00% | 🔴 Needs Work |
| `engine.rs` | 97.06% | 97.00% | 100.00% | 🟢 Excellent |
| `input_line.rs` | 80.46% | 83.05% | 69.23% | 🟡 Good |
| `input.rs` | 89.69% | 87.03% | 100.00% | 🟡 Good |
| `macro_def.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `mccp.rs` | 71.09% | 74.85% | 73.33% | 🟡 Good |
| `mud_selection.rs` | 82.18% | 86.77% | 73.33% | 🟡 Good |
| `mud.rs` | 88.32% | 90.65% | 85.19% | 🟡 Good |
| `offline_mud/game.rs` | 96.58% | 95.88% | 96.55% | 🟢 Excellent |
| `offline_mud/parser.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `output_window.rs` | 93.88% | 96.00% | 85.71% | 🟢 Excellent |
| `plugins/perl.rs` | 82.80% | 87.61% | 85.00% | 🟡 Good |
| `plugins/python.rs` | 88.14% | 89.22% | 90.91% | 🟡 Good |
| `plugins/stack.rs` | 73.13% | 78.10% | 55.00% | 🟡 Good |
| `screen.rs` | 94.79% | 96.75% | 96.30% | 🟢 Excellent |
| `scrollback.rs` | 91.71% | 93.13% | 100.00% | 🟢 Excellent |
| `select.rs` | 97.50% | 96.61% | 100.00% | 🟢 Excellent |
| `selectable.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `selection.rs` | 75.77% | 80.58% | 77.78% | 🟡 Good |
| `session.rs` | 85.92% | 88.62% | 87.50% | 🟡 Good |
| `socket.rs` | 91.54% | 92.35% | 100.00% | 🟢 Excellent |
| `status_line.rs` | 91.43% | 94.55% | 83.33% | 🟢 Excellent |
| `telnet.rs` | 97.67% | 99.02% | 100.00% | 🟢 Excellent |
| `tty.rs` | 31.11% | 25.84% | 55.56% | 🔴 Needs Work |
| `window.rs` | 57.14% | 56.09% | 66.67% | 🟠 Moderate |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `alias.rs` - 94.01%
- `config.rs` - 94.67%
- `engine.rs` - 97.06%
- `macro_def.rs` - 100.00%
- `offline_mud/game.rs` - 96.58%
- `offline_mud/parser.rs` - 100.00%
- `output_window.rs` - 93.88%
- `screen.rs` - 94.79%
- `scrollback.rs` - 91.71%
- `select.rs` - 97.50%
- `selectable.rs` - 100.00%
- `socket.rs` - 91.54%
- `status_line.rs` - 91.43%
- `telnet.rs` - 97.67%

### 🟡 Good (70-89% lines)
- `ansi.rs` - 88.65%
- `control.rs` - 72.75%
- `input.rs` - 89.69%
- `input_line.rs` - 80.46%
- `mccp.rs` - 71.09%
- `mud.rs` - 88.32%
- `mud_selection.rs` - 82.18%
- `plugins/perl.rs` - 82.80%
- `plugins/python.rs` - 88.14%
- `plugins/stack.rs` - 73.13%
- `selection.rs` - 75.77%
- `session.rs` - 85.92%

### 🟠 Moderate (40-69% lines)
- `action.rs` - 68.93%
- `window.rs` - 57.14%

### 🔴 Needs Work (<40% lines)
- `curses.rs` - 18.56% (ncurses FFI - TTY dependent)
- `main.rs` - 0.00% (event loop - needs integration tests)
- `tty.rs` - 31.11% (requires real TTY)

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 73.99% | ⏳ In Progress |
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
