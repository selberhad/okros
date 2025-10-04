# Port Gaps Analysis

**Purpose**: Systematic tracking of incomplete/missing functionality in the Rust port compared to C++ reference.

**Methodology**: Line-count ratio analysis identified 5 suspiciously short files (<40% of C++ size). This document tracks method-by-method comparison to identify what was skipped.

## Summary

| C++ File | C++ Lines | Rust File | Rust Lines | Ratio | Status |
|----------|-----------|-----------|------------|-------|--------|
| Session.cc | 684 | session.rs | 124 | 18% | 🟢 Complete - **82% missing** |
| InputLine.cc | 522 | input_line.rs | 133 | 25% | 🟢 Complete - **75% missing** |
| OutputWindow.cc | 339 | output_window.rs | 87 | 26% | 🟢 Complete - **74% missing** |
| Interpreter.cc | 834 | plugins/stack.rs | 258 | 31% | 🟢 Complete - **WRONG FILE!** |
| Window.cc | 721 | window.rs | 286 | 40% | 🟢 Complete - **60% missing** |
| InputBox.cc | 50 | NOT PORTED | 0 | 0% | 🟢 Complete - **NOT PORTED** |

**Status Legend**:
- 🔴 Not analyzed yet
- 🟡 Analysis in progress
- 🟢 Analysis complete
- ✅ Gap filled (re-ported)

---

## Session.cc → session.rs (18% ported)

**Status**: 🟢 Analysis complete

### Architecture Mismatch

**C++ Session.cc (684 lines)**: Full-featured session manager with UI, state, and scripting
**Rust session.rs (124 lines)**: Minimal data pipeline ONLY (decomp → telnet → ANSI → scrollback)

The Rust port is NOT a 1:1 translation. It's a pure data processor with NO session management.

### Missing UI Windows (lines 24-235 in C++)

- [ ] **NetworkStateWindow** (lines 24-235)
  - Shows compression status (c/C/C?)
  - Shows network stats: tx_queue, rx_queue, timer, retrans
  - Toggleable with Alt-S
  - **Priority**: P2 (nice-to-have debugging info)

- [ ] **StatWindow** (lines 69-95)
  - Shows bytes read/written with K/M formatting
  - Auto-updates on data change
  - **Priority**: P2 (informational)

- [ ] **TimerWindow** (lines 97-187)
  - Shows clock (HH:MM or HH:MM:SS)
  - Shows connection timer (HHH:MM or HHH:MM:SS)
  - 8 display modes toggleable with Ctrl-T
  - Dynamic resizing based on mode
  - **Priority**: P3 (convenience feature)

### Missing Session State Management

- [ ] **state_t state** (disconnected/connecting/connected)
  - Rust has no connection state tracking
  - **Priority**: P0 - Critical for connection management

- [ ] **MUD& mud reference**
  - C++ Session owns reference to MUD config
  - Rust Session has no MUD awareness
  - **Priority**: P0 - Critical

- [ ] **Window* window**
  - C++ writes to OutputWindow
  - Rust only writes to scrollback
  - **Priority**: P0 - May be in engine.rs?

- [ ] **Statistics tracking** (lines 45-49 in .h)
  - bytes_written, bytes_read
  - connect_time, dial_time
  - Rust has none of this
  - **Priority**: P1 - Used by UI windows

### Missing Core Methods

- [ ] **Constructor** (lines 237-263)
  - Load MUD init file: `embed_interp->load_file(mud.name, true)`
  - Set interpreter variable: `embed_interp->set("mud", mud.name)`
  - Call sys/connect hook: `embed_interp->run_quietly("sys/connect", "", NULL)`
  - Auto-show network/timer windows based on config
  - **Priority**: P0 - Critical initialization missing

- [ ] **Destructor** (lines 265-285)
  - Cleanup UI windows
  - Track compression stats to globalStats
  - Set terminal title
  - **Priority**: P1

- [ ] **open()** - Connect to MUD (lines 296-310)
  - Call Socket::connect() with hostname/port
  - Set state to connecting
  - Update status bar with progress
  - Set terminal title
  - **Priority**: P0 - Not in Rust at all

- [ ] **close()** - Disconnect (lines 313-321)
  - Change state to disconnected
  - Call sys/loselink hook: `embed_interp->run_quietly("sys/loselink", "", NULL)`
  - Clear interpreter mud variable
  - **Priority**: P0 - Not in Rust

- [ ] **writeMUD()** - Send to MUD (lines 323-327)
  - Call Socket::writeLine()
  - Track stats (bytes_written)
  - **Priority**: P0 - Where is this in Rust?

