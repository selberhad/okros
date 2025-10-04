# Scripts Directory

This directory contains automation scripts for the okros project.

## LOC Comparison Scripts

### `generate_loc_report.pl` - Automated Report Generation

**Purpose**: Auto-generates `LOC_COMPARISON.md` for version control

**Usage**:
```bash
./scripts/generate_loc_report.pl
```

**Features**:
- Generates markdown file with LOC statistics
- Includes file-by-file comparison table
- Flags suspiciously short files (<40% of C++ size)
- **Auto-run by pre-commit hook** when source files change

**Output**: `LOC_COMPARISON.md` (committed to repo)

### `compare_loc.pl` - Interactive Console Report

**Purpose**: Interactive LOC analysis with detailed output

**Usage**:
```bash
./scripts/compare_loc.pl
```

**Features**:
- Console-based detailed comparison
- Full statistics breakdown by language
- File-by-file analysis with warnings
- Human-readable interactive output

**Output**: Console only (not committed)

## Coverage Report

### `generate-coverage-report.sh`

**Purpose**: Auto-generates `COVERAGE_REPORT.md` with test coverage data

**Usage**:
```bash
./scripts/generate-coverage-report.sh
```

**Features**:
- Runs `cargo tarpaulin` for coverage metrics
- Generates markdown report with per-file breakdown
- **Auto-run by pre-commit hook** when source files change

**Output**: `COVERAGE_REPORT.md` (committed to repo)

## MUD Helper Scripts

### `mud_cmd.sh`

**Purpose**: Send commands to headless okros instance via control socket

**Usage**:
```bash
./scripts/mud_cmd.sh /tmp/okros/instance.sock "command"
```

**Example**:
```bash
./scripts/mud_cmd.sh /tmp/okros/nodeka.sock "look"
```

## Pre-Commit Hook

The `.git/hooks/pre-commit` hook automatically runs:

1. **rustfmt** - Format all staged Rust files
2. **generate-coverage-report.sh** - Update coverage report if source files changed
3. **generate_loc_report.pl** - Update LOC comparison if source files changed

Both generated reports are automatically staged for commit.

## Dependencies

- **cloc** - Count Lines of Code tool
  ```bash
  brew install cloc
  ```

- **jq** - JSON processor (for mud_cmd.sh)
  ```bash
  brew install jq
  ```

- **cargo-tarpaulin** - Coverage tool
  ```bash
  cargo install cargo-tarpaulin
  ```

## Development Workflow

When committing code changes:

1. Stage your changes: `git add <files>`
2. Commit: `git commit -m "message"`
3. Pre-commit hook runs automatically:
   - Formats Rust code
   - Updates `COVERAGE_REPORT.md` (if needed)
   - Updates `LOC_COMPARISON.md` (if needed)
   - Stages updated reports
4. Commit proceeds with updated reports included

No manual action needed! Reports stay in sync automatically.
