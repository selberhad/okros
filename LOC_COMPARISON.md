# Lines of Code Comparison: C++ MCL → Rust okros

**⚠️ AUTO-GENERATED - DO NOT EDIT MANUALLY**

*This file is automatically updated by the pre-commit hook. To regenerate: `./scripts/generate_loc_report.pl`*

---

## Overall Summary

**Rust is 3% larger than C++** (1.03x the size)

| Metric | C++ (Reference) | Rust (okros) | Ratio |
|--------|----------------|--------------|-------|
| **Code Lines** | 8,815 | 9,038 | **1.03x** |
| **Comments** | 637 | 1,255 | 1.97x |
| **Blank Lines** | 2,219 | 1,325 | 0.60x |
| **TOTAL** | 11,671 | 11,618 | **1.00x** |
| **Files** | 79 | 41 | 0.52x |

**Difference**: +223 lines of code (+2.5%)

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
| InputLine.cc | 522 | input_line.rs | 466 | 0.89 | ✅ |
| Interpreter.cc | 834 | plugins/stack.rs | 258 | 0.31 | ✅ ⚠️ SHORT |
| MUD.cc | 135 | mud.rs | 375 | 2.78 | ✅ |
| Option.cc | 136 | SKIPPED/DEFERRED | - | - | ⏭️ |
| OutputWindow.cc | 339 | output_window.rs | 442 | 1.30 | ✅ |
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
| Window.cc | 721 | window.rs | 349 | 0.48 | ✅ |
| main.cc | 250 | main.rs | 1040 | 4.16 | ✅ |
| misc.cc | 209 | SKIPPED/DEFERRED | - | - | ⏭️ |
| plugins/PerlEmbeddedInterpreter.cc | 260 | plugins/perl.rs | 533 | 2.05 | ✅ |
| plugins/PythonEmbeddedInterpreter.cc | 268 | plugins/python.rs | 427 | 1.59 | ✅ |

---

## ⚠️ Files Requiring Investigation

The following files are suspiciously short (<40% of C++ size) and may be incomplete:

- **Interpreter.cc** (834 lines) → **plugins/stack.rs** (258 lines) = **31%** of original

**See `PORT_GAPS.md` for detailed gap analysis.**

---

## Notes

- **Deferred features**: Chat.cc, Borg.cc, Group.cc (intentional)
- **Using stdlib**: String.cc, Buffer.cc, StaticBuffer.cc replaced by Rust stdlib
- **Missing**: InputBox.cc not ported (needs implementation)
- **Incomplete ports**: See PORT_GAPS.md for comprehensive analysis

---

*Generated: Sat Oct  4 19:31:12 2025*
*Tool: [cloc](https://github.com/AlDanial/cloc) v
*