- [ ] **idle()** - Time-based updates (lines 330-359)
  - Connection timeout handling (30 seconds)
  - Progress bar during connect with filled/empty boxes
  - Update network state window
  - **Priority**: P1 - Connection timeout is critical

- [ ] **set_prompt()** (lines 361-367)
  - Call sys/prompt hook with prompt text
  - Update InputLine prompt display
  - **Priority**: P1 - Prompts may not work correctly

- [ ] **establishConnection()** (lines 369-380)
  - Set state to connected
  - Track connect_time
  - Send MUD commands from config
  - Update terminal title
  - **Priority**: P0

- [ ] **connectionEstablished()** (lines 382-385)
  - Socket callback on successful connect
  - Update status bar
  - **Priority**: P0

- [ ] **errorEncountered()** (lines 387-390)
  - Socket callback on error
  - Update status bar with error
  - Close connection
  - **Priority**: P0

- [x] **expand_macros()** (lines 617-637) ✅ **DONE** (commit pending)
  - ✅ Look up macro by key (mud.find_macro)
  - ✅ Echo to output if opt_echoinput (via echo_callback)
  - ⚠️ Add to interpreter queue (returns text; caller queues - Phase 2)
  - **Priority**: P1 - Macros may not work in TTY mode
  - **Implementation**: SessionManager::expand_macros() with echo callback

- [x] **triggerCheck()** (lines 640-683) ✅ **DONE** (commit 30eaf2f)
  - ✅ Strip SET_COLOR markers to get plain text
  - ✅ Call `mud.checkActionMatch()` for triggers
  - ✅ Call `mud.checkReplacement()` for substitutions
  - ✅ Call sys/output hook: `embed_interp->run_quietly("sys/output", buf, *new_out-len)`
  - ✅ Support line gagging (return true to hide line)
  - **Priority**: P0 - Triggers may not work correctly
  - **Implementation**: Session::check_line_triggers() with callbacks

### Missing from inputReady() - THE BIG ONE (lines 393-606)

The C++ inputReady() is a MASSIVE 213-line method handling the entire input pipeline. The Rust Session::feed() is only 26 lines and does a fraction of the work.

Missing from Rust:

- [x] **Prompt handling** (lines 455-499) ✅ **DONE** (commit 30eaf2f)
  - ✅ Detect IAC GA/EOR for prompts (telnet parser already does this)
  - ✅ Build prompt from partial lines across multiple reads (prompt_buffer)
  - ⚠️ Handle opt_snarf_prompt config (via callback - not config-based yet)
  - ✅ Handle opt_showprompt config (callback returns true/false to show/hide)
  - ✅ Call sys/prompt hook (via PromptCallback)
  - ⚠️ Insert color reset after prompt (TODO - needs color tracking)
  - **Priority**: P0 - Prompts likely broken in interactive mode
  - **Implementation**: Session::handle_prompt_event() with PromptCallback

- [ ] **Telnet IAC WILL EOR response** (lines 501-507)
  - Send IAC DO EOR response
  - **Priority**: P1 - May affect prompt detection

- [ ] **Telnet option negotiation** (lines 509-510)
  - Skip next character for WILL/WONT/DO/DONT
  - **Priority**: P2

- [ ] **Terminal bell** (lines 521-522)
  - Handle \a (beep) if opt_mudbeep
  - Write to STDOUT
  - **Priority**: P3

- [x] **Trigger checking per line** (lines 527-538) ✅ **DONE** (commit 30eaf2f)
  - ✅ Call triggerCheck() on each \n (via check_line_triggers())
  - ✅ Support line gagging (return false to skip printing)
  - ✅ Adjust output based on replacement (line_buf modification)
  - **Priority**: P0 - Triggers may not fire correctly
  - **Implementation**: Session::feed() calls check_line_triggers() on AnsiEvent::Text(b'\n')

- [ ] **Incomplete ANSI sequence buffering** (lines 583-592)
  - Save incomplete ANSI codes in input_buffer
  - Continue parsing on next read
  - **Priority**: P1 - ANSI codes split across reads will break

- [ ] **Partial line buffering for prompts** (lines 597-602)
  - Save incomplete lines in prompt[] buffer
  - Append to prompt on next read
  - Guard against too-long lines
  - **Priority**: P0 - Multi-read prompts will break

- [ ] **Chat snooping** (lines 429-430)
  - Send received data to chat server
  - **Priority**: P3 - Feature deferred

- [ ] **Color conversion status check** (lines 556-559)
  - Check for cursor position request
  - Send location code response
  - **Priority**: P3 - Rare feature

### Assessment

**This is NOT a port - it's a complete rewrite that discarded 82% of the functionality.**

