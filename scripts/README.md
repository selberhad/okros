# Development Scripts

This directory contains automation scripts for the project.

## Coverage Reporting

### `generate-coverage-report.sh`

Generates `COVERAGE_REPORT.md` from `cargo llvm-cov` output in a git-diff-friendly format.

**Features**:
- Parses llvm-cov summary data
- Creates markdown tables sorted alphabetically by module
- Color-coded status indicators (🟢🟡🟠🔴)
- Stable formatting for clean git diffs
- Auto-categorizes modules by coverage tier

**Usage**:
```bash
# Direct invocation
./scripts/generate-coverage-report.sh

# Via Make/Just
make coverage-report
just coverage-report
```

**Output**: Updates `COVERAGE_REPORT.md` at project root

**Format**:
- Header with timestamp and overall coverage percentages
- Table of all modules with line/region/function coverage
- Categorized lists by coverage tier (Excellent/Good/Moderate/Needs Work)
- Coverage targets and status

## Git Hooks

### `install-git-hooks.sh`

Installs or uninstalls git hooks for automatic coverage tracking.

**Usage**:
```bash
# Install hooks
./scripts/install-git-hooks.sh install
make install-hooks
just install-hooks

# Uninstall hooks
./scripts/install-git-hooks.sh uninstall
make uninstall-hooks
just uninstall-hooks
```

**What it installs**:
- Pre-push hook (`pre-push-coverage`) to `.git/hooks/pre-push`

**Safety**:
- Prompts before overwriting existing hooks
- Provides manual installation instructions if user declines

### `pre-push-coverage`

Git pre-push hook that ensures coverage reports stay up-to-date.

**Workflow**:
1. Runs `generate-coverage-report.sh`
2. Checks if `COVERAGE_REPORT.md` was modified
3. If modified:
   - Shows what changed (coverage % diff)
   - Blocks the push
   - Prompts user to commit updated report
4. If unchanged:
   - Allows push to proceed

**Bypass**:
```bash
# Skip hook for emergency pushes
git push --no-verify
```

**Example Output**:
```
🔬 Running pre-push coverage check...
Generating coverage data...
✅ Coverage report generated: COVERAGE_REPORT.md

📊 Coverage report has been updated!

Changes detected in COVERAGE_REPORT.md

Please review and commit the updated coverage report:
  git add COVERAGE_REPORT.md
  git commit --amend --no-edit

Or to skip this hook: git push --no-verify

Coverage changes:
-**Overall Coverage**: **62.61%** lines
+**Overall Coverage**: **65.23%** lines
```

## Why Bash (Not Perl)?

**Rationale for using Bash**:
1. ✅ **Universal availability** - Guaranteed on any Unix system (macOS, Linux, BSD)
2. ✅ **Git hook convention** - 99% of git hooks are bash
3. ✅ **Simple operations** - Just running cargo commands and checking files
4. ✅ **Zero dependencies** - Works out of the box
5. ✅ **CI/CD compatibility** - Runs in all common CI environments

**When to use Perl instead**:
- Complex text parsing (awk handles our simple parsing fine)
- MUD bot logic (where Perl shines!)
- Script interpreters for game automation
- Protocol parsing with advanced regex

**Philosophy**: Use the right tool for the job. Bash for simple automation, Perl for MUD magic! 🐪

## Coverage Report Format

### Git-Diff Friendliness

The generated report is optimized for readable git diffs:

1. **Stable Sorting**: Modules alphabetically sorted (same order every time)
2. **Consistent Formatting**: Fixed table structure with aligned columns
3. **Semantic Changes Only**: Timestamp and percentages are the only changing parts
4. **No Noise**: No random ordering, no unstable data

**Example Git Diff**:
```diff
 # Test Coverage Report

-**Last Updated**: 2025-10-03 01:15
+**Last Updated**: 2025-10-03 01:20
-**Overall Coverage**: **62.61%** lines
+**Overall Coverage**: **65.23%** lines

 | Module | Line Coverage | Region Coverage | Functions | Status |
 |--------|--------------|-----------------|-----------|--------|
 | `ansi.rs` | 95.65% | 90.32% | 100.00% | 🟢 Excellent |
-| `control.rs` | 40.97% | 35.33% | 46.67% | 🟠 Moderate |
+| `control.rs` | 52.34% | 48.21% | 53.33% | 🟠 Moderate |
```

Clean, semantic, easy to review! ✨

## Integration with Development Workflow

### Recommended Setup (One-Time)

```bash
# 1. Install coverage tool
cargo install cargo-llvm-cov

# 2. Install git hooks
make install-hooks

# 3. Done! Coverage stays current automatically
```

### Daily Workflow

```bash
# Write code and tests...

# Run tests
make test

# Check coverage (HTML)
make coverage         # Opens in browser

# Push changes (hook runs automatically)
git add .
git commit -m "feat: add awesome feature"
git push              # Hook checks coverage, may ask for report update
```

### Before PR

```bash
# Update coverage report
make coverage-report

# Review changes
git diff COVERAGE_REPORT.md

# Commit if changed
git add COVERAGE_REPORT.md
git commit -m "chore: update coverage report"
```

## Maintenance

### Updating Scripts

After modifying scripts:

```bash
# Make sure they're executable
chmod +x scripts/*.sh

# Test coverage report generation
./scripts/generate-coverage-report.sh

# Test hook installation (safe, prompts before overwrite)
./scripts/install-git-hooks.sh install
```

### Troubleshooting

**Coverage report shows 0% for everything**:
- Run `cargo llvm-cov clean` then try again
- Ensure tests are passing: `cargo test`

**Git hook not triggering**:
- Check it's installed: `ls -la .git/hooks/pre-push`
- Verify it's executable: `chmod +x .git/hooks/pre-push`
- Test manually: `.git/hooks/pre-push`

**Coverage percentages look wrong**:
- The script parses llvm-cov output at specific field positions
- If llvm-cov output format changes, update field numbers in `generate-coverage-report.sh`:
  - Field 4: Region coverage %
  - Field 7: Function coverage %
  - Field 10: Line coverage %

## Files

| File | Purpose | Executable |
|------|---------|-----------|
| `generate-coverage-report.sh` | Generate COVERAGE_REPORT.md | ✅ |
| `install-git-hooks.sh` | Install/uninstall git hooks | ✅ |
| `pre-push-coverage` | Git pre-push hook | ✅ |
| `README.md` | This file | ❌ |

## See Also

- [DEVELOPMENT.md](../DEVELOPMENT.md) - Full development guide
- [TESTING.md](../TESTING.md) - Testing guide and coverage docs
- [COVERAGE_REPORT.md](../COVERAGE_REPORT.md) - Current coverage report
