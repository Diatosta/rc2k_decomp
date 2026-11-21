#!/bin/bash
# Starts the game from its own folder: for a Steam Deck's "Add a Non-Steam Game" entry (point it at
# this file), or from a terminal. libsteam_api.so beside rc2k lets the game use Steam's network play.
set -eu
cd "$(dirname "$(readlink -f "$0")")" || exit 1
exec ./rc2k "$@"
