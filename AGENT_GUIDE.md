# Agent Guide: Playing MUDs with okros

This guide explains how to connect AI agents, LLMs, or bots to MUD servers using okros headless mode.

## Quick Start

### 1. Start Headless Daemon

```bash
# Start okros in headless mode
okros --headless --instance mybot

# Socket created at: /tmp/okros/mybot.sock
```

### 2. Connect to MUD

```bash
# Send connect command via Unix socket
echo '{"cmd":"connect","data":"mud.example.com:4000"}' | nc -U /tmp/okros/mybot.sock
# Response: {"event":"Ok"}
```

### 3. Read Output

```bash
# Get current viewport buffer
echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/mybot.sock
# Response: {"event":"Buffer","lines":["Welcome to the MUD!","...", ">"]}
```

### 4. Send Commands

```bash
# Send text to MUD (raw socket write)
echo '{"cmd":"sock_send","data":"look\n"}' | nc -U /tmp/okros/mybot.sock
# Response: {"event":"Ok"}
```

## Control Protocol

All commands are JSON Lines format (one JSON object per line).

### Commands

| Command | Parameters | Description |
|---------|-----------|-------------|
| `connect` | `data: "host:port"` | Connect to MUD server |
| `get_buffer` | (none) | Get current viewport buffer |
| `sock_send` | `data: "text"` | Send raw text to MUD socket |
| `send` | `data: "text"` | Echo text to local buffer (for testing) |
| `status` | (none) | Get connection status |
| `attach` | (none) | Attach to session (for multi-agent) |
| `detach` | (none) | Detach from session |
| `stream` | `interval_ms: 200` | Stream live output (blocking) |
| `quit` / `shutdown` | (none) | Gracefully shutdown headless daemon |

### Responses

| Event | Fields | Description |
|-------|--------|-------------|
| `Ok` | (none) | Command succeeded |
| `Error` | `message: "..."` | Command failed |
| `Buffer` | `lines: ["...", ...]` | Viewport buffer contents |
| `Status` | `attached: bool` | Connection status |

## Understanding Output

### ANSI Color Codes

Output preserves ANSI escape sequences for colors:

```
\u001b[0m        - Reset to default
\u001b[1;32m     - Bold green
\u001b[0;40;31m  - Normal red on black background
```

**Stripping colors** (if you want plain text):
```bash
# Using sed
echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/mybot.sock | \
  sed 's/\\u001b\[[0-9;]*m//g'
```

**Preserving colors** (for terminal display):
```bash
# Colors work in most terminals
printf '{"cmd":"get_buffer"}\n' | nc -U /tmp/okros/mybot.sock | \
  perl -pe 's/\\u001b/\e/g'
```

### Buffer Behavior

- **Viewport size**: Default 80 columns × 20 rows
- **Scrollback**: 2000 lines of history
- **Updates**: Buffer updates on every `feed_inbound()` call
- **Caching**: Repeated `get_buffer` calls are cached (fast)

## Playing MUDs: Best Practices

### ⚠️ CRITICAL: Be Respectful - No Command Spamming

**NEVER send multiple commands without reading responses between each command.** This is:
- Against most MUD rules and will get you banned
- Disrespectful to the MUD community and server operators
- Ineffective (MUDs are conversational and require reading prompts)

**❌ FORBIDDEN - Will Get You Banned:**
```bash
# NEVER chain commands with sleeps - this is spamming
echo '{"cmd":"sock_send","data":"north\n"}' | nc -U /tmp/okros/mybot.sock
sleep 1
echo '{"cmd":"sock_send","data":"look\n"}' | nc -U /tmp/okros/mybot.sock
sleep 1
echo '{"cmd":"sock_send","data":"attack goblin\n"}' | nc -U /tmp/okros/mybot.sock

# NEVER script multi-step processes without reading responses
# Even for character creation - you MUST read each prompt
```

**✅ REQUIRED - Human-Like Interaction:**
```bash
# 1. Send ONE command
echo '{"cmd":"sock_send","data":"look\n"}' | nc -U /tmp/okros/mybot.sock

# 2. Wait for server response
sleep 1

# 3. READ what the MUD says (MANDATORY)
BUFFER=$(echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/mybot.sock)
echo "$BUFFER" | jq -r '.lines[]'

# 4. Parse/analyze the response
# (What is the MUD asking? What are my options?)

# 5. Decide on next action based on what you read
# Then go back to step 1

# NEVER skip step 3 - always read responses
```

### 1. NEVER Guess - Always Use MUD Output

**CRITICAL: If you can't see the information you need in the MUD output, that's a BUG in okros, not a reason to guess.**

