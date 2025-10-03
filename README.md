# okros - Rust MUD Client

**okros** (from _ochre_, rusty mud) is a modern MUD client written in Rust, reviving the design principles of MCL (MUD Client for Linux). Built for headless/detachable operation, it's perfect for automation, LLM agents, and cloud deployments.

> **Current Status**: ~95% complete - all core features implemented, validation pending with real MUD servers. See [ORIENTATION.md](ORIENTATION.md) for detailed status.

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

### CLI Reference

```bash
# Interactive mode (default)
okros [mudname]                     # Connect to saved MUD profile
okros                               # Start without connection (use #open command)

# Offline demo mode
okros --offline                     # Play internal MUD (no network required)

# Headless mode (background daemon)
okros --headless --instance NAME              # Start headless session (network)
okros --headless --offline --instance NAME    # Headless offline MUD (for testing/LLM agents)
okros --attach NAME                           # Attach to running session

# Environment variables
MCL_CONNECT=127.0.0.1:4000 okros   # Auto-connect on startup
```

### Interactive Mode

Connect to a MUD server interactively:

```bash
# Auto-connect via environment variable
MCL_CONNECT=example.com:4000 okros

# Start client, then connect manually
okros
#open 127.0.0.1 4000
```

**Key bindings:**
- `PageUp/PageDown` - Scroll through history
- `Arrow Up` - Command history
- `Alt-Q` - Quit (not yet implemented)
- `Ctrl-C` - Cancel current line
- `Home/End` - Navigate scrollback buffer

**Internal commands:**
- `#open <host> <port>` - Connect to MUD server (IPv4 only currently)
- `#quit` - Exit client

### Offline Mode (Internal MUD)

Play a built-in text adventure for testing or offline demo:

```bash
okros --offline
```

**Features:**
- 5 interconnected rooms (forest, clearing, cave, stream, village)
- 3 collectible items (rusty sword, torch, iron key)
- Full ANSI color output
- No network connection required
- Perfect for testing the UI without a MUD server

**Commands:** `go <direction>`, `look`, `take <item>`, `drop <item>`, `inventory`, `help`, `quit`
**Direction aliases:** `n`, `s`, `e`, `w`, `u`, `d`

### Headless Mode

Run a MUD session as a background daemon:

```bash
# Start headless session (network mode)
okros --headless --instance ar example.com 4000

# Start headless offline MUD (perfect for LLM testing)
okros --headless --offline --instance demo

# Attach to running session
okros --attach ar

# Send commands to session
echo '{"cmd":"send","data":"look\n"}' | nc -U /tmp/okros/ar.sock

# Get buffered output
echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/ar.sock

# Check game status (offline mode only)
echo '{"cmd":"status"}' | nc -U /tmp/okros/demo.sock
# Returns: {"event":"Status","inventory_count":0,"location":"clearing"}

# Stream live output
echo '{"cmd":"stream"}' | nc -U /tmp/okros/ar.sock
```

**Helper scripts** for session management and output access:
```bash
# Session Management
./scripts/start_headless.sh nodeka nodeka.com:23  # Start/restart session
./scripts/stop_headless.sh nodeka                 # Gracefully stop session
./scripts/list_sessions.sh                        # Show all running sessions
./scripts/session_status.sh nodeka                # Detailed session info

# Output/Buffer Access
./scripts/mud_cmd.sh /tmp/okros/nodeka.sock "look"  # Send command + show response
./scripts/get_buffer.sh nodeka                      # Get full scrollback buffer
./scripts/recent_lines.sh nodeka 20                 # Get last N lines
./scripts/watch_output.sh nodeka                    # Continuously watch output
```

**Test credentials** are stored in `.env` (gitignored) - see `.env.example` for format.

**Headless Offline Mode** is perfect for LLM agent development and testing - no network required, deterministic behavior, and full JSON control protocol.

### LLM Agent Integration

