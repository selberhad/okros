#!/bin/bash
# Run cargo tests with a pseudo-TTY (required for ncurses tests)
# Uses the 'script' command to create a pseudo-terminal

set -e

echo "Running tests with pseudo-TTY support..."
echo ""

# Ensure TERM is set to something ncurses can work with
export TERM="${TERM:-xterm-256color}"

# macOS uses 'script -q /dev/null command'
# Linux uses 'script -qec "command" /dev/null'
# Detect OS and use appropriate syntax

if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    # Note: ncurses tests may still skip if the terminal database isn't accessible
    # in the pseudo-TTY environment. This is expected - tests are defensive.
    script -q /dev/null cargo test "$@"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    script -qec "cargo test $*" /dev/null
else
    echo "Unsupported OS: $OSTYPE"
    echo "Falling back to normal cargo test (ncurses tests may skip)"
    cargo test "$@"
fi

echo ""
echo "Note: ncurses tests (curses::tests::*) require a real TTY with terminfo database."
echo "If they skip, that's expected. To run them interactively, use:"
echo "  cargo test --lib curses::tests -- --nocapture"
echo "from a real terminal session."
