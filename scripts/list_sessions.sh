#!/bin/bash
# List all running headless sessions

SOCK_DIR="/tmp/okros"

if [ ! -d "$SOCK_DIR" ]; then
  echo "No sessions running (directory $SOCK_DIR does not exist)"
  exit 0
fi

SOCKS=$(find "$SOCK_DIR" -name "*.sock" 2>/dev/null)

if [ -z "$SOCKS" ]; then
  echo "No sessions running"
  exit 0
fi

echo "Running headless sessions:"
echo "-------------------------"

for SOCK in $SOCKS; do
  INSTANCE=$(basename "$SOCK" .sock)
  echo "  - $INSTANCE"
  echo "    Socket: $SOCK"

  # Try to get status
  STATUS=$(echo '{"cmd":"status"}' | nc -U "$SOCK" 2>/dev/null)
  if [ -n "$STATUS" ]; then
    echo "    Status: $STATUS"
  fi
  echo
done
