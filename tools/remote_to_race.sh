#!/bin/bash
# Starts the remote build straight into a single rally's stage (the game's launch switches, no
# menus): remote_to_race.sh [rally] [stage], both counted from 0 in the stage screen's order (from
# Wales, its first stage). 3 4 is Langton, the Jim Clark rally's fifth and the game's shortest;
# 4 0 is Hamilton's Folly, in rain; 0 4 is Dyfi, in snow. Other arguments go to `remote.py start`.
# PLAYERS=2 in the environment: a two-player race (split screen).
set -euo pipefail
cd "$(dirname "$0")/.." || exit 1
rally=0 stage=0
if [[ ${1:-} =~ ^[0-9]+$ ]]; then rally=$1; shift; fi
if [[ ${1:-} =~ ^[0-9]+$ ]]; then stage=$1; shift; fi
python3 tools/remote.py start "$@" -- --stage "$((rally + 1))-$((stage + 1))" --players "${PLAYERS:-1}"
# Until the race runs and has drawn 20 frames; at most 120 s (a handheld loads slower).
for _ in $(seq 240); do
    state=$(python3 tools/remote.py state || true)
    drawn=$(grep -o 'screen_frames=[0-9]*' <<<"$state" | cut -d= -f2 || true)
    if grep -q 'race=1' <<<"$state" && [ "${drawn:-0}" -ge 20 ]; then
        python3 tools/remote.py state
        exit 0
    fi
    sleep 0.5
done
echo "remote_to_race: no race" >&2
exit 1
