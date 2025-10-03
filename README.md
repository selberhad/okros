# okros - Rust MUD Client

> **📚 Doc-Driven Development (DDD) in Action**
> This project exemplifies [DDD Porting Mode](DDD.md): a reference-driven translation workflow combining Discovery (validate risky patterns via toy models) + Execution (systematic translation). Evidence: [12 toys validating FFI/unsafe patterns](toys/) (including [internal MUD](toys/toy12_internal_mud/) for e2e testing), [historical porting record](PORTING_HISTORY.md), [architectural documentation](CODE_MAP.md), and [95% completion with comprehensive docs](ORIENTATION.md). See [DDD.md](DDD.md) for the full methodology.

**okros** (from _ochre_, rusty mud) is a modern MUD client written in Rust, reviving the design principles of MCL (MUD Client for Linux). MCL was a powerful, feature-rich Linux MUD client that went unmaintained circa 2000 and offline by 2010. okros resurrects its core concepts while bringing them into the modern era with headless/detachable operation, perfect for automation, LLM agents, and cloud deployments.

## Features

### Core Functionality

- **ANSI Color Support** - Full 16-color ANSI rendering with attributes (bold, etc.)
- **Telnet Protocol** - IAC command handling, GA/EOR prompt detection
- **MCCP Compression** - Built-in MCCP v1/v2 support (optional `mccp` feature)
- **Scrollback Buffer** - Configurable ring buffer for session history
- **Input Editing** - Full line editing with history, cursor movement, and search

### Headless & Detachable Mode (LLM-Friendly)

okros is designed for non-interactive use cases:

- **Headless Engine** - Run sessions without a TTY, perfect for automation
- **Control Server** - Unix domain socket with JSON Lines protocol
- **Attach/Detach** - Connect and disconnect from running sessions without data loss
- **Stream Output** - Subscribe to live output or retrieve buffered history
- **Send Commands** - Inject input into sessions remotely
- **Multiple Sessions** - Run multiple MUD connections in parallel

**Use cases:**
- LLM agents playing MUDs
- Cloud-hosted MUD bots
- CI/CD testing against MUD servers
- Screen/tmux-style session management
- Remote control via SSH

### Scripting (Optional)

- **Python** - Embedded Python interpreter via pyo3 (`--features python`)
- **Perl** - Embedded Perl interpreter via raw FFI (`--features perl`)
- **Stacked Interpreters** - Chain multiple script engines together

## Installation

### Prerequisites

- Rust 1.70+ (2021 edition)
- ncurses development headers
- (Optional) Python 3.10+ development headers for Python support
- (Optional) Perl 5.34+ with development headers for Perl support

### Building from Source

```bash
# Basic build (no scripting)
cargo build --release

# With Python support
cargo build --release --features python

# With Perl support
cargo build --release --features perl

# With all features
cargo build --release --all-features
```

The compiled binary will be at `target/release/okros`.

### Running Tests

```bash
# Core tests
cargo test

# With features
cargo test --all-features

# Using task runner (recommended)
make test              # Or: just test
make coverage          # Generate coverage report
```

See [TESTING.md](TESTING.md) for comprehensive testing guide.

## Usage

### Interactive Mode

```bash
# Connect to a MUD server
okros example.com 4000

# Connect with a saved MUD profile
okros mudname
```

**Key bindings:**
- `PageUp/PageDown` - Scroll through history
- `Arrow Up` - Command history
- `Alt-Q` - Quit
- `Ctrl-C` - Cancel current line
- `Home/End` - Navigate scrollback buffer

### Headless Mode

Run a MUD session as a background daemon:

```bash
# Start headless session named "ar"
okros --headless --instance ar example.com 4000

# Attach to running session
okros --attach ar

# Send commands to session
echo '{"cmd":"send","data":"look\n"}' | nc -U ~/.mcl/control/ar.sock

# Get buffered output
echo '{"cmd":"get_buffer"}' | nc -U ~/.mcl/control/ar.sock

# Stream live output
echo '{"cmd":"stream"}' | nc -U ~/.mcl/control/ar.sock
```

### LLM Agent Integration

okros headless mode is designed for simplicity - LLM agents just need to read text buffers and send commands:

```python
import socket
import json
import os

# Connect to headless okros instance
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect(os.path.expanduser("~/.mcl/control/ar.sock"))

# Read MUD output
sock.sendall(json.dumps({"cmd": "get_buffer"}).encode() + b'\n')
response = json.loads(sock.recv(4096))
# Parse buffer response (format: {"event":"Buffer","lines":[...]})
mud_text = "\n".join(response.get("lines", []))

# LLM processes mud_text and decides next action...
# (Your LLM logic here)

# Send command back to MUD
action = "north\n"  # LLM's decision
sock.sendall(json.dumps({"cmd": "send", "data": action}).encode() + b'\n')

sock.close()
```

**Philosophy**: No structured events, no complex parsing - just raw MUD text. LLMs already understand natural language; let them do what they do best.

### Control Server Protocol

The control server uses JSON Lines (one JSON object per line):

**Commands:**
```json
{"cmd":"status"}
{"cmd":"attach"}
{"cmd":"detach"}
{"cmd":"send","data":"north\n"}
{"cmd":"get_buffer","from":0}
{"cmd":"stream","interval_ms":200}
{"cmd":"sock_send","data":"raw telnet bytes"}
```