**❌ FORBIDDEN:**
```bash
# NEVER guess based on "common MUD patterns"
echo '{"cmd":"sock_send","data":"adventurer\n"}' | nc -U /tmp/okros/mybot.sock  # WRONG!
# Each MUD is unique - "adventurer" might not even be a valid class

# NEVER assume commands work the same across MUDs
echo '{"cmd":"sock_send","data":"north\n"}' | nc -U /tmp/okros/mybot.sock  # Maybe this MUD uses "n" not "north"?
```

**✅ REQUIRED:**
```bash
# 1. Read what the MUD actually tells you
BUFFER=$(echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/mybot.sock)
echo "$BUFFER" | jq -r '.lines[]'

# Output shows: "Choose class: Berserker, Elementalist, Shadowdancer"

# 2. Use EXACTLY what the MUD listed
echo '{"cmd":"sock_send","data":"Berserker\n"}' | nc -U /tmp/okros/mybot.sock
```

**If the MUD output is missing or unclear:**
- That's a bug in okros prompt capture - file an issue
- Use `peek` with larger line counts to see more context
- Check if help text scrolled off (press enter to continue, then re-read)
- **DO NOT guess or assume "standard MUD behavior" - there is no standard**

### 2. Read Before Every Action

MUDs are **conversational** - they ask questions and expect specific responses.

### 2. Handle Multi-Step Prompts

MUDs often require multi-step interactions (character creation, dialogs, etc.).

**Example: Character Creation**

```bash
# Step 1: Read initial prompt
PROMPT=$(echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/mybot.sock | \
  jq -r '.lines[-5:][]')

# Check what it's asking for
if echo "$PROMPT" | grep -q "Type 'create'"; then
  echo '{"cmd":"sock_send","data":"create\n"}' | nc -U /tmp/okros/mybot.sock
  sleep 1
fi

# Step 2: Read next prompt
PROMPT=$(echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/mybot.sock | \
  jq -r '.lines[-5:][]')

# Enter name
if echo "$PROMPT" | grep -q "Enter the name"; then
  echo '{"cmd":"sock_send","data":"mycharacter\n"}' | nc -U /tmp/okros/mybot.sock
  sleep 1
fi

# Step 3: Confirm
PROMPT=$(echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/mybot.sock | \
  jq -r '.lines[-5:][]')

if echo "$PROMPT" | grep -q "Hit ENTER"; then
  echo '{"cmd":"sock_send","data":"\n"}' | nc -U /tmp/okros/mybot.sock
  sleep 1
fi

# And so on...
```

### 3. Parse Output Carefully

MUDs return **unstructured text**. You need to parse:

- **Prompts**: `>`, `HP:100 >`, `[Command]:`, etc.
- **Exits**: `Exits: north, south, east`
- **Items**: `You see: a sword, a shield`
- **NPCs**: `A goblin is here.`
- **Combat**: `You hit the goblin for 15 damage!`

**Example: Extract last prompt line**

```bash
# Get last non-empty line (usually the prompt)
PROMPT=$(echo '{"cmd":"get_buffer"}' | nc -U /tmp/okros/mybot.sock | \
  jq -r '.lines | map(select(length > 0)) | last')

echo "Current prompt: $PROMPT"
```

### 4. Add Delays

MUDs expect human-speed input. Add delays between commands:

```bash
send_command() {
  echo "{\"cmd\":\"sock_send\",\"data\":\"$1\\n\"}" | nc -U /tmp/okros/mybot.sock
  sleep 1  # Wait for MUD to respond
}

send_command "look"
send_command "north"
send_command "inventory"
```

### 5. Handle Disconnects

MUDs can disconnect due to:
- Timeouts (idle too long)
- Network issues
- Server restarts

**Check connection status:**
```bash
STATUS=$(echo '{"cmd":"status"}' | nc -U /tmp/okros/mybot.sock)
# Response: {"event":"Status","attached":true}
```

**Reconnect if needed:**
```bash
if ! echo "$STATUS" | grep -q '"attached":true'; then
  echo '{"cmd":"connect","data":"mud.example.com:4000"}' | nc -U /tmp/okros/mybot.sock
fi
```

## Example: Simple Bot Loop

