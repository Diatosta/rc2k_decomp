#!/bin/bash
# Starts the remote build and goes through the menus into a single rally's first stage.
cd "$(dirname "$0")/.." || exit 1
python3 tools/remote.py start "$@"
sleep 9
for _ in 1 2 3 4 5 6 7 8; do
    python3 tools/remote.py key Return 100 >/dev/null
    sleep 2.5
    python3 tools/remote.py state | grep -q 'race=1' && break
done
sleep 14
python3 tools/remote.py state
