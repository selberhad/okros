#!/bin/bash
# Get the current scrollback buffer from a headless session

INSTANCE=${1:-nodeka}
SOCK="/tmp/okros/${INSTANCE}.sock"

if [ ! -S "$SOCK" ]; then
  echo "Error: Socket $SOCK does not exist" >&2
  exit 1
fi

echo '{"cmd":"get_buffer"}' | nc -U "$SOCK"
