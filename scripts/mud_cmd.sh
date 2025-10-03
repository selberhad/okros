#!/bin/bash
# Send command to MUD and show response
SOCK=${1:?Usage: $0 <socket> <command>}
CMD="$2"

# Use jq to properly escape the command for JSON (handles !, @, etc.)
jq -nc --arg cmd "$CMD" '{"cmd":"sock_send","data":($cmd + "\n")}' | nc -U "$SOCK"
sleep 2
echo '{"cmd":"get_buffer"}' | nc -U "$SOCK" | jq -r '.lines[]' | tail -5
