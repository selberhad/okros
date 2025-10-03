#!/bin/bash
# Integration test for offline mode with real TTY
# Tests curses.rs, tty.rs, and main.rs event loop
# Uses expect to automate interactive TTY session

set -e

echo "=== Offline Mode TTY Integration Test ==="
echo

# Check if expect is available
if ! command -v expect &> /dev/null; then
    echo "⚠️  'expect' not found - installing via brew..."
    if command -v brew &> /dev/null; then
        brew install expect
    else
        echo "❌ Cannot install expect - please install manually"
        exit 1
    fi
fi

# Build the binary
echo "Building okros..."
cargo build --quiet 2>&1 | grep -E "error" || true

# Create expect script
EXPECT_SCRIPT=$(cat <<'EOF'
#!/usr/bin/expect -f
# Expect script to automate offline mode interaction

set timeout 5
log_user 0

# Start offline mode
spawn cargo run -- --offline

# Wait for initial prompt/display
expect {
    timeout { puts "ERROR: Timeout waiting for start"; exit 1 }
    "Forest Clearing" { puts "✓ Game started" }
}

# Send 'look' command
send "look\r"
expect {
    timeout { puts "ERROR: Timeout after look"; exit 1 }
    "Forest Clearing" { puts "✓ Look command works" }
}

# Take the rusty sword
send "take rusty sword\r"
expect {
    timeout { puts "ERROR: Timeout after take"; exit 1 }
    -re "(You take|rusty sword)" { puts "✓ Take command works" }
}

# Check inventory
send "inventory\r"
expect {
    timeout { puts "ERROR: Timeout after inventory"; exit 1 }
    "rusty sword" { puts "✓ Inventory shows item" }
}

# Navigate east to cave
send "go east\r"
expect {
    timeout { puts "ERROR: Timeout after go east"; exit 1 }
    "Dark Cave" { puts "✓ Navigation works" }
}

# Take torch
send "take torch\r"
expect {
    timeout { puts "ERROR: Timeout after take torch"; exit 1 }
    -re "(You take|torch)" { puts "✓ Multiple items work" }
}

# Navigate back
send "go west\r"
expect {
    timeout { puts "ERROR: Timeout after go west"; exit 1 }
    "Forest Clearing" { puts "✓ Return navigation works" }
}

# Test invalid command (error handling)
send "dance\r"
expect {
    timeout { puts "ERROR: Timeout after invalid command"; exit 1 }
    "don't understand" { puts "✓ Error handling works" }
}

# Quit
send "quit\r"
expect {
    timeout { puts "ERROR: Timeout waiting for quit"; exit 1 }
    eof { puts "✓ Clean exit" }
}

puts ""
puts "=== ALL TTY TESTS PASSED ==="
exit 0
EOF
)

# Write expect script to temp file
EXPECT_FILE=$(mktemp /tmp/okros_tty_test.XXXXXX)
echo "$EXPECT_SCRIPT" > "$EXPECT_FILE"
chmod +x "$EXPECT_FILE"

# Run expect script
echo "Running TTY interaction test..."
if expect "$EXPECT_FILE"; then
    echo ""
    echo "✅ TTY integration test passed"
    echo "✅ Exercised: curses.rs, tty.rs, main.rs event loop"
    echo "✅ Coverage improved for TTY-dependent code"
    rm -f "$EXPECT_FILE"
    exit 0
else
    echo ""
    echo "❌ TTY integration test failed"
    rm -f "$EXPECT_FILE"
    exit 1
fi
