#!/bin/bash
# Test Nodeka connection in TTY mode and show debug output

echo "Cleaning debug log..."
rm -f /tmp/okros_debug.log

echo "Building (suppressing warnings)..."
cargo build 2>&1 | grep -E "^(error|Finished|Compiling okros)" || true

echo ""
echo "Connecting to Nodeka (you'll see ncurses screen for 5 seconds)..."
echo "Press any key after to continue..."
sleep 1

OKROS_CONNECT=nodeka.com:23 timeout 5 cargo run 2>/dev/null || true

echo ""
echo "========================================"
echo "=== Debug Output from /tmp/okros_debug.log ==="
echo "========================================"
if [ -f /tmp/okros_debug.log ]; then
    cat /tmp/okros_debug.log
else
    echo "ERROR: No debug log found!"
    echo "This means the debug code didn't run."
    echo "Check if ncurses screen appeared at all."
fi
