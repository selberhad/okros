#!/bin/bash
# Gracefully stop a headless session

INSTANCE=${1:-nodeka}
SOCK="/tmp/okros/${INSTANCE}.sock"

if [ ! -S "$SOCK" ]; then
  echo "Session $INSTANCE is not running (socket not found)" >&2
  exit 1
fi

echo "Stopping headless session: $INSTANCE"
echo '{"cmd":"quit"}' | nc -U "$SOCK" 2>/dev/null

# Wait a moment for cleanup
sleep 0.5

# Check if socket is gone
if [ ! -S "$SOCK" ]; then
  echo "Session stopped successfully"
else
  echo "Warning: Socket still exists, session may not have stopped cleanly" >&2
  exit 1
fi
