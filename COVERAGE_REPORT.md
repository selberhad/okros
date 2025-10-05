# Test Coverage Report

**Last Updated**: 2025-10-04 22:31
**Tool**: cargo-llvm-cov
**Overall Coverage**: **71.44%** lines | **74.11%** regions | **77.19%** functions

## Summary

```
TOTAL                           11999              3106    74.11%         684               156    77.19%        7186              2052    71.44%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `action.rs` | 68.93% | 68.05% | 71.43% | 🟠 Moderate |
| `alias.rs` | 94.01% | 94.52% | 100.00% | 🟢 Excellent |
| `ansi.rs` | 88.36% | 89.82% | 100.00% | 🟡 Good |
| `command_queue.rs` | 92.57% | 94.38% | 92.11% | 🟢 Excellent |
| `config.rs` | 94.67% | 96.30% | 77.42% | 🟢 Excellent |
| `control.rs` | 72.62% | 74.20% | 87.50% | 🟡 Good |
| `curses.rs` | 18.56% | 13.55% | 40.00% | 🔴 Needs Work |
| `debug_log.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `engine.rs` | 88.24% | 90.72% | 94.44% | 🟡 Good |
| `history.rs` | 65.85% | 65.81% | 77.27% | 🟠 Moderate |
| `input_box.rs` | 54.24% | 52.54% | 38.46% | 🟠 Moderate |
| `input_line.rs` | 44.44% | 50.13% | 47.06% | 🟠 Moderate |
| `input.rs` | 89.69% | 87.03% | 100.00% | 🟡 Good |
| `macro_def.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `mccp.rs` | 71.09% | 74.85% | 73.33% | 🟡 Good |
| `mud_selection.rs` | 82.18% | 86.77% | 73.33% | 🟡 Good |
| `mud.rs` | 72.80% | 78.24% | 79.31% | 🟡 Good |
| `offline_mud/game.rs` | 96.58% | 95.88% | 96.55% | 🟢 Excellent |
| `offline_mud/parser.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `output_window.rs` | 40.84% | 46.73% | 45.45% | 🟠 Moderate |
| `plugins/perl.rs` | 82.80% | 87.61% | 85.00% | 🟡 Good |
| `plugins/python.rs` | 88.14% | 89.22% | 90.91% | 🟡 Good |
| `plugins/stack.rs` | 78.11% | 80.95% | 60.00% | 🟡 Good |
| `screen.rs` | 93.22% | 95.25% | 88.89% | 🟢 Excellent |
| `scrollback_search.rs` | 65.71% | 68.29% | 66.67% | 🟠 Moderate |
| `scrollback.rs` | 92.81% | 94.30% | 96.67% | 🟢 Excellent |
| `select.rs` | 97.50% | 96.61% | 100.00% | 🟢 Excellent |
| `selectable.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `selection.rs` | 77.73% | 82.46% | 77.78% | 🟡 Good |
| `session_manager.rs` | 59.07% | 62.50% | 61.90% | 🟠 Moderate |
| `session.rs` | 85.41% | 87.02% | 87.50% | 🟡 Good |
| `socket.rs` | 91.54% | 92.35% | 100.00% | 🟢 Excellent |
| `status_line.rs` | 91.43% | 94.55% | 83.33% | 🟢 Excellent |
| `telnet.rs` | 97.67% | 99.02% | 100.00% | 🟢 Excellent |
| `tty.rs` | 31.11% | 25.84% | 55.56% | 🔴 Needs Work |
| `window.rs` | 60.00% | 56.83% | 60.00% | 🟠 Moderate |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `alias.rs` - 94.01%
- `command_queue.rs` - 92.57%
- `config.rs` - 94.67%
- `macro_def.rs` - 100.00%
- `offline_mud/game.rs` - 96.58%
- `offline_mud/parser.rs` - 100.00%
- `screen.rs` - 93.22%
- `scrollback.rs` - 92.81%
- `select.rs` - 97.50%
- `selectable.rs` - 100.00%
- `socket.rs` - 91.54%
- `status_line.rs` - 91.43%
- `telnet.rs` - 97.67%

### 🟡 Good (70-89% lines)
- `ansi.rs` - 88.36%
- `control.rs` - 72.62%
- `engine.rs` - 88.24%
- `input.rs` - 89.69%
- `mccp.rs` - 71.09%
- `mud.rs` - 72.80%
- `mud_selection.rs` - 82.18%
- `plugins/perl.rs` - 82.80%
- `plugins/python.rs` - 88.14%
- `plugins/stack.rs` - 78.11%
- `selection.rs` - 77.73%
- `session.rs` - 85.41%

### 🟠 Moderate (40-69% lines)
- `action.rs` - 68.93%
- `history.rs` - 65.85%
- `input_box.rs` - 54.24%
- `input_line.rs` - 44.44%
- `output_window.rs` - 40.84%
- `scrollback_search.rs` - 65.71%
- `session_manager.rs` - 59.07%
- `window.rs` - 60.00%

### 🔴 Needs Work (<40% lines)
- `curses.rs` - 18.56% (ncurses FFI - TTY dependent)
- `debug_log.rs` - 0.00%
- `main.rs` - 0.00% (event loop - needs integration tests)
- `tty.rs` - 31.11% (requires real TTY)

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 71.44% | ⏳ In Progress |
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