**Responses:**
```json
{"event":"Ok"}
{"event":"Status","attached":true}
{"event":"Buffer","lines":["You are standing in a room.","Exits: north, south"]}
{"event":"Error","message":"not connected"}
```

### Configuration

Create `~/.okros/config` (or `~/.mcl/mclrc` for MCL compatibility):

```
MUD example {
  Host example.com 4000
  Commands myusername;mypassword
  Alias go north;east;up
}
```

## Architecture

okros is a 1:1 Rust port of MCL using a "safety third" approach - liberal use of `unsafe` and FFI to match C++ behavior exactly. The codebase is organized into tiers:

- **Tier 1 (Foundation)** - Use Rust stdlib (`String`, `Vec`) where possible
- **Tier 2 (Core)** - Network, config, telnet, MCCP, TTY handling
- **Tier 3 (UI)** - ncurses wrapper, diff-based rendering, scrollback
- **Tier 4 (Logic)** - Session management, command processing, interpreters
- **Tier 5 (Plugins)** - Optional Python/Perl scripting engines
- **Tier 6 (Engine)** - Headless mode and control server

## Development Status

okros is under active development. Current status:

- ✅ Tier 1 (Foundation) - Complete
- ✅ Tier 2 (Core) - Complete
- ✅ Tier 3 (UI) - Complete
- ✅ Tier 4 (Logic) - Complete (session/engine; aliases/actions deferred to scripts)
- ✅ Tier 5 (Plugins) - Complete (Python & Perl)
- ✅ Tier 6 (Main & Engine) - Complete (event loop, CLI args, headless & control server)
- ⏸️  Tier 7 (Integration) - Validation pending (implementation complete, needs real MUD testing)

See [PORTING_HISTORY.md](PORTING_HISTORY.md) for detailed porting history and [FUTURE_WORK.md](FUTURE_WORK.md) for remaining tasks.

## Comparison with Original MCL

okros revives MCL's design philosophy while modernizing the implementation:

| Feature | MCL (C++) | okros (Rust) |
|---------|-----------|--------------|
| Platform | Linux (vcsa or TTY) | Any Unix (TTY only) |
| Build System | autoconf/make | Cargo |
| Dependencies | ncurses, libperl, libpython | ncurses, optional pyo3/perl |
| Scripting | Perl/Python | Perl/Python (feature-gated) |
| Virtual Console | Yes (Linux-specific) | No (portable) |
| Headless Mode | No | Yes (Unix socket API) |
| Memory Safety | Manual | Rust (with strategic unsafe) |

## License

okros is released under the GPL v2.

## Historical Note

okros is a from-scratch Rust implementation inspired by MCL (MUD Client for Linux), originally written by Erwin S. Andreasen. MCL was last maintained around 2000 and went offline circa 2010. This project revives MCL's design philosophy and feature set using a modern reference implementation discovered in the wild, bringing it back to life for contemporary use cases while preserving its spirit.

## Development

### Task Automation

okros provides multiple options for running development tasks (like `npm scripts` in Node):

**1. Makefile** (recommended, no extra install):
```bash
make help              # Show all commands
make test              # Run tests
make coverage          # Generate coverage report
make run-python        # Run with Python plugin
make pre-commit        # Format + lint + test
```

**2. Just** (modern alternative, install: `cargo install just`):
```bash
just                   # Show all commands
just test              # Run tests
just coverage          # Generate coverage report
```

**3. Cargo aliases** (in `.cargo/config.toml`):
```bash
cargo t                # test
cargo cov              # coverage
cargo bp               # build with Python
```

### Coverage Reports

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Generate HTML report
make coverage          # Generates and opens HTML report

# Update markdown report (git-friendly)
make coverage-report   # Updates COVERAGE_REPORT.md

# Auto-update on git push (recommended)
make install-hooks     # Installs pre-push hook
```

**Current Coverage**: 62.61% lines | See [COVERAGE_REPORT.md](COVERAGE_REPORT.md) for details

### Development Resources

- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Complete development guide (workflows, tools, CI/CD)
- **[TESTING.md](TESTING.md)** - Testing guide (running tests, coverage, CI)
- **[CLAUDE.md](CLAUDE.md)** - Project methodology and porting guidelines
- **[PORTING_HISTORY.md](PORTING_HISTORY.md)** - Historical record of C++ → Rust porting
- **[FUTURE_WORK.md](FUTURE_WORK.md)** - Remaining tasks and post-MVP enhancements

## Contributing

Contributions welcome! Key areas:

- Porting remaining C++ modules (aliases, actions, macros)
- Testing headless mode with LLM agents
- Documentation and examples
- Cross-platform support (Windows, macOS)

See [DEVELOPMENT.md](DEVELOPMENT.md) for setup and [CLAUDE.md](CLAUDE.md) for porting guidelines.

## Acknowledgments

- **Erwin S. Andreasen** - Original MCL author whose design still inspires 25 years later
- **Rust Community** - For excellent FFI and async tooling
- **pyo3** - Python integration without the C API pain
- **Discovery Phase** - Validated via 12 toy implementations in [toys/](toys/) (including built-in test MUD)

---

*okros: Like mud, but rustier.*
