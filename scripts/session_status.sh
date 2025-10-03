#!/bin/bash
# Check if a session is running and show connection info

INSTANCE=${1:-nodeka}
SOCK="/tmp/okros/${INSTANCE}.sock"
DEBUG_LOG="/tmp/${INSTANCE}_debug.log"
STDOUT_LOG="/tmp/${INSTANCE}_stdout.log"

echo "Session: $INSTANCE"
echo "==================="

# Check socket
if [ -S "$SOCK" ]; then
  echo "Status: RUNNING"
  echo "Socket: $SOCK"
else
  echo "Status: NOT RUNNING"
  echo "Socket: $SOCK (not found)"
  exit 1
fi

echo

# Try to get info from control protocol
echo "Control Protocol Response:"
STATUS=$(echo '{"cmd":"status"}' | nc -U "$SOCK" 2>/dev/null || echo '{"event":"NoStatusCommand"}')
echo "$STATUS" | jq . 2>/dev/null || echo "$STATUS"

echo

# Check log files
echo "Log Files:"
if [ -f "$DEBUG_LOG" ]; then
  SIZE=$(wc -l < "$DEBUG_LOG")
  echo "  Debug: $DEBUG_LOG ($SIZE lines)"
else
  echo "  Debug: $DEBUG_LOG (not found)"
fi

if [ -f "$STDOUT_LOG" ]; then
  SIZE=$(wc -l < "$STDOUT_LOG")
  echo "  Stdout: $STDOUT_LOG ($SIZE lines)"
else
  echo "  Stdout: $STDOUT_LOG (not found)"
fi

echo

# Check if process is running
PIDS=$(pgrep -f "okros.*--instance ${INSTANCE}")
if [ -n "$PIDS" ]; then
  echo "Process IDs: $PIDS"
else
  echo "Warning: No process found (orphaned socket?)"
fi
