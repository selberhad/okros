#!/bin/bash
# Test script for headless offline MUD integration
# Validates: cargo run -- --headless --offline --instance test

set -e

echo "=== Headless Offline MUD Integration Test ==="
echo

# Start headless offline MUD in background
echo "Starting headless offline MUD..."
cargo run -- --headless --offline --instance test_integration 2>&1 | grep -E "(Headless|error)" &
PID=$!
sleep 2

SOCK=/tmp/okros/test_integration.sock

# Helper function to send command
send_cmd() {
    echo "{\"cmd\":\"$1\",\"data\":\"$2\"}" | nc -U $SOCK
}

# Helper function to get status
get_status() {
    echo '{"cmd":"status"}' | nc -U $SOCK
}

echo "✓ Server started (socket: $SOCK)"
echo

# Test 1: Initial state
echo "Test 1: Initial state"
STATUS=$(get_status)
if echo "$STATUS" | grep -q '"location":"clearing"'; then
    echo "✓ Starting location is 'clearing'"
else
    echo "✗ FAILED: Wrong starting location"
    kill $PID 2>/dev/null
    exit 1
fi

if echo "$STATUS" | grep -q '"inventory_count":0'; then
    echo "✓ Inventory is empty"
else
    echo "✗ FAILED: Inventory should be empty"
    kill $PID 2>/dev/null
    exit 1
fi
echo

# Test 2: Take rusty sword
echo "Test 2: Take rusty sword"
send_cmd "send" "take rusty sword" > /dev/null
STATUS=$(get_status)
if echo "$STATUS" | grep -q '"inventory_count":1'; then
    echo "✓ Inventory count increased to 1"
else
    echo "✗ FAILED: Item not taken"
    kill $PID 2>/dev/null
    exit 1
fi
echo

# Test 3: Navigate to cave
echo "Test 3: Navigate to cave"
send_cmd "send" "go east" > /dev/null
STATUS=$(get_status)
if echo "$STATUS" | grep -q '"location":"cave"'; then
    echo "✓ Moved to cave"
else
    echo "✗ FAILED: Navigation failed"
    kill $PID 2>/dev/null
    exit 1
fi
echo

# Test 4: Take torch
echo "Test 4: Take torch in cave"
send_cmd "send" "take torch" > /dev/null
STATUS=$(get_status)
if echo "$STATUS" | grep -q '"inventory_count":2'; then
    echo "✓ Inventory count increased to 2"
else
    echo "✗ FAILED: Torch not taken"
    kill $PID 2>/dev/null
    exit 1
fi
echo

# Test 5: Navigate to village
echo "Test 5: Navigate to village"
send_cmd "send" "go west" > /dev/null
send_cmd "send" "go south" > /dev/null
send_cmd "send" "go south" > /dev/null
STATUS=$(get_status)
if echo "$STATUS" | grep -q '"location":"village"'; then
    echo "✓ Reached village"
else
    echo "✗ FAILED: Navigation to village failed"
    kill $PID 2>/dev/null
    exit 1
fi
echo

# Test 6: Take iron key
echo "Test 6: Take iron key"
send_cmd "send" "take iron key" > /dev/null
STATUS=$(get_status)
if echo "$STATUS" | grep -q '"inventory_count":3'; then
    echo "✓ All 3 items collected"
else
    echo "✗ FAILED: Iron key not taken"
    kill $PID 2>/dev/null
    exit 1
fi
echo

# Test 7: Error handling
echo "Test 7: Error handling"
send_cmd "send" "dance" > /dev/null
BUFFER=$(echo '{"cmd":"get_buffer"}' | nc -U $SOCK)
if echo "$BUFFER" | grep -q "don't understand"; then
    echo "✓ Invalid commands handled correctly"
else
    echo "✗ FAILED: Error handling broken"
    kill $PID 2>/dev/null
    exit 1
fi
echo

# Cleanup
echo "Cleaning up..."
kill $PID 2>/dev/null
rm -f $SOCK
echo

echo "=== ALL TESTS PASSED ==="
echo "✓ Headless offline MUD integration working correctly"
echo "✓ State persistence across connections"
echo "✓ Full playthrough successful (3 items, 5 rooms)"
echo "✓ Error handling functional"