```bash
#!/bin/bash

SOCK="/tmp/okros/mybot.sock"

# Connect to MUD
echo '{"cmd":"connect","data":"mud.example.com:4000"}' | nc -U $SOCK
sleep 2

# Main loop
while true; do
  # 1. Read buffer
  BUFFER=$(echo '{"cmd":"get_buffer"}' | nc -U $SOCK)

  # 2. Extract last few lines (parse this based on your MUD)
  RECENT=$(echo "$BUFFER" | jq -r '.lines[-5:][]' | grep -v '^$')

  echo "=== MUD Output ==="
  echo "$RECENT"
  echo ""

  # 3. Decide on action (simple example: look every 5 seconds)
  # (Your AI/LLM logic goes here)
  ACTION="look"

  # 4. Send command
  echo "{\"cmd\":\"sock_send\",\"data\":\"$ACTION\\n\"}" | nc -U $SOCK

  # 5. Wait
  sleep 5
done
```

## LLM Integration Example

```python
#!/usr/bin/env python3
import socket
import json
import time

# Connect to okros headless socket
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect("/tmp/okros/mybot.sock")

def send_cmd(cmd, data=None):
    """Send JSON command to okros"""
    msg = {"cmd": cmd}
    if data:
        msg["data"] = data
    sock.sendall(json.dumps(msg).encode() + b'\n')
    response = sock.recv(65536)
    return json.loads(response)

def get_mud_output():
    """Get recent MUD output"""
    result = send_cmd("get_buffer")
    lines = result.get("lines", [])
    # Get last 10 non-empty lines
    recent = [l for l in lines[-10:] if l.strip()]
    return "\n".join(recent)

# Connect to MUD
send_cmd("connect", "mud.example.com:4000")
time.sleep(2)

# Main loop
while True:
    # 1. Get MUD output
    mud_text = get_mud_output()
    print(f"=== MUD Output ===\n{mud_text}\n")

    # 2. Call your LLM (pseudo-code)
    # prompt = f"You are playing a MUD. Current state:\n{mud_text}\n\nWhat do you do?"
    # llm_response = call_llm(prompt)
    # action = parse_llm_action(llm_response)

    # For demo, just send 'look'
    action = "look"

    # 3. Send action to MUD
    print(f">>> {action}")
    send_cmd("sock_send", action + "\n")

    # 4. Wait
    time.sleep(3)
```

## Tips for LLM Agents

### Prompt Engineering

Give your LLM context about MUD mechanics:

```
You are playing a text-based MUD (Multi-User Dungeon).

Rules:
1. Read the output carefully before acting
2. MUD commands are simple verbs: look, north, take sword, attack goblin
3. Pay attention to prompts - they tell you what input is expected
4. Don't repeat commands unnecessarily
5. If you see "Unknown command", try rephrasing

Current MUD output:
{mud_text}

What command do you want to send? Respond with ONLY the command, no explanation.
```

### Common MUD Commands

- **Movement**: `north`, `south`, `east`, `west`, `up`, `down` (or `n`, `s`, `e`, `w`, `u`, `d`)
- **Observation**: `look`, `examine <item>`, `inventory`, `score`
- **Interaction**: `take <item>`, `drop <item>`, `give <item> to <npc>`
- **Combat**: `kill <target>`, `attack <target>`, `flee`
- **Communication**: `say <message>`, `tell <player> <message>`
- **Meta**: `help`, `quit`, `who`, `score`

### Parsing Strategies

1. **Regex for structured data**:
   ```python
   import re
   exits = re.findall(r'Exits?: ([^\.]+)', mud_text)
   items = re.findall(r'You see: (.+)', mud_text)
   ```

2. **LLM-based parsing**:
   ```
   Parse this MUD output and extract:
   - Current room name
   - Available exits
   - Items visible
   - NPCs present

   {mud_text}

   Return JSON format.
   ```

3. **Prompt detection**:
   ```python
   last_line = lines[-1] if lines else ""
   is_prompt = last_line.endswith('>') or ':' in last_line
   ```

## Offline Mode for Testing

Test your agent logic without connecting to a real MUD:

```bash
# Start with internal test MUD
okros --headless --offline --instance demo

# Play the 5-room demo world
echo '{"cmd":"send","data":"look\n"}' | nc -U /tmp/okros/demo.sock
echo '{"cmd":"send","data":"north\n"}' | nc -U /tmp/okros/demo.sock
echo '{"cmd":"send","data":"take sword\n"}' | nc -U /tmp/okros/demo.sock
```

The offline MUD is deterministic and perfect for:
- Testing your agent logic
- Debugging parsing code
- Developing without network dependency

## Common Pitfalls

### ❌ Don't flood the MUD
**This will get you BANNED.** MUDs will disconnect and ban you for spamming. ALWAYS read responses between commands. Use 1-2s delays and NEVER chain commands.

### ❌ Don't ignore prompts
MUDs use prompts to guide you (character creation, dialogs, menus). You MUST read and respond to each one individually.

