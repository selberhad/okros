#!/bin/bash
# Start or restart a headless okros session

INSTANCE=${1:-nodeka}
MUD_HOST=${2:-nodeka.com:23}

# Graceful shutdown if running
echo '{"cmd":"quit"}' | nc -U /tmp/okros/${INSTANCE}.sock 2>/dev/null
sleep 0.5

# Start new session
nohup ./target/release/okros --headless --instance ${INSTANCE} \
  </dev/null >/tmp/${INSTANCE}_stdout.log 2>/tmp/${INSTANCE}_debug.log &

# Wait for socket to be ready
sleep 0.5

# Connect to MUD if specified
if [ -n "$MUD_HOST" ]; then
  echo "{\"cmd\":\"connect\",\"data\":\"${MUD_HOST}\"}" | nc -U /tmp/okros/${INSTANCE}.sock
fi

echo "Headless session started: /tmp/okros/${INSTANCE}.sock"
echo "Debug log: /tmp/${INSTANCE}_debug.log"
