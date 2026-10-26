#!/usr/bin/env bash
# deploy_standalone.sh
# Builds the program that runs the ported code without the game's code (rc2k.exe) and
# copies it, with SDL3.dll and FFmpeg's avcodec and avutil, to the game's folder (its
# `files\` and `var\`; RAL.EXE's own data is built into rc2k.exe since 2026-10-19). That
# folder is STANDALONE_DIR if deploy.local sets it, else the folder above GAME_DIR.
# Start it there: `wine rc2k.exe` (it writes rc2k.log next to itself).
# `deploy_standalone.sh x64` deploys the 64-bit build instead (into an x64 folder), and
# `deploy_standalone.sh linux` the native Linux one (into a linux folder).
set -euo pipefail

cd "$(dirname "$0")"

[ -f deploy.local ] && source deploy.local
: "${GAME_DIR:?Set GAME_DIR, e.g. in deploy.local}"
DEST="${STANDALONE_DIR:-$GAME_DIR/..}"

if [ "${1:-}" = linux ]; then
    # The native Linux build (phase 11), on the system's SDL3 and FFmpeg, into a linux
    # folder in the game's folder. Started from the game's folder: `linux/rc2k`.
    CARGO_TARGET_DIR=target/linux cargo build --release --target x86_64-unknown-linux-gnu \
        --bin rc2k
    mkdir -p "$DEST/linux"
    cp --remove-destination target/linux/x86_64-unknown-linux-gnu/release/rc2k "$DEST/linux/"
    echo "deployed the Linux rc2k to $DEST/linux"
    exit 0
fi

./tools/fetch_sdl3.sh

if [ "${1:-}" = x64 ]; then
    # The 64-bit build, into an x64 folder beside the 32-bit one (its DLLs have the same
    # names). Started from the game's folder, which holds `files\` and `var\`:
    # `wine x64/rc2k.exe`.
    ./tools/build_ffmpeg.sh x64
    CARGO_TARGET_DIR=target/x64 cargo xwin build --release --xwin-arch x86_64 \
        --target x86_64-pc-windows-msvc --bin rc2k
    mkdir -p "$DEST/x64"
    cp --remove-destination target/x64/x86_64-pc-windows-msvc/release/rc2k.exe "$DEST/x64/"
    cp --remove-destination third_party/sdl3/lib/x64/SDL3.dll "$DEST/x64/"
    cp --remove-destination third_party/ffmpeg-x64/bin/avcodec-61.dll \
        third_party/ffmpeg-x64/bin/avutil-59.dll "$DEST/x64/"
    echo "deployed the 64-bit rc2k.exe and its DLLs to $DEST/x64"
    exit 0
fi

./tools/build_ffmpeg.sh

cargo xwin build --release --xwin-arch x86 --target i686-pc-windows-msvc --bin rc2k


cp --remove-destination target/i686-pc-windows-msvc/release/rc2k.exe "$DEST/"
cp --remove-destination third_party/sdl3/lib/x86/SDL3.dll "$DEST/"
cp --remove-destination third_party/ffmpeg/bin/avcodec-61.dll third_party/ffmpeg/bin/avutil-59.dll "$DEST/"

echo "deployed rc2k.exe, SDL3.dll and FFmpeg's two DLLs to $DEST"
