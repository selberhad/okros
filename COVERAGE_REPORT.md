# Test Coverage Report

**Last Updated**: 2025-10-03 08:22
**Tool**: cargo-llvm-cov
**Overall Coverage**: **64.38%** lines | **69.27%** regions | **77.26%** functions

## Summary

```
TOTAL                            5675              1744    69.27%         343                78    77.26%        2350               837    64.38%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `action.rs` | 65.96% | 66.67% | 69.23% | 🟠 Moderate |
| `alias.rs` | 93.98% | 94.52% | 100.00% | 🟢 Excellent |
| `ansi.rs` | 94.37% | 90.45% | 100.00% | 🟢 Excellent |
| `config.rs` | 90.00% | 87.50% | 71.43% | 🟢 Excellent |
| `control.rs` | 39.10% | 33.43% | 46.67% | 🔴 Needs Work |
| `curses.rs` | 18.56% | 13.55% | 40.00% | 🔴 Needs Work |
| `engine.rs` | 65.04% | 63.67% | 87.50% | 🟠 Moderate |
| `input_line.rs` | 88.46% | 84.21% | 75.00% | 🟡 Good |
| `input.rs` | 92.50% | 87.03% | 100.00% | 🟢 Excellent |
| `macro_def.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `mccp.rs` | 83.33% | 74.85% | 73.33% | 🟡 Good |
| `mud.rs` | 85.29% | 83.93% | 71.43% | 🟡 Good |
| `offline_mud/game.rs` | 96.61% | 95.88% | 96.55% | 🟢 Excellent |
| `offline_mud/parser.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `output_window.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `plugins/stack.rs` | 78.26% | 78.10% | 55.00% | 🟡 Good |
| `screen.rs` | 98.84% | 97.79% | 96.15% | 🟢 Excellent |
| `scrollback.rs` | 96.15% | 93.13% | 100.00% | 🟢 Excellent |
| `select.rs` | 100.00% | 96.61% | 100.00% | 🟢 Excellent |
| `selectable.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `session.rs` | 89.47% | 88.62% | 87.50% | 🟡 Good |
| `socket.rs` | 92.86% | 92.35% | 100.00% | 🟢 Excellent |
| `status_line.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `telnet.rs` | 100.00% | 99.02% | 100.00% | 🟢 Excellent |
| `tty.rs` | 34.15% | 25.84% | 55.56% | 🔴 Needs Work |
| `window.rs` | 89.19% | 84.51% | 85.71% | 🟡 Good |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `alias.rs` - 93.98%
- `ansi.rs` - 94.37%
- `config.rs` - 90.00%
- `input.rs` - 92.50%
- `macro_def.rs` - 100.00%
- `offline_mud/game.rs` - 96.61%
- `offline_mud/parser.rs` - 100.00%
- `output_window.rs` - 100.00%
- `screen.rs` - 98.84%
- `scrollback.rs` - 96.15%
- `select.rs` - 100.00%
- `selectable.rs` - 100.00%
- `socket.rs` - 92.86%
- `status_line.rs` - 100.00%
- `telnet.rs` - 100.00%

### 🟡 Good (70-89% lines)
- `input_line.rs` - 88.46%
- `mccp.rs` - 83.33%
- `mud.rs` - 85.29%
- `plugins/stack.rs` - 78.26%
- `session.rs` - 89.47%
- `window.rs` - 89.19%

### 🟠 Moderate (40-69% lines)
- `action.rs` - 65.96%
- `engine.rs` - 65.04%

### 🔴 Needs Work (<40% lines)
- `control.rs` - 39.10%
- `curses.rs` - 18.56% (ncurses FFI - TTY dependent)
- `main.rs` - 0.00% (event loop - needs integration tests)
- `tty.rs` - 34.15% (requires real TTY)

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 64.38% | ⏳ In Progress |
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
