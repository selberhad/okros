# okros - Rust MUD Client

**okros** (from _ochre_, rusty mud) is a modern MUD client written in Rust, reviving the design principles of MCL (MUD Client for Linux). MCL was a beloved Linux MUD client that went unmaintained circa 2000 and offline by 2010. okros resurrects its core concepts while bringing them into the modern era with headless/detachable operation, perfect for automation, LLM agents, and cloud deployments.

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
```

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
echo '{"command":"send","data":"look\n"}' | nc -U /tmp/okros-ar.sock

# Get buffered output
echo '{"command":"get_buffer"}' | nc -U /tmp/okros-ar.sock

# Stream live output
echo '{"command":"stream"}' | nc -U /tmp/okros-ar.sock
```

### Control Server Protocol

The control server uses JSON Lines (one JSON object per line):

**Commands:**
```json
{"command":"status"}
{"command":"attach"}
{"command":"detach"}
{"command":"send","data":"north\n"}
{"command":"get_buffer","from":0}
{"command":"stream"}
```

**Responses:**
```json
{"status":"ok","connected":true,"mud":"example.com:4000"}
{"output":"You are standing in a room.\n","cursor":123}
{"error":"not connected"}
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
- ⚠️  Tier 4 (Logic) - Partial (session management done; aliases/actions pending)
- ✅ Tier 5 (Plugins) - Complete (Python & Perl)
- ✅ Tier 6 (Engine) - Complete (headless & control server)

See [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) for detailed progress.

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

## Contributing

Contributions welcome! Key areas:

- Porting remaining C++ modules (aliases, actions, macros)
- Testing headless mode with LLM agents
- Documentation and examples
- Cross-platform support (Windows, macOS)

See [CLAUDE.md](CLAUDE.md) for development guidelines.

## Acknowledgments

- **Erwin S. Andreasen** - Original MCL author and visionary MUD client designer
- **MCL Community** - Players and scripters who made MCL legendary in its heyday
- **Rust Community** - For excellent FFI and async tooling
- **pyo3** - Python integration without the C API pain
- **Discovery Phase** - Validated via 11 toy implementations in [toys/](toys/)

---

*okros: Like mud, but rustier.*
