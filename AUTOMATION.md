# Automation System

okros supports three types of automation: **aliases**, **triggers/actions**, and **macros**.

## Quick Reference

| Feature | Command | Example | When It Runs |
|---------|---------|---------|--------------|
| **Alias** | `#alias <name> <text>` | `#alias n go north` | Input (before sending to MUD) |
| **Trigger** | `#action "pattern" commands` | `#action "^You are attacked" flee` | Output (on MUD text) |
| **Substitution** | `#subst "pattern" replacement` | `#subst "stupid" smart` | Output (text replacement) |
| **Macro** | `#macro <key> <text>` | `#macro n north` | Keypress (ASCII only) |

## Aliases

Text expansion with parameter substitution.

### Syntax

```
#alias <name> <expansion>
```

### Parameters

- `%0` - All arguments
- `%1`, `%2`, `%3`, ... - Individual tokens (space-separated)
- `%-N` - Tokens 1 through N (range from start)
- `%+N` - Tokens N through end (range to end)
- `%%` - Literal `%`

### Examples

```bash
# Simple alias
#alias n go north

# With parameters
#alias say tell bob %1
# Usage: say hello → sends "tell bob hello"

# Multiple parameters
#alias tell whisper %1 %+2
# Usage: tell alice hello there → sends "whisper alice hello there"

# Range from start
#alias shout yell %-2
# Usage: shout hello world foo → sends "yell hello world"

# All arguments
#alias echo say %0
# Usage: echo hello world → sends "say hello world"
```

### Implementation

- **Location**: `src/alias.rs`
- **Storage**: `Mud.alias_list` (Vec<Alias>)
- **Execution**: Input pipeline (main.rs:276-306)
- **Lookup**: First word of input → find alias → expand with args

## Triggers & Actions

Pattern-based automation using Perl or Python regex.

### Trigger (Action)

Executes commands when pattern matches MUD output.

```bash
#action "^You are attacked" flee
#action "^.* says 'hello'" say Hi there!
```

**Regex Engine**: Perl (if `--features perl`) or Python (if `--features python`)

### Substitution

Replaces text in MUD output.

```bash
#subst "stupid" smart
#subst "foo" bar
```

### Gag

Suppresses lines matching pattern (future - not yet implemented).

```bash
#action "spam message" ""  # Empty commands = gag
```

### Implementation

- **Location**: `src/action.rs`
- **Storage**: `Mud.action_list` (Vec<Action>)
- **Compilation**: When added via `#action` (compiles regex with interpreter)
- **Execution**: Output pipeline (main.rs:366-407)
- **Regex**:
  - Perl: `match_prepare()` creates Perl sub with pattern
  - Python: `re.compile()` via pyo3

### How It Works

1. **Add Trigger**: `#action "pattern" commands`
2. **Compile**: Action.compile() → interpreter.match_prepare() → store compiled regex
3. **MUD Output**: Line arrives → session.feed()
4. **Check**: For each trigger → action.check_match(line, interpreter)
5. **Execute**: If match → send commands to MUD
6. **Feedback**: Status line shows "Trigger fired: pattern"

## Macros

Keyboard shortcuts (currently ASCII only).

```bash
#macro n north
#macro s south
```

**Note**: F1, F2, etc. not yet implemented (need key_lookup() from C++ MCL).

### Implementation

- **Location**: `src/macro_def.rs`
- **Storage**: `Mud.macro_list` (Vec<Macro>)
- **Execution**: Key press event (not yet wired - TODO)

## Requirements

### For Aliases & Macros

- **No dependencies** - Pure Rust, works without plugins

### For Triggers/Actions

- **Requires Perl OR Python plugin**:
  ```bash
  cargo build --features perl    # Perl regex
  cargo build --features python  # Python regex
  ```

- Without plugins, triggers can be added but won't execute (no regex engine)

## File Locations

| Component | File | Purpose |
|-----------|------|---------|
| Alias expansion | `src/alias.rs` | Parameter substitution (%N) |
| Action/trigger | `src/action.rs` | Pattern matching, regex |
| Macro bindings | `src/macro_def.rs` | Keyboard shortcuts |
| MUD storage | `src/mud.rs` | Lists and lookup methods |
| Input pipeline | `src/main.rs:276-306` | Alias expansion |
| Output pipeline | `src/main.rs:366-407` | Trigger checking |
| Perl regex | `src/plugins/perl.rs` | match_prepare/exec |
| Python regex | `src/plugins/python.rs` | re.compile() via pyo3 |

## Testing

```bash
# Unit tests (138 total)
cargo test

# With plugins
cargo test --features perl
cargo test --features python

# Coverage
./scripts/generate-coverage-report.sh
# alias.rs: 93.98% ✅
# action.rs: 65.96%
# mud.rs: 85.29%
```

## Examples

### Complex Alias

```bash
# Create a "say to" alias
#alias sayto tell %1 %+2

# Usage
sayto bob hello there friend
# → Expands to: tell bob hello there friend
```

### Multi-Step Trigger

```bash
# Flee on low health (requires Perl/Python)
#action "^Health: ([0-9]+)/([0-9]+)" flee

# Auto-loot after kill
#action "^.* is dead" get all from corpse
```

### Substitution Chain

```bash
# Clean up output
#subst "says, 'hello'" says hello
#subst "stupid" smart
```

## Future Enhancements

- [ ] Macro key codes (F1-F12, etc.) - need key_lookup()
- [ ] Hierarchical lookup (global → session-specific)
- [ ] Action priorities / ordering
- [ ] Gag implementation (suppress matched lines)
- [ ] Action enable/disable commands
- [ ] Persistence (save/load from config file)

## Architecture Notes

### C++ Reference Mapping

okros automation is ported from C++ MCL:

| C++ File | Rust Module | Status |
|----------|-------------|--------|
| `Alias.cc` | `src/alias.rs` | ✅ Complete |
| `Action.h` | `src/action.rs` | ✅ Complete |
| `MUD.cc` (lists) | `src/mud.rs` | ✅ Complete |
| `Session.cc` (triggerCheck) | `src/main.rs` | ✅ Wired |
| `PerlEmbeddedInterpreter.cc` | `src/plugins/perl.rs` | ✅ Complete |
| `PythonEmbeddedInterpreter.cc` | `src/plugins/python.rs` | ✅ Complete |

### Design Principles

1. **Simplicity first** - Rust idioms where simpler (Vec, String)
2. **Behavioral equivalence** - Match C++ behavior exactly
3. **Feature gates** - Perl/Python optional (no runtime bloat)
4. **No global state** - All state in Mud struct

### Performance

- Alias expansion: O(n) where n = number of aliases
- Trigger checking: O(m) where m = number of triggers
- Regex compilation: Once per trigger (cached in Action)
- Execution overhead: Minimal (compiled regex)

## See Also

- `MUD_LEARNINGS.md` - Real-world MUD testing (Nodeka)
- `PORTING_HISTORY.md` - C++ → Rust porting notes
- `tests/action_trigger_tests.rs` - Golden tests from C++ reference
