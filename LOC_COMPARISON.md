# Lines of Code Comparison: C++ MCL → Rust okros

**⚠️ AUTO-GENERATED - DO NOT EDIT MANUALLY**

*This file is automatically updated by the pre-commit hook. To regenerate: `./scripts/generate_loc_report.pl`*

---

## Overall Summary

**Rust is 5% more concise than C++** (0.95x the size)

| Metric | C++ (Reference) | Rust (okros) | Ratio |
|--------|----------------|--------------|-------|
| **Code Lines** | 8,815 | 8,335 | **0.95x** |
| **Comments** | 637 | 1,073 | 1.68x |
| **Blank Lines** | 2,219 | 1,153 | 0.52x |
| **TOTAL** | 11,671 | 10,561 | **0.90x** |
| **Files** | 79 | 39 | 0.49x |

**Difference**: -480 lines of code (-5.4%)

---

## File-by-File Comparison

| C++ File | C++ Lines | Rust File | Rust Lines | Ratio | Status |
|----------|-----------|-----------|------------|-------|--------|
| Alias.cc | 257 | alias.rs | 280 | 1.09 | ✅ |
| Borg.cc | 132 | SKIPPED/DEFERRED | - | - | ⏭️ |
| Buffer.cc | 115 | SKIPPED/DEFERRED | - | - | ⏭️ |
| Chat.cc | 1469 | SKIPPED/DEFERRED | - | - | ⏭️ |
| Config.cc | 704 | config.rs | 555 | 0.79 | ✅ |
| Curses.cc | 95 | curses.rs | 208 | 2.19 | ✅ |
| Embedded.cc | 299 | SKIPPED/DEFERRED | - | - | ⏭️ |
| Group.cc | 144 | SKIPPED/DEFERRED | - | - | ⏭️ |
| Hotkey.cc | 93 | macro_def.rs | 54 | 0.58 | ✅ |
| InputBox.cc | 50 | SKIPPED/DEFERRED | - | - | ⏭️ |
| InputLine.cc | 522 | input_line.rs | 461 | 0.88 | ✅ |
| Interpreter.cc | 834 | plugins/stack.rs | 258 | 0.31 | ✅ ⚠️ SHORT |
| MUD.cc | 135 | mud.rs | 375 | 2.78 | ✅ |
| Option.cc | 136 | SKIPPED/DEFERRED | - | - | ⏭️ |
| OutputWindow.cc | 339 | output_window.rs | 87 | 0.26 | ✅ ⚠️ SHORT |
| Pipe.cc | 97 | SKIPPED/DEFERRED | - | - | ⏭️ |
| Screen.cc | 350 | screen.rs | 547 | 1.56 | ✅ |
| Selectable.cc | 47 | selectable.rs | 24 | 0.51 | ✅ |
| Selection.cc | 213 | selection.rs | 350 | 1.64 | ✅ |
| Session.cc | 684 | session.rs | 285 | 0.42 | ✅ |
| Shell.cc | 128 | SKIPPED/DEFERRED | - | - | ⏭️ |
| Socket.cc | 290 | socket.rs | 198 | 0.68 | ✅ |
| StaticBuffer.cc | 110 | SKIPPED/DEFERRED | - | - | ⏭️ |
| StatusLine.cc | 60 | status_line.rs | 73 | 1.22 | ✅ |
| String.cc | 19 | SKIPPED/DEFERRED | - | - | ⏭️ |
| TTY.cc | 297 | tty.rs | 172 | 0.58 | ✅ |
| Window.cc | 721 | window.rs | 286 | 0.40 | ✅ ⚠️ SHORT |
| main.cc | 250 | main.rs | 988 | 3.95 | ✅ |
| misc.cc | 209 | SKIPPED/DEFERRED | - | - | ⏭️ |
| plugins/PerlEmbeddedInterpreter.cc | 260 | plugins/perl.rs | 533 | 2.05 | ✅ |
| plugins/PythonEmbeddedInterpreter.cc | 268 | plugins/python.rs | 427 | 1.59 | ✅ |

---

## ⚠️ Files Requiring Investigation

The following files are suspiciously short (<40% of C++ size) and may be incomplete:

- **OutputWindow.cc** (339 lines) → **output_window.rs** (87 lines) = **26%** of original
- **Interpreter.cc** (834 lines) → **plugins/stack.rs** (258 lines) = **31%** of original
- **Window.cc** (721 lines) → **window.rs** (286 lines) = **40%** of original

**See `PORT_GAPS.md` for detailed gap analysis.**

---

## Notes

- **Deferred features**: Chat.cc, Borg.cc, Group.cc (intentional)
- **Using stdlib**: String.cc, Buffer.cc, StaticBuffer.cc replaced by Rust stdlib
- **Missing**: InputBox.cc not ported (needs implementation)
- **Incomplete ports**: See PORT_GAPS.md for comprehensive analysis

---

*Generated: Sat Oct  4 16:04:01 2025*
*Tool: [cloc](https://github.com/AlDanial/cloc) v
*
