#!/bin/bash
# Starts the remote build and goes through the menus into a single rally's stage:
# remote_to_race.sh [rally] [stage], both counts of Right on the stage screen (from Wales, its
# first stage). 3 4 is Langton, the Jim Clark rally's fifth and the game's shortest; 4 0 is
# Hamilton's Folly, in rain. Other arguments go to `remote.py start`.
cd "$(dirname "$0")/.." || exit 1
rally=0 stage=0
[[ $1 =~ ^[0-9]+$ ]] && rally=$1 && shift
[[ $1 =~ ^[0-9]+$ ]] && stage=$1 && shift
press() {
    for _ in $(seq "$2"); do
        python3 tools/remote.py key "$1" 100 >/dev/null
        sleep "$3"
    done
}
python3 tools/remote.py start "$@"
sleep 9
press Return 6 3
press Right "$rally" 1.2
press Down 1 1
press Right "$stage" 1.2
press Return 1 16
python3 tools/remote.py state
