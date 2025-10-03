# Agent Guide: Playing MUDs with okros

This guide explains how to connect AI agents, LLMs, or bots to MUD servers using okros headless mode.

## Quick Start

```bash
# 1. Start headless daemon
okros --headless --instance mybot
# Socket: /tmp/okros/mybot.sock

# 2. Connect to MUD
echo '{"cmd":"connect","data":"mud.example.com:4000"}' | nc -U /tmp/okros/mybot.sock

# 3. Read output
echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/mybot.sock

# 4. Send command (use helper script)
./scripts/mud_cmd.sh /tmp/okros/mybot.sock "look"
```

## Helper Scripts

### mud_cmd.sh - Send Command and Read Response

```bash
./scripts/mud_cmd.sh <socket> <command>
# Sends command, waits 2s, shows last 5 lines
```

### Test Credentials

Credentials stored in `.env` (gitignored):
```bash
NODEKA_USERNAME=YourCharacter
NODEKA_PASSWORD=YourSecurePassword
```

## Control Protocol

All commands are JSON Lines format.

### Commands

| Command | Parameters | Description |
|---------|-----------|-------------|
| `connect` | `data: "host:port"` | Connect to MUD server |
| `get_buffer` | (none) | Get current viewport buffer |
| `peek` | `lines: N` | Peek at last N lines without consuming |
| `sock_send` | `data: "text"` | Send raw text to MUD socket |
| `send` | `data: "text"` | Echo text locally (offline testing) |
| `hex` | `lines: N` | Debug view (hex + color codes) |
| `status` | (none) | Get connection status |
| `stream` | `interval_ms: 200` | Stream live output (blocking) |
| `quit` | (none) | Shutdown daemon |

### Responses

| Event | Fields | Description |
|-------|--------|-------------|
| `Ok` | (none) | Command succeeded |
| `Error` | `message` | Command failed |
| `Buffer` | `lines: []` | Viewport buffer contents |
| `Hex` | `lines: []` | Debug hex dump |
| `Status` | `attached: bool` | Connection status |

## Critical Rules

### ⚠️ NEVER Spam Commands

**This will get you BANNED.** Always use read-act-read cycle:

```bash
# ❌ WRONG - Command spam
echo '{"cmd":"sock_send","data":"north\n"}' | nc -U /tmp/okros/bot.sock
sleep 1
echo '{"cmd":"sock_send","data":"look\n"}' | nc -U /tmp/okros/bot.sock

# ✅ CORRECT - Read between commands
echo '{"cmd":"sock_send","data":"look\n"}' | nc -U /tmp/okros/bot.sock
sleep 1
BUFFER=$(echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/bot.sock)
echo "$BUFFER" | jq -r '.lines[]'
# Parse output, decide next action based on what MUD said
```

### ⚠️ NEVER Guess

**If you can't see the information in MUD output, that's a bug in okros - not a reason to guess.**

```bash
# ❌ WRONG - Guessing based on "common patterns"
echo '{"cmd":"sock_send","data":"adventurer\n"}' | nc -U /tmp/okros/bot.sock

# ✅ CORRECT - Read what MUD tells you
BUFFER=$(echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/bot.sock)
echo "$BUFFER" | jq -r '.lines[]'
# Output: "Choose class: Berserker, Elementalist, Shadowdancer"
echo '{"cmd":"sock_send","data":"Berserker\n"}' | nc -U /tmp/okros/bot.sock
```

Each MUD is unique. Use exactly what the MUD shows you.

### ⚠️ Always Read Prompts

MUDs are conversational. Multi-step flows (character creation, dialogs) require reading each prompt:

```bash
# Step 1: Read prompt
PROMPT=$(echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/bot.sock | jq -r '.lines[-3:][]')

# Step 2: Respond to what you see
if echo "$PROMPT" | grep -q "Type 'create'"; then
  echo '{"cmd":"sock_send","data":"create\n"}' | nc -U /tmp/okros/bot.sock
  sleep 1
fi

# Step 3: Read next prompt, repeat
```

## Special Cases

### Passwords with Special Characters

Use `jq` to safely encode JSON:

```bash
jq -nc --arg pass 'MyPass!@#$%' '{"cmd":"sock_send","data":($pass + "\n")}' | nc -U /tmp/okros/bot.sock
```

### ANSI Colors

Output preserves ANSI codes. Strip if needed:

```bash
# Strip colors
echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/bot.sock | sed 's/\\u001b\[[0-9;]*m//g'
```

### Full-Screen Displays

Some MUDs use cursor positioning for help screens/menus. These **cannot be captured** in headless mode (fundamental limitation). Workarounds:
1. Use TTY mode to see them
2. Check MUD website documentation
3. Try the command anyway based on external knowledge

## Example: Simple Bot

```python
#!/usr/bin/env python3
import socket, json, time

sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect("/tmp/okros/bot.sock")

def send(cmd, data=None):
    msg = {"cmd": cmd, "data": data} if data else {"cmd": cmd}
    sock.sendall(json.dumps(msg).encode() + b'\n')
    return json.loads(sock.recv(65536))

# Connect
send("connect", "mud.example.com:4000")
time.sleep(2)

# Main loop
while True:
    # 1. Read MUD output
    result = send("get_buffer")
    lines = [l for l in result.get("lines", [])[-10:] if l.strip()]
    print("\n".join(lines))

    # 2. Decide action (your LLM logic here)
    action = "look"

    # 3. Send command
    send("sock_send", action + "\n")

    # 4. Wait
    time.sleep(2)
```

## Common Pitfalls

- ❌ **Don't flood** - ALWAYS read responses between commands (1-2s delays minimum)
- ❌ **Don't ignore prompts** - Read and respond to each prompt individually
- ❌ **Don't guess** - Use exactly what MUD tells you, no assumptions
- ❌ **Don't omit `\n`** - Commands need newlines: `"look\n"` not `"look"`
- ❌ **Don't script multi-step flows** - Read each prompt, respond to what you see

## Offline Testing

Test without network:

```bash
# Start with internal test MUD
okros --headless --offline --instance demo

# Play 5-room demo world
echo '{"cmd":"send","data":"look\n"}' | nc -U /tmp/okros/demo.sock
```

## Troubleshooting

```bash
# Check daemon running
ps aux | grep okros

# Check socket exists
ls -la /tmp/okros/*.sock

# Check connection status
echo '{"cmd":"status"}' | nc -U /tmp/okros/bot.sock
```

## Further Reading

- **README.md** - User-facing overview
- **MUD_LEARNINGS.md** - Technical debugging findings
- **src/control.rs** - Full JSON API implementation
- **src/offline_mud/** - Test environment code
