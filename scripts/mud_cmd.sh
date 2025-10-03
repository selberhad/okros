#!/bin/bash
# Send command to MUD and show response
SOCK=${1:?Usage: $0 <socket> <command>}
CMD="$2"
printf '{"cmd":"sock_send","data":"%s\\n"}\n' "$CMD" | nc -U "$SOCK"
sleep 2
echo '{"cmd":"get_buffer"}' | nc -U "$SOCK" | jq -r '.lines[]' | tail -5
