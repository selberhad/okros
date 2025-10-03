#!/bin/bash
# Continuously watch MUD output (like tail -f)

INSTANCE=${1:-nodeka}
NUM_LINES=${2:-20}
INTERVAL=${3:-1}
SOCK="/tmp/okros/${INSTANCE}.sock"

if [ ! -S "$SOCK" ]; then
  echo "Error: Socket $SOCK does not exist" >&2
  exit 1
fi

echo "Watching $INSTANCE (Ctrl+C to stop)..."
echo "----------------------------------------"

# Get initial buffer
echo "{\"cmd\":\"recent_lines\",\"data\":\"$NUM_LINES\"}" | nc -U "$SOCK"

# Poll for updates
while true; do
  sleep "$INTERVAL"
  # Use recent_lines with a small number to catch new output
  OUTPUT=$(echo "{\"cmd\":\"recent_lines\",\"data\":\"5\"}" | nc -U "$SOCK")
  if [ -n "$OUTPUT" ]; then
    echo "$OUTPUT"
  fi
done
