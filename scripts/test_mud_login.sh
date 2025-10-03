#!/bin/bash
# Test MUD login using credentials from .env file

set -e

# Load .env if it exists
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
else
    echo "Error: .env file not found. Copy .env.example to .env and fill in credentials."
    exit 1
fi

MUD=${1:-nodeka}
SOCK="/tmp/okros/${MUD}_test.sock"

case "$MUD" in
    nodeka)
        HOST="nodeka.com:23"
        USERNAME="${NODEKA_USERNAME:-}"
        PASSWORD="${NODEKA_PASSWORD:-}"
        ;;
    wotmud)
        HOST="game.wotmud.org:2224"
        USERNAME="${WOTMUD_USERNAME:-}"
        PASSWORD="${WOTMUD_PASSWORD:-}"
        ;;
    *)
        echo "Unknown MUD: $MUD"
        echo "Usage: $0 [nodeka|wotmud]"
        exit 1
        ;;
esac

if [ -z "$USERNAME" ] || [ -z "$PASSWORD" ]; then
    echo "Error: Credentials not set in .env for $MUD"
    exit 1
fi

echo "Testing $MUD login with user: $USERNAME"

# Start headless
./target/release/okros --headless --instance ${MUD}_test &
OKROS_PID=$!
sleep 2

# Connect
echo '{"cmd":"connect","data":"'$HOST'"}' | nc -U $SOCK
sleep 3

# Get initial prompt
echo '{"cmd":"peek","count":5}' | nc -U $SOCK | jq -r '.lines[]' | tail -3

# Shutdown
echo '{"cmd":"quit"}' | nc -U $SOCK

echo "Done"