**See [AGENT_GUIDE.md](AGENT_GUIDE.md) for complete LLM agent documentation** including:
- Control protocol reference
- Helper scripts (session management, buffer access, command sending)
- Test credentials (`.env`)
- Critical rules (no command spamming, never guess MUD commands)
- Common pitfalls and troubleshooting

okros headless mode is designed for simplicity - LLM agents just need to read text buffers and send commands:

```python
import socket
import json

# Start headless offline MUD for testing (no real MUD server needed)
# $ okros --headless --offline --instance demo

# Connect to headless okros instance
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect("/tmp/okros/demo.sock")

# Read MUD output
sock.sendall(json.dumps({"cmd": "get_buffer"}).encode() + b'\n')
response = json.loads(sock.recv(4096))
# Parse buffer response (format: {"event":"Buffer","lines":[...]})
mud_text = "\n".join(response.get("lines", []))

# LLM processes mud_text and decides next action...
# (Your LLM logic here)

# Send command back to MUD
action = "take rusty sword\n"  # LLM's decision
sock.sendall(json.dumps({"cmd": "send", "data": action}).encode() + b'\n')
sock.recv(4096)  # Read OK response

# Check game state (offline mode provides structured status)
sock.sendall(json.dumps({"cmd": "status"}).encode() + b'\n')
status = json.loads(sock.recv(4096))
# Returns: {"event":"Status","inventory_count":1,"location":"clearing"}

sock.close()
```

**Philosophy**: No structured events, no complex parsing - just raw MUD text. LLMs already understand natural language; let them do what they do best.

**Tip**: Use `--headless --offline` for LLM agent development - provides a deterministic test environment with the `status` command for validation.

### Control Server Protocol

The control server uses JSON Lines (one JSON object per line):

**Commands:**
```javascript
{"cmd":"status"}                               // Get session/game status
{"cmd":"attach"}                               // Attach to session
{"cmd":"detach"}                               // Detach from session
{"cmd":"send","data":"north\n"}                // Send command to MUD
{"cmd":"get_buffer"}                           // Get buffered output (consumes new lines)
{"cmd":"peek","lines":20}                      // Peek at recent lines without consuming
{"cmd":"hex","lines":10}                       // Debug view (hex + color codes)
{"cmd":"stream","interval_ms":200}             // Stream live output
{"cmd":"sock_send","data":"raw telnet bytes"}  // Send raw bytes (network mode)
{"cmd":"connect","data":"host:port"}           // Connect to MUD (network mode)
```

**Responses:**
```javascript
{"event":"Ok"}
{"event":"Status","attached":true}                              // Network mode
{"event":"Status","location":"cave","inventory_count":2}        // Offline mode
{"event":"Buffer","lines":["You are standing in a room.","Exits: north, south"]}
{"event":"Hex","lines":[{"hex":"48:07 65:07","text":"He","colors":"07 07"}]}  // Debug mode
{"event":"Error","message":"not connected"}
```

### Configuration

Create `~/.okros/config`:

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

**Implementation**: ~95% complete (all tiers done)
**Testing**: 82 tests passing | 65% coverage
**Validation**: ✅ Tested with Nodeka MUD (nodeka.com:23)

Recent work:
- ✅ Per-character color storage (fixed black-on-black menus)
- ✅ Circular buffer flattening (headless mode)
- ✅ Hex dump debug tool (`hex` command)
- ✅ Login flow with real MUD

See [ORIENTATION.md](ORIENTATION.md) for current status, [PORTING_HISTORY.md](PORTING_HISTORY.md) for implementation history, [MUD_LEARNINGS.md](MUD_LEARNINGS.md) for debugging findings, and [FUTURE_WORK.md](FUTURE_WORK.md) for remaining tasks.

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

- **[AGENT_GUIDE.md](AGENT_GUIDE.md)** - LLM agent integration guide (control protocol, helper scripts, best practices)
- **[MUD_LEARNINGS.md](MUD_LEARNINGS.md)** - Technical findings from Nodeka integration (debugging methodology, test cases)
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
