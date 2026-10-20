#!/usr/bin/env bash
# deploy_standalone.sh
# Builds the program that runs the ported code without the game's code (rc2k.exe) and
# copies it, with SDL3.dll and FFmpeg's avcodec and avutil, to the game's folder (its
# `files\` and `var\`; RAL.EXE's own data is built into rc2k.exe since 2026-10-19). That
# folder is STANDALONE_DIR if deploy.local sets it, else the folder above GAME_DIR.
# Start it there: `wine rc2k.exe` (it writes rc2k.log next to itself).
set -euo pipefail

cd "$(dirname "$0")"

[ -f deploy.local ] && source deploy.local
: "${GAME_DIR:?Set GAME_DIR, e.g. in deploy.local}"
DEST="${STANDALONE_DIR:-$GAME_DIR/..}"

./tools/fetch_sdl3.sh
./tools/build_ffmpeg.sh

cargo xwin build --release --xwin-arch x86 --target i686-pc-windows-msvc \
    --features standalone --bin rc2k


cp --remove-destination target/i686-pc-windows-msvc/release/rc2k.exe "$DEST/"
cp --remove-destination third_party/sdl3/lib/x86/SDL3.dll "$DEST/"
cp --remove-destination third_party/ffmpeg/bin/avcodec-61.dll third_party/ffmpeg/bin/avutil-59.dll "$DEST/"

echo "deployed rc2k.exe, SDL3.dll and FFmpeg's two DLLs to $DEST"
