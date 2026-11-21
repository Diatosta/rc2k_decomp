#!/usr/bin/env bash
# Puts a game folder (files, var, savedata) into the app's private storage (adb run-as: a package built
# with `tools/android/build.sh --debuggable`; the release build refuses it):
#   tools/android/push_game.sh <game folder>      (an emulator or phone on adb)
set -euo pipefail
source "$(dirname "$0")/env.sh"
src=${1:?the game folder}
pkg=org.rc2k.rally
adb shell run-as $pkg mkdir -p files/game
for part in files var savedata; do
    [ -e "$src/$part" ] || continue
    tar -C "$src" -chf - "$part" | adb shell "run-as $pkg tar -C files/game -xf -"
done
adb shell run-as $pkg ls files/game
