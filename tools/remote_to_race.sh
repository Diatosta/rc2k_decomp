#!/bin/bash
# Starts the remote build and goes through the menus into a single rally's stage:
# remote_to_race.sh [rally] [stage], both counts of Right on the stage screen (from Wales, its
# first stage). 3 4 is Langton, the Jim Clark rally's fifth and the game's shortest; 4 0 is
# Hamilton's Folly, in rain. Other arguments go to `remote.py start`. PLAYERS=2 in the
# environment: a two-player race (split screen).
cd "$(dirname "$0")/.." || exit 1
rally=0 stage=0
[[ $1 =~ ^[0-9]+$ ]] && rally=$1 && shift
[[ $1 =~ ^[0-9]+$ ]] && stage=$1 && shift
# Each step waits for the screen it needs (the handheld draws its menus slower than the PC).
press() {
    for _ in $(seq "$2"); do
        python3 tools/remote.py key "$1" 100 >/dev/null
        sleep "$3"
    done
}
# Until `remote.py state` shows $1 (a pattern) on a screen that has drawn 20 frames (so it
# takes keys), pressing $2 each $3 s until it is there; at most 40 tries.
until_state() {
    for _ in $(seq 40); do
        state=$(python3 tools/remote.py state)
        drawn=$(grep -o 'screen_frames=[0-9]*' <<<"$state" | cut -d= -f2)
        if grep -q -- "$1" <<<"$state"; then
            # There: no more keys, only until it has drawn enough.
            [ "${drawn:-0}" -ge 20 ] && return 0
            sleep 0.5
            continue
        fi
        [ -n "$2" ] && python3 tools/remote.py key "$2" 100 >/dev/null
        sleep "$3"
    done
    echo "remote_to_race: \"$1\" not reached" >&2
    exit 1
}
python3 tools/remote.py start "$@"
# Through the title, the main menu and the single rally's pages to the stage screen (menu 1,
# the rally row item 2).
until_state 'menu=None' '' 1
# PLAYERS=2: two players (split screen), chosen on the players' screen (menu 4).
if [ "${PLAYERS:-1}" = 2 ]; then
    until_state 'menu=Some(MenuId(4))' Return 3
    press Right 1 1.5
fi
until_state 'menu=Some(MenuId(1)) item=2' Return 3
press Right "$rally" 1.5
until_state 'item=3' Down 1.5
press Right "$stage" 1.5
python3 tools/remote.py key Return 100 >/dev/null
until_state 'race=1' '' 2
python3 tools/remote.py state
