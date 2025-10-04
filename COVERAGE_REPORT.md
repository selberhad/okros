# Test Coverage Report

**Last Updated**: 2025-10-03 23:38
**Tool**: cargo-llvm-cov
**Overall Coverage**: **74.01%** lines | **76.13%** regions | **80.83%** functions

## Summary

```
TOTAL                            8713              2080    76.13%         506                97    80.83%        5125              1332    74.01%           0                 0         -
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
| `input_line.rs` | 83.58% | 84.21% | 75.00% | 🟡 Good |
| `input.rs` | 89.69% | 87.03% | 100.00% | 🟡 Good |
| `macro_def.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `mccp.rs` | 71.09% | 74.85% | 73.33% | 🟡 Good |
| `mud_selection.rs` | 87.36% | 90.06% | 84.62% | 🟡 Good |
| `mud.rs` | 88.21% | 90.65% | 85.19% | 🟡 Good |
| `offline_mud/game.rs` | 96.58% | 95.88% | 96.55% | 🟢 Excellent |
| `offline_mud/parser.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `output_window.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `plugins/perl.rs` | 82.80% | 87.61% | 85.00% | 🟡 Good |
| `plugins/python.rs` | 88.14% | 89.22% | 90.91% | 🟡 Good |
| `plugins/stack.rs` | 73.13% | 78.10% | 55.00% | 🟡 Good |
| `screen.rs` | 95.78% | 97.78% | 96.15% | 🟢 Excellent |
| `scrollback.rs` | 91.71% | 93.13% | 100.00% | 🟢 Excellent |
| `select.rs` | 97.50% | 96.61% | 100.00% | 🟢 Excellent |
| `selectable.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `selection.rs` | 71.14% | 76.47% | 75.00% | 🟡 Good |
| `session.rs` | 85.92% | 88.62% | 87.50% | 🟡 Good |
| `socket.rs` | 91.54% | 92.35% | 100.00% | 🟢 Excellent |
| `status_line.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `telnet.rs` | 97.67% | 99.02% | 100.00% | 🟢 Excellent |
| `tty.rs` | 31.11% | 25.84% | 55.56% | 🔴 Needs Work |
| `window.rs` | 88.24% | 84.51% | 85.71% | 🟡 Good |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `alias.rs` - 94.01%
- `config.rs` - 94.67%
- `engine.rs` - 97.06%
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
- `control.rs` - 72.75%
- `input.rs` - 89.69%
- `input_line.rs` - 83.58%
- `mccp.rs` - 71.09%
- `mud.rs` - 88.21%
- `mud_selection.rs` - 87.36%
- `plugins/perl.rs` - 82.80%
- `plugins/python.rs` - 88.14%
- `plugins/stack.rs` - 73.13%
- `selection.rs` - 71.14%
- `session.rs` - 85.92%
- `window.rs` - 88.24%

### 🟠 Moderate (40-69% lines)
- `action.rs` - 68.93%

### 🔴 Needs Work (<40% lines)
- `curses.rs` - 18.56% (ncurses FFI - TTY dependent)
- `main.rs` - 0.00% (event loop - needs integration tests)
- `tty.rs` - 31.11% (requires real TTY)

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 74.01% | ⏳ In Progress |
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