### ❌ Don't assume structure or guess
MUDs have NO standard format. Each MUD is completely different with unique commands, classes, races, and mechanics. NEVER guess based on "common patterns" - always use what the MUD explicitly tells you. If you can't see the information, that's a bug to fix, not a reason to guess.

### ❌ Don't send partial commands
Always append `\n` to your commands: `"look\n"`, not `"look"`

### ❌ Don't trust the buffer size
The viewport is fixed (80×20 default). Long messages may scroll off. Consider using `peek` with higher counts.

### ❌ Don't script character creation
Character creation is multi-step and interactive. You MUST read each prompt and respond appropriately. NEVER pre-script the entire flow.

## Special Characters in Passwords

When sending passwords with special characters, use `jq` to properly escape JSON:

```bash
# Use jq -nc with --arg to safely handle special characters
jq -nc --arg pass 'MyPassword!@#$%' '{"cmd":"sock_send","data":($pass + "\n")}' | nc -U /tmp/okros/instance.sock

# This handles: ! @ # $ % ^ & * ( ) and other special chars safely
```

## Understanding Prompts

MUDs use two types of prompts:

1. **Prompts with telnet GA/EOR signals** - okros automatically captures these
2. **Prompts without newlines or signals** - okros captures these in the current line buffer

Both types are now properly captured by `get_buffer` and `peek` commands.

## Known Limitations: Full-Screen Displays

**IMPORTANT: Some MUD content cannot be captured in headless mode.**

### The Problem

Some MUDs use ANSI cursor positioning (e.g., `ESC[H`, `ESC[row;colH`) to draw full-screen displays like:
- Help pages during character creation
- Complex menus
- Character sheets
- Maps

When a MUD uses cursor positioning to draw text at arbitrary screen locations (instead of line-by-line output), okros's scrollback buffer cannot capture it. The scrollback only captures newline-terminated text via `print_line()`.

### Example: Class Selection Help

During Nodeka character creation, typing "help" at the class selection prompt displays a full-screen list of classes. However, this help screen:
- Uses cursor positioning to draw the list
- Gets cleared when you press enter
- Never appears in the scrollback buffer
- Results in an empty buffer or just "Press enter to continue"

### Workarounds

1. **Try the command anyway** - If you know a valid option (e.g., "Fallad" for Nodeka), just try it:
   ```bash
   echo '{"cmd":"sock_send","data":"Fallad\n"}' | nc -U /tmp/okros/instance.sock
   ```

2. **Use TTY mode instead** - Run `okros` interactively (not headless) to see the full-screen displays

3. **Check MUD documentation** - Look up valid options on the MUD's website or wiki

4. **Trial and error** - If the MUD rejects your input, it will tell you and you can try again

### This is NOT a Bug

This is a fundamental limitation of headless mode. okros was designed as a line-oriented MUD client for TTY use. Full-screen cursor-positioned displays are visible in TTY mode but cannot be captured in scrollback history for headless/agent mode.

**Bottom line**: If you can't see expected information in the buffer, it might be cursor-positioned content. Try the command anyway based on external knowledge of the MUD.

## Advanced: Stream Mode

For real-time monitoring, use stream mode:

```bash
# Start streaming (blocks and sends updates every 200ms)
echo '{"cmd":"stream","interval_ms":200}' | nc -U /tmp/okros/mybot.sock

# Output: continuous JSON lines with buffer updates
{"event":"Buffer","lines":[...]}
{"event":"Buffer","lines":[...]}
...
```

**Warning**: Stream mode is blocking. Use in a separate thread/process.

## Troubleshooting

### "Connection refused" error
```bash
# Make sure daemon is running
ps aux | grep okros

# Check socket exists
ls -la /tmp/okros/*.sock
```

### Empty buffer
```bash
# Check if connected
echo '{"cmd":"status"}' | nc -U /tmp/okros/mybot.sock

# Reconnect if needed
echo '{"cmd":"connect","data":"mud.example.com:4000"}' | nc -U /tmp/okros/mybot.sock
```

### Garbled output
ANSI codes may appear as escape sequences. Strip them or convert:
```bash
# Strip all ANSI
sed 's/\\u001b\[[0-9;]*m//g'

# Or convert to real escapes for terminal
perl -pe 's/\\u001b/\e/g'
```

## Further Reading

- **README.md**: User-facing okros overview
- **ORIENTATION.md**: Project status and architecture
- **Control Protocol**: See `src/control.rs` for full JSON API
- **Offline MUD**: See `src/offline_mud/` for test environment

---

Happy MUDding! 🦀
