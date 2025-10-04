# Test Coverage Report

**Last Updated**: 2025-10-03 22:19
**Tool**: cargo-llvm-cov
**Overall Coverage**: **76.90%** lines | **79.78%** regions | **82.76%** functions

## Summary

```
TOTAL                            7474              1511    79.78%         435                75    82.76%        4515              1043    76.90%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `action.rs` | 68.93% | 68.05% | 71.43% | 🔴 Needs Work |
| `alias.rs` | 94.01% | 94.52% | 100.00% | 🔴 Needs Work |
| `ansi.rs` | 88.65% | 90.45% | 100.00% | 🔴 Needs Work |
| `config.rs` | 94.67% | 96.30% | 77.42% | 🔴 Needs Work |
| `control.rs` | 72.75% | 74.25% | 87.50% | 🔴 Needs Work |
| `curses.rs` | 85.57% | 82.58% | 80.00% | 🔴 Needs Work |
| `engine.rs` | 97.06% | 97.00% | 100.00% | 🔴 Needs Work |
| `input_line.rs` | 83.58% | 84.21% | 75.00% | 🔴 Needs Work |
| `input.rs` | 89.69% | 87.03% | 100.00% | 🔴 Needs Work |
| `macro_def.rs` | 100.00% | 100.00% | 100.00% | 🔴 Needs Work |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `mccp.rs` | 71.09% | 74.85% | 73.33% | 🔴 Needs Work |
| `mud_selection.rs` | 95.06% | 96.67% | 100.00% | 🔴 Needs Work |
| `mud.rs` | 88.21% | 90.65% | 85.19% | 🔴 Needs Work |
| `offline_mud/game.rs` | 96.92% | 96.34% | 96.55% | 🔴 Needs Work |
| `offline_mud/parser.rs` | 100.00% | 100.00% | 100.00% | 🔴 Needs Work |
| `output_window.rs` | 100.00% | 100.00% | 100.00% | 🔴 Needs Work |
| `plugins/stack.rs` | 73.13% | 78.10% | 55.00% | 🔴 Needs Work |
| `screen.rs` | 95.78% | 97.79% | 96.15% | 🔴 Needs Work |
| `scrollback.rs` | 91.71% | 93.13% | 100.00% | 🔴 Needs Work |
| `select.rs` | 97.50% | 96.61% | 100.00% | 🔴 Needs Work |
| `selectable.rs` | 100.00% | 100.00% | 100.00% | 🔴 Needs Work |
| `selection.rs` | 71.14% | 76.47% | 75.00% | 🔴 Needs Work |
| `session.rs` | 85.92% | 88.62% | 87.50% | 🔴 Needs Work |
| `socket.rs` | 91.54% | 92.35% | 100.00% | 🔴 Needs Work |
| `status_line.rs` | 100.00% | 100.00% | 100.00% | 🔴 Needs Work |
| `telnet.rs` | 97.67% | 99.02% | 100.00% | 🔴 Needs Work |
| `tty.rs` | 77.78% | 83.15% | 100.00% | 🔴 Needs Work |
| `window.rs` | 88.24% | 84.51% | 85.71% | 🔴 Needs Work |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)

### 🟡 Good (70-89% lines)

### 🟠 Moderate (40-69% lines)

### 🔴 Needs Work (<40% lines)

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 76.90% | ⏳ In Progress |
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
