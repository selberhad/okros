#!/bin/bash
# Get last N lines from scrollback buffer

INSTANCE=${1:-nodeka}
NUM_LINES=${2:-10}
SOCK="/tmp/okros/${INSTANCE}.sock"

if [ ! -S "$SOCK" ]; then
  echo "Error: Socket $SOCK does not exist" >&2
  exit 1
fi

echo "{\"cmd\":\"recent_lines\",\"data\":\"$NUM_LINES\"}" | nc -U "$SOCK"