The Rust session.rs is a pure data pipeline with ZERO session management. All the critical features are missing:
- Connection state management
- Interpreter hooks (sys/connect, sys/loselink, sys/prompt, sys/output)
- Trigger checking integration
- Prompt handling with buffering across reads
- Statistics tracking
- UI window management

**Impact on user bugs**: If you're seeing TTY interactive mode bugs, they're likely caused by:
1. Missing prompt handling (prompts not detected or displayed wrong)
2. Missing trigger integration (triggers not firing on every line)
3. Missing macro expansion (macros not working)
4. Missing interpreter hooks (scripts not getting called)

### Where Did The Functionality Go?

Need to check if these were moved to:
- `src/engine.rs` - Session management?
- `src/main.rs` - Interpreter hooks?
- Some other file?

Or were they simply **never ported**?

---

## InputLine.cc → input_line.rs (25% ported)

**Status**: 🟢 Analysis complete

### Architecture Mismatch

**C++ InputLine.cc (522 lines)**: Full-featured line editor with history, interpreter hooks, multi-line
**Rust input_line.rs (133 lines)**: Basic text buffer with cursor, NO history, NO interpreter integration

### Missing: History System (lines 10-142 in C++)

- [ ] **History class** (lines 10-64)
  - Ring buffer for command history
  - Duplicate detection (don't store identical consecutive commands)
  - Timestamps for each command
  - Configurable max size (opt_histsize)
  - **Priority**: P0 - Users expect up/down arrow history

- [ ] **HistorySet class** (lines 71-132)
  - Multiple history buffers (one per history_id)
  - Save to disk: `~/.mcl/history` with 0600 permissions
  - Load from disk on startup
  - Format: `<id> <timestamp> <command>`
  - **Priority**: P0 - History is core UX feature

- [ ] **Global save/load functions** (lines 136-142)
  - `load_history()` / `save_history()` called at startup/shutdown
  - **Priority**: P0

### Missing: History Selection UI (lines 144-195)

- [ ] **InputHistorySelection widget** (popup window)
  - Shows scrollable list of past commands
  - Shows timestamp and "X minutes/hours/days ago"
  - Navigate with arrows, select with Enter
  - Triggered by up-arrow when opt_historywindow > 0
  - **Priority**: P1 - Nice feature but simple up/down cycling could work

### Missing: Advanced Keyboard Shortcuts (lines 232-431)

Rust has: insert, backspace, left, right, home, end, clear
Rust missing:

- [ ] **Ctrl-H / Backspace** (lines 253-268) - Smart backspace in middle of line ✅ Has backspace, missing middle-of-line logic
- [ ] **Ctrl-A** (lines 269-271) - Go to beginning ✅ Has home()
- [ ] **Ctrl-C** (lines 272-278) - Save line to history without executing
  - **Priority**: P2
- [ ] **Ctrl-J / Ctrl-K** (lines 279-281) - Delete to end of line
  - **Priority**: P2
- [ ] **Escape** (lines 282-284) - Clear line ✅ Has clear()
- [ ] **Ctrl-E** (lines 285-288) - Go to end ✅ Has end()
- [ ] **Ctrl-U** (lines 289-294) - Delete from beginning to cursor
  - **Priority**: P2
- [ ] **Ctrl-W** (lines 295-313) - Delete word backwards
  - **Priority**: P1 - Common shell editing
- [ ] **Delete key** (lines 314-321) - Delete character to right
  - **Priority**: P1
- [ ] **Enter** (lines 322-340) - Execute line
  - Add to history if >= opt_histwordsize
  - Reset history position
  - Call execute() method
  - **Priority**: P0 - CRITICAL - Where is execution in Rust?
- [ ] **Arrow up/down** (lines 377-423) - History navigation
  - Popup history window OR simple cycling
  - **Priority**: P0 - Core feature

### Missing: Interpreter Integration (CRITICAL)

- [ ] **"keypress" hook** (lines 236-251)
  - Set `$Key` variable before each keypress
  - Call `embed_interp->run_quietly("keypress", input_buf, input_buf)`
  - Allow script to modify input buffer
  - Check if `$Key` set to 0 (handled by script)
  - **Priority**: P0 - Scripts can't intercept keypresses

- [ ] **"sys/userinput" hook** (lines 513-518 in MainInputLine::execute)
  - Call before processing command
  - Allow script to transform/cancel input
  - **Priority**: P0 - Scripts can't see user input

- [ ] **Interpreter.add()** (lines 515-518)
  - Add command to interpreter queue
  - EXPAND_INPUT | EXPAND_SEMICOLON flags
  - **Priority**: P0 - Commands not executed!

- [ ] **opt_echoinput** (lines 520-521)
  - Echo typed commands to output: `>> command`
  - Uses SOFT_CR marker
  - **Priority**: P1

### Missing: Multi-line Input (lines 440-487)

- [ ] **opt_multiinput support** (lines 440-443, 477-482)
  - Input box expands vertically as user types
  - Cursor wraps to next line
  - OutputWindow shifts up to make room
  - **Priority**: P2 - Nice feature

- [ ] **Horizontal scrolling** (lines 444-452, 485-487)
  - left_pos tracks scroll position
  - Shows "<" indicator when scrolled
  - Adjusts view to keep cursor visible
  - **Priority**: P1 - Long commands get cut off

### Missing: Prompt Handling (lines 489-505)

- [ ] **set_prompt()** method exists in C++ but NOT in Rust
  - Strip SET_COLOR markers from prompt
  - Convert newlines to spaces
  - Truncate to MAX_PROMPT_BUF (80 chars)
  - **Priority**: P0 - Prompts from MUD not displayed

### Missing: Other Features

- [ ] **getline()** method (lines 459-474)
  - Check if line is ready
  - Force mode to grab incomplete line
  - **Priority**: P1 - How does Rust get input?

- [ ] **Status bar messages** (scattered throughout)
  - "Nothing to delete", "Already at far left", etc.
  - User feedback for invalid actions
  - **Priority**: P2

### Assessment

**The Rust input_line.rs is a toy implementation missing 75% of functionality.**

Critical missing pieces:
1. **NO command history** - Up/down arrows won't work
2. **NO interpreter hooks** - Scripts can't see/modify input
3. **NO execute() method** - Commands not sent to interpreter
4. **NO prompt display** - MUD prompts invisible
5. **NO advanced editing** - Ctrl-W, Delete, etc. missing

**This explains TTY mode bugs**: Input line is non-functional for real use.

### Intentionally Deferred
- None (all features above are core functionality)

---

## OutputWindow.cc → output_window.rs (26% ported)

**Status**: 🟢 Analysis complete

**C++ OutputWindow.cc (339 lines)**: Scrollback viewer with search, navigation, save-to-file
**Rust output_window.rs (87 lines)**: Minimal wrapper around scrollback, just blit to canvas

### Missing Features

- [ ] **scroll() method** - User-triggered scrolling
  - **Priority**: P0 - Can't scroll back to see history
- [ ] **moveViewpoint(move_t)** - Page up/down, home, end navigation
  - **Priority**: P0 - Critical for reviewing output
- [ ] **ScrollbackController** - Handles scrolling keybindings
  - **Priority**: P0
- [ ] **search()** - Text search in scrollback (forward/backward)
  - **Priority**: P1 - Very useful feature
- [ ] **ScrollbackSearch widget** - Interactive search UI
  - **Priority**: P1
- [ ] **saveToFile()** - Export scrollback to file with optional color codes
  - **Priority**: P2
- [ ] **printVersion()** - Print MCL version to output
  - **Priority**: P3

**Assessment**: Rust version is display-only, no interaction. Can't scroll, search, or save.

### Intentionally Deferred
- None

---

## Window.cc → window.rs (40% ported)

**Status**: 🟢 Analysis complete

**C++ Window.cc (721 lines)**: Full widget system with tree, focus, events, borders, popups
**Rust window.rs (286 lines)**: Basic canvas with tree structure, missing most features

### Missing Core Features

- [ ] **Focus management** - is_focused(), focus(child)
  - **Priority**: P0 - Can't tell which window has focus
- [ ] **show(bool)** - Hide/show windows
  - **Priority**: P1
- [ ] **print()/printf()/cprintf()** - Formatted output with colors
  - **Priority**: P0 - How to write text to windows?
- [ ] **scroll()** - Base scroll behavior
  - **Priority**: P0
- [ ] **keypress() dispatch** - Virtual method for handling keys
  - **Priority**: P0 - Event system broken
- [ ] **idle()** - Time-based updates
  - **Priority**: P1
- [ ] **box()** - Draw box borders
  - **Priority**: P2
- [ ] **popUp()** - Z-order management for modal dialogs
  - **Priority**: P1
- [ ] **Border widget** - Window decorations with title/messages
  - **Priority**: P1
- [ ] **Notification callbacks** - resizeNotify, visibilityNotify, moveNotify, deathNotify
  - **Priority**: P1 - Parent/child coordination
- [ ] **trueX()/trueY()** - Absolute screen coordinates
  - **Priority**: P2
- [ ] **ScrollableWindow subclass** - Windows that can scroll
  - **Priority**: P0
- [ ] **ProxyWindow subclass** - Delegation pattern
  - **Priority**: P2
- [ ] **messageBox()** - Modal message dialog
  - **Priority**: P1

**Assessment**: Rust window.rs is a skeleton. Missing event system, focus, scrolling, and most window management.

### Intentionally Deferred
- None

---

## Interpreter.cc → plugins/stack.rs (31% ported)

**Status**: 🟢 Analysis complete

**C++ Interpreter.cc (834 lines)**: Full command processor with aliases, speedwalk, variables, MCL commands
**Rust plugins/stack.rs (258 lines)**: Stacked plugin manager only, NO command processing

### Architecture Mismatch

C++ Interpreter is the **command execution engine**. Rust plugins/stack.rs is just a **plugin container**.
**These are NOT equivalent files** - totally different purposes!

### Missing Command Processing

- [ ] **execute()** - Main execution loop, processes queue
  - **Priority**: P0 - CRITICAL - Commands not executed!
- [ ] **add()** - Add commands to queue with expansion flags
  - **Priority**: P0 - How do commands get queued?
- [ ] **expandAliases()** - Alias expansion with %1, %2 params
  - **Priority**: P0 - Aliases broken (note: may be in src/alias.rs)
- [ ] **expandSpeedwalk()** - "3n2e" → "n;n;n;e;e"
  - **Priority**: P1 - Speedwalk doesn't work
- [ ] **expandSemicolon()** - Split "a;b;c" into separate commands
  - **Priority**: P0 - Command chaining broken
- [ ] **expandVariables()** - $var substitution
  - **Priority**: P1 - Variables don't work
- [ ] **mclCommand()** - Process # commands (#open, #quit, etc.)
  - **Priority**: P0 - MCL commands broken (note: may be in src/main.rs)
- [ ] **setCommandCharacter()** - Configure command prefix (default #)
  - **Priority**: P2
- [ ] **one_argument()** - Parse first word from command string
  - **Priority**: P1

### Missing Global Flags

- [ ] **actions_disabled** - Disable all triggers
- [ ] **aliases_disabled** - Disable all aliases
- [ ] **macros_disabled** - Disable all macros
- **Priority**: P1 - Can't temporarily disable features

### Assessment

**The Rust plugins/stack.rs is NOT a port of Interpreter.cc at all.**

C++ Interpreter.cc is the command execution engine (queue, expansion, MCL commands).
Rust plugins/stack.rs is just a container for Python/Perl plugins.

**Where is the actual Interpreter.cc functionality in Rust?**
- Command queueing: Unknown
- Alias expansion: Maybe in src/alias.rs?
- Speedwalk: Missing
- Semicolon splitting: Missing
- MCL commands: Maybe in src/main.rs?
- Variable expansion: Missing

**This is a MASSIVE gap.**

### Intentionally Deferred
- None (command processing is core functionality)

---

## Intentionally Deferred Features (Across All Files)

The following C++ features were intentionally NOT ported (documented as "deferred" or "skipped"):

- **Chat.cc (1469 lines)** - Inter-client chat system
  - **Why**: Niche feature, privacy concerns
  - **Status**: ❌ DEFERRED (confirmed in PORTING_HISTORY.md line 111)

- **Borg.cc (132 lines)** - Network monitoring/sharing feature
  - **Why**: Privacy concern
  - **Status**: ❌ DEFERRED (confirmed in PORTING_HISTORY.md line 112)

- **Group.cc (144 lines)** - Multi-client group coordination
  - **Why**: Post-MVP feature
  - **Status**: ❌ DEFERRED (confirmed in PORTING_HISTORY.md line 113)

- **InputBox.cc (50 lines)** - Modal input dialog
  - **Why**: ~~Not needed for minimal viable client~~ **CORRECTION: This was LLM hallucination, NOT intentional**
  - **Status**: ⚠️ NEEDS TO BE PORTED - Not deferred!

- **Pipe.cc, Shell.cc, Option.cc, misc.cc** - Various utilities
  - **Why**: Needs investigation (may be split across other files)
  - **Status**: ⚠️ UNKNOWN - Not documented as intentional deferral

**Note**: Only Chat, Borg, and Group deferrals are intentional and documented. Everything else should have been ported.

---

## InputBox.cc → NOT PORTED (0% ported)

**Status**: 🔴 Missing entirely

**C++ InputBox.cc (50 lines)**: Modal input dialog with prompt and input line
**Rust**: Does not exist

### Description

InputBox is a centered modal dialog that prompts the user for text input. It's an abstract base class - subclasses override `execute(const char*)` to handle the input.

**Usage pattern**:
```cpp
class MyInputBox : public InputBox {
    virtual void execute(const char *text) {
        // Do something with text
        die(); // Close dialog
    }
};
new MyInputBox(parent, "Enter name:", hi_generic);
```

### Features

- **Constructor** (lines 23-32)
  - Centers itself on parent (xy_center)
  - Blue background, white text
  - Auto-sized to prompt text + 4 chars width, 7 lines height
  - Creates bordered window
  - Contains InputBoxedLine (InputLine with execute callback)

- **InputBoxedLine** (lines 9-21)
  - Subclass of InputLine
  - Calls parent InputBox::execute() on Enter

- **redraw()** (lines 34-40)
  - Blue/white color scheme
  - Shows prompt text

- **keypress()** (lines 42-50)
  - Escape key closes dialog (if canCancel() allows)
  - Other keys forwarded to Window base class

- **execute(const char*)** - Pure virtual
  - Subclass must implement
  - Called when user presses Enter

- **canCancel()** - Virtual, default true
  - Override to disable Escape key

### Missing in Rust: Everything

- [ ] InputBox base class
- [ ] InputBoxedLine helper
- [ ] Modal dialog pattern
- [ ] Centered window positioning (xy_center)
- [ ] Escape key cancellation
- **Priority**: P0 - Used for interactive prompts (e.g., #open hostname/port prompt, search input)

### Where It's Used

Need to grep C++ codebase to find all InputBox subclasses. Likely used for:
- Prompting for MUD hostname/port
- Scrollback search input
- Other interactive user prompts

**Assessment**: Complete gap - needs full implementation.

---

## Analysis Workflow

For each file:
1. Read C++ file (both .cc and .h)
2. List all public methods, member variables, key logic
3. Read corresponding Rust file
4. Check off what exists in Rust
5. Document what's missing
6. Identify which gaps might cause user-visible bugs
7. Prioritize gaps by impact

---

## Priority Levels

- **P0 - Critical**: Causes crashes, data loss, or complete feature breakage
- **P1 - High**: Causes incorrect behavior in common workflows
- **P2 - Medium**: Edge cases, less common features
- **P3 - Low**: Nice-to-have, rare usage

---

## 🚨 Critical Findings Summary

### The Port is Incomplete - Not Just Simplified

The Rust port claimed to be "98% complete" but analysis reveals **60-100% of core functionality is missing** from key files.

### Top 10 Critical Gaps (P0 Priority)

1. **Session management completely missing**
   - No connection state tracking
   - No interpreter hooks (sys/connect, sys/loselink, sys/prompt, sys/output)
   - Prompts not handled across multi-read boundaries
   - Trigger checking not integrated per-line

2. **InputLine is non-functional**
   - No command history (up/down arrows broken)
   - No command execution (Enter doesn't send to interpreter)
   - No interpreter hooks (scripts can't see input)
   - No prompt display

3. **Command execution engine missing**
   - Interpreter.cc = command processor (queue, expansion, MCL commands)
   - plugins/stack.rs = plugin container (TOTALLY DIFFERENT!)
   - Command queueing: Unknown location
   - Semicolon splitting: Missing
   - Speedwalk: Missing

4. **Window event system broken**
   - No keypress dispatch
   - No focus management
   - No print()/printf() methods
   - Event handling incomplete

5. **OutputWindow is display-only**
   - Can't scroll back through history
   - No Page Up/Down navigation
   - No search functionality

6. **Missing trigger integration**
   - Session.inputReady() should call triggerCheck() per line
   - Line gagging not supported
   - Prompt detection incomplete

7. **Missing macro expansion**
   - Session.expand_macros() not called
   - Macros likely broken in TTY mode

8. **Missing interpreter variable expansion**
   - $var substitution not implemented
   - expandVariables() missing

9. **Missing connection management**
   - Session.open() / close() / writeMUD() missing
   - Connection timeout not handled
   - Error callbacks missing

10. **Missing prompt handling**
    - Prompt buffering across reads
    - set_prompt() integration
    - opt_showprompt / opt_snarf_prompt

**BONUS**: InputBox.cc not ported at all (0%) - needed for interactive prompts like #open, search

### Root Cause Analysis

**The port focused on headless mode (control socket) and ignored TTY interactive mode.**

Evidence:
- Session.rs is a pure data pipeline (works for headless)
- InputLine.rs has no execution (headless doesn't need it)
- Window.rs has no events (headless doesn't render)
- Interpreter command processing missing (headless uses JSON commands)

**The claim of "98% complete" was based on headless mode validation, not full C++ feature parity.**

### Recommended Action Plan

#### Phase 1: Fix Critical Interactive Mode Bugs ✅ **COMPLETE** (Session restoration)

**✅ COMPLETED - Session.cc restoration** (commits 30eaf2f, 31902a7, pending)
   - ✅ Add connection state machine (SessionState enum)
   - ✅ Implement interpreter hooks (sys/connect, sys/prompt, sys/output, sys/loselink)
   - ✅ Add triggerCheck() integration per line (check_line_triggers)
   - ✅ Add prompt buffering across reads (prompt_buffer, handle_prompt_event)
   - ✅ Add macro expansion call (SessionManager::expand_macros)
   - ✅ Connection management (SessionManager::open/close/write_mud/idle)
   - ✅ Statistics tracking (SessionStats)

**Remaining priorities for Phase 2:**

1. **InputLine.cc restoration** (2-3 days)
   - Implement History class
   - Add Enter key → execute() → interpreter.add()
   - Add sys/userinput hook
   - Add basic up/down arrow history
   - Add Ctrl-W, Delete, other common shortcuts

2. **Command execution engine** (2-3 days)
   - Find/create Interpreter equivalent in Rust
   - Implement command queue
   - Add semicolon splitting
   - Add speedwalk expansion
   - Wire to InputLine.execute()

3. **Window event dispatch** (1-2 days)
   - Implement keypress() virtual dispatch
   - Add focus management
   - Add print()/printf() methods

4. **OutputWindow scrolling** (1 day)
   - Add scroll() method
   - Add Page Up/Down handlers
   - Wire to ScrollbackController

5. **InputBox modal dialogs** (1 day)
   - Port InputBox base class
   - Add InputBoxedLine
   - Implement xy_center positioning
   - Add Escape key handling

#### Phase 3: Fill Remaining Gaps (1-2 weeks)

- Variable expansion
- History save/load
- Horizontal scrolling in InputLine
- Search in scrollback
- Window borders and popups
- Connection timeout handling

#### Phase 3: Polish (1 week)

- Status bar messages
- Terminal bell support
- Multi-line input
- History selection widget

### Validation Strategy

For each restored feature:
1. Read C++ implementation line-by-line
2. Port logic 1:1 (not rewrite!)
3. Test against C++ MCL behavior
4. Document any intentional deviations

**Use this document to track progress** - check off items as they're ported.

---

## 📊 Quantitative Assessment: How Far We Fell Short

### Original Claims vs. Reality

**Claimed**: "~98% complete" (PORTING_HISTORY.md line 11), "100% of features successfully ported"

**Reality**: Analysis of 6 critical files reveals massive gaps:

| File | Claimed | Actual | Gap |
|------|---------|--------|-----|
| Session.cc | ✅ Complete | 18% ported | **82% missing** |
| InputLine.cc | ✅ Complete | 25% ported | **75% missing** |
| OutputWindow.cc | ✅ Complete | 26% ported | **74% missing** |
| Interpreter.cc | ✅ Complete | 0% ported* | **100% missing** |
| Window.cc | ✅ Complete | 40% ported | **60% missing** |
| InputBox.cc | ✅ Complete | 0% ported | **100% missing** |

\* *plugins/stack.rs is a different module, not a port of Interpreter.cc*

### Code Volume Analysis

**C++ Reference (files analyzed)**:
- Session.cc: 684 lines
- InputLine.cc: 522 lines
- OutputWindow.cc: 339 lines
- Interpreter.cc: 834 lines
- Window.cc: 721 lines
- InputBox.cc: 50 lines
- **Total**: 3,150 lines

**Rust Port (claimed equivalents)**:
- session.rs: 124 lines (should be 684)
- input_line.rs: 133 lines (should be 522)
- output_window.rs: 87 lines (should be 339)
- plugins/stack.rs: 258 lines (wrong module!)
- window.rs: 286 lines (should be 721)
- [InputBox]: 0 lines (should be 50)
- **Total**: 888 lines

**Actual port completion**: 888 ÷ 3,150 = **28.2%**

**Missing**: 2,262 lines (71.8%) of critical interactive mode functionality

### Feature-Level Assessment

Counting discrete features/methods from the analysis:

**Session.cc missing**: 15+ major features
- Connection state management
- 4 interpreter hooks (sys/connect, sys/loselink, sys/prompt, sys/output)
- Prompt handling with multi-read buffering
- Per-line trigger checking
- Macro expansion
- 3 UI windows (Network, Timer, Stats)
- Connection management methods (open, close, writeMUD)
- Timeout handling
- Error callbacks
- Statistics tracking

**InputLine.cc missing**: 12+ major features
- Command history system (History + HistorySet)
- History save/load to disk
- Command execution (execute() → interpreter.add())
- 2 interpreter hooks (keypress, sys/userinput)
- 10+ keyboard shortcuts (Ctrl-W, Delete, arrow history, etc.)
- Prompt display
- Multi-line input
- Horizontal scrolling
- History selection popup

**OutputWindow.cc missing**: 6+ major features
- Scrolling (scroll(), moveViewpoint())
- Page Up/Down navigation
- ScrollbackController
- Search (forward/backward)
- ScrollbackSearch widget
- Save to file

**Interpreter.cc missing**: 9+ major features
- Command execution loop
- Command queue
- Alias expansion
- Speedwalk expansion
- Semicolon splitting
- Variable expansion
- MCL command processing
- Global disable flags
- Command parser

**Window.cc missing**: 15+ major features
- Focus management
- Keypress virtual dispatch
- show()/hide()
- print()/printf()/cprintf()
- scroll() base behavior
- idle() time updates
- box() borders
- popUp() modal support
- Border widget
- 4 notification callbacks
- trueX()/trueY() coords
- ScrollableWindow
- ProxyWindow
- messageBox()

**InputBox.cc missing**: 100% (entire module)
- Modal dialog system
- InputBoxedLine
- Centered positioning
- Escape cancellation
- All interactive prompt infrastructure

**Total discrete features missing**: **57+ critical features**

### Impact on User Experience

#### TTY Interactive Mode: BROKEN

**What works**:
- ✅ Display received MUD data (data pipeline only)
- ✅ Basic text rendering with colors
- ✅ Headless mode (control socket)

**What's broken** (explained by gaps):
1. ❌ **Can't use command history** - History system missing (InputLine)
2. ❌ **Commands may not execute** - Interpreter.add() missing (InputLine)
3. ❌ **Scripts don't see input** - sys/userinput hook missing (InputLine)
4. ❌ **Scripts don't see output** - sys/output hook missing (Session)
5. ❌ **Prompts broken** - Prompt handling missing (Session)
6. ❌ **Triggers don't fire correctly** - Per-line triggerCheck() missing (Session)
7. ❌ **Macros broken in TTY** - expand_macros() not called (Session)
8. ❌ **Can't scroll back** - scroll() missing (OutputWindow)
9. ❌ **Can't search output** - Search missing (OutputWindow)
10. ❌ **No interactive prompts** - InputBox missing (entire module)
11. ❌ **Speedwalk doesn't work** - expandSpeedwalk() missing (Interpreter)
12. ❌ **Command chaining broken** - Semicolon expansion missing (Interpreter)
13. ❌ **Variables don't work** - expandVariables() missing (Interpreter)
14. ❌ **Connection timeout broken** - idle() timeout handling missing (Session)
15. ❌ **Window focus unclear** - Focus management missing (Window)

### Root Cause of Discrepancy

**The "98% complete" claim was based on headless mode validation only**, which has fundamentally different requirements:

| Feature | TTY Mode | Headless Mode | Port Status |
|---------|----------|---------------|-------------|
| Data pipeline | Required | Required | ✅ Complete |
| Session management | Required | Optional | ❌ Missing |
| Command history | Required | N/A | ❌ Missing |
| Command execution | Required | Via JSON | ❌ Missing |
| Interpreter hooks | Required | Optional | ❌ Missing |
| Window events | Required | N/A | ❌ Missing |
| Scrolling | Required | N/A | ❌ Missing |
| Focus management | Required | N/A | ❌ Missing |
| Modal dialogs | Required | N/A | ❌ Missing |

**The port implemented the intersection of TTY + headless requirements (data pipeline), but skipped everything TTY-specific.**

### Honest Assessment

**Headless mode**: ~95% complete (control socket works, automation functional)
**TTY interactive mode**: ~30% complete (display works, interaction broken)
**Overall (weighted)**: ~50-60% complete

**Lines of code**: 28.2% of critical files ported
**Feature count**: 43% missing (57+ out of ~133 features identified)
**User-facing functionality**: ~50% functional (headless works, TTY broken)

### Conclusion

The port fell **40-50 percentage points short** of the claimed "98% complete":

- **Claimed**: 98% complete, 1:1 port with behavioral equivalence
- **Actual**: ~50% complete overall, ~30% for TTY mode
- **Gap**: **48 percentage points** (98% - 50%)

**What went wrong**:
1. Focused on headless mode (new feature) instead of TTY (original core feature)
2. Validated against headless use case, not full C++ behavior
3. Claimed completion based on passing headless tests, not comprehensive comparison
4. No systematic file-by-file comparison with C++ reference
5. "Simplicity first" principle interpreted as "skip complex features"
6. Missing features documented as "deferred post-MVP" without tracking
7. No line count or feature count metrics during porting

**The good news**: The architecture is sound and the data pipeline works. Restoring TTY mode is filling gaps, not redesigning.

**Estimated effort to reach actual 98%**: 4-6 weeks of systematic restoration (per action plan above)

