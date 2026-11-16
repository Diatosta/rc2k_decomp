#!/bin/bash
# Makes what rc2k_core's film test compares its decoders with: the two MPEG films' pictures and
# samples as FFmpeg decodes them (build_ffmpeg.sh here, into third_party/ffmpeg-linux), cut and
# decoded the way the film player does it.
#   tools/film_reference/make.sh <folder with INTRO.MPG and MFLOGO.MPG> <folder for the dumps>
# Then: RC2K_FILM_FILES=<films> RC2K_FILM_DUMPS=<dumps> cargo test --release -p rc2k_core film::
# The dumps take about 400 MB; they and the films are never committed.
# Needs: gcc, python3, tools/film_reference/build_ffmpeg.sh run once.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
films="${1:?the folder with the films}"
dumps="${2:?the folder for the dumps}"
lib="$here/../../third_party/ffmpeg-linux/lib"
mkdir -p "$dumps"
gcc -O2 -o "$dumps/mpeg_reference" "$here/mpeg_reference.c" "$lib/libavcodec.a" "$lib/libavutil.a" -lm -lpthread
for name in INTRO MFLOGO; do
    # the video stream to <name>.m1v, the sound to <name>.mp2
    python3 -I "$here/demux.py" "$films/$name.MPG" "$dumps/$name"
    "$dumps/mpeg_reference" mpeg1video "$dumps/$name.m1v" "$dumps/$name.yuv" "$dumps/$name.vsizes"
    "$dumps/mpeg_reference" mp2 "$dumps/$name.mp2" "$dumps/$name.s16" "$dumps/$name.asizes"
done
