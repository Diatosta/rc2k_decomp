#!/usr/bin/env bash
# tools/build_ffmpeg.sh
# Builds FFmpeg's decoder library for the standalone program's films (`src/film.rs`):
# the version pinned here, from its signed source, for 32-bit Windows with clang-cl and
# the Windows SDK that cargo-xwin keeps (~/.cache/cargo-xwin/xwin), only the decoders the
# films need (ALLOGO.AVI: Indeo 4; MFLOGO.MPG, INTRO.MPG: MPEG-1 video and MPEG audio
# layer 2) and their parsers, no assembly (no nasm needed), shared (LGPL 2.1). Into
# third_party/ffmpeg (not in git): bin/avcodec-61.dll, bin/avutil-59.dll and their import
# libraries bin/avcodec.lib, bin/avutil.lib. `build_ffmpeg.sh x64` builds the same for
# 64-bit Windows into third_party/ffmpeg-x64.
# Needs: curl, gpg, clang-cl, lld-link, llvm-lib, llvm-nm, make; a cargo xwin build run
# once (for the SDK).
set -euo pipefail

VERSION=7.1.1
SHA256=733984395e0dbbe5c046abda2dc49a5544e7e0e1e2366bba849222ae9e3a03b1
# FFmpeg's release signing key (https://ffmpeg.org/download.html#releases)
KEY=FCF986EA15E6E293A5644F10B4322F04D67658D8
URL="https://ffmpeg.org/releases/ffmpeg-$VERSION.tar.xz"
# What the build turns on, part of what is checked to say it is already built.
CONFIG="indeo4 mpeg1video mp2"

ARCH="${1:-x86}"
case "$ARCH" in
    x86) TRIPLE=i686-pc-windows-msvc; FFARCH=x86; SUFFIX="" ;;
    x64) TRIPLE=x86_64-pc-windows-msvc; FFARCH=x86_64; SUFFIX="-x64" ;;
    *) echo "usage: build_ffmpeg.sh [x86|x64]" >&2; exit 1 ;;
esac

cd "$(dirname "$0")/.."
DEST="$PWD/third_party/ffmpeg$SUFFIX"

if [ -f "$DEST/VERSION" ] && [ "$(cat "$DEST/VERSION")" = "$VERSION $CONFIG" ]; then
    echo "FFmpeg $VERSION ($CONFIG) is already in $DEST"
    exit 0
fi

XWIN="${XWIN_CACHE:-$HOME/.cache/cargo-xwin/xwin}"
[ -d "$XWIN/crt/lib/$FFARCH" ] || { echo "no Windows SDK for $FFARCH in $XWIN: run a cargo xwin build for it first" >&2; exit 1; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

curl -sSfL -o "$TMP/ffmpeg.tar.xz" "$URL"
curl -sSfL -o "$TMP/ffmpeg.tar.xz.asc" "$URL.asc"
curl -sSfL -o "$TMP/ffmpeg-devel.asc" "https://ffmpeg.org/ffmpeg-devel.asc"
echo "$SHA256  $TMP/ffmpeg.tar.xz" | sha256sum -c - >/dev/null
mkdir -m 700 "$TMP/gnupg"
gpg --homedir "$TMP/gnupg" --quiet --import "$TMP/ffmpeg-devel.asc" 2>/dev/null
gpg --homedir "$TMP/gnupg" --status-fd 1 --verify "$TMP/ffmpeg.tar.xz.asc" "$TMP/ffmpeg.tar.xz" 2>/dev/null \
    | grep -q "VALIDSIG $KEY" || { echo "FFmpeg's signature does not check out" >&2; exit 1; }

tar xf "$TMP/ffmpeg.tar.xz" -C "$TMP"
SRC="$TMP/ffmpeg-$VERSION"

# FFmpeg's makedef calls lib.exe by name: llvm-lib stands for it; NM=llvm-nm makes it
# read the symbols with llvm-nm instead of dumpbin.
mkdir "$TMP/shim"
printf '#!/bin/sh\nexec llvm-lib "$@"\n' > "$TMP/shim/lib.exe"
chmod +x "$TMP/shim/lib.exe"

mkdir "$TMP/build"
cd "$TMP/build"
CFLAGS="--target=$TRIPLE -imsvc $XWIN/crt/include -imsvc $XWIN/sdk/include/ucrt -imsvc $XWIN/sdk/include/um -imsvc $XWIN/sdk/include/shared"
LDFLAGS="-libpath:$XWIN/crt/lib/$FFARCH -libpath:$XWIN/sdk/lib/um/$FFARCH -libpath:$XWIN/sdk/lib/ucrt/$FFARCH"
"$SRC/configure" --toolchain=msvc --cc=clang-cl --cxx=clang-cl --ld=lld-link --ar=llvm-lib \
    --nm=llvm-nm --target-os=win32 --arch=$FFARCH --enable-cross-compile --disable-asm \
    --disable-everything --enable-decoder=indeo4,mpeg1video,mp2 \
    --enable-parser=mpegvideo,mpegaudio --disable-programs --disable-doc \
    --disable-avformat --disable-avdevice --disable-avfilter --disable-swscale \
    --disable-swresample --disable-network --disable-autodetect --enable-shared \
    --disable-static --extra-cflags="$CFLAGS" --extra-ldflags="$LDFLAGS" \
    --prefix="$TMP/out" >/dev/null
PATH="$TMP/shim:$PATH" NM=llvm-nm make -j"$(nproc)" >/dev/null 2>"$TMP/make.err" \
    || { tail -20 "$TMP/make.err" >&2; exit 1; }
PATH="$TMP/shim:$PATH" NM=llvm-nm make install >/dev/null

rm -rf "$DEST"
mkdir -p "$DEST/bin"
cp "$TMP/out/bin/avcodec-61.dll" "$TMP/out/bin/avutil-59.dll" \
    "$TMP/out/bin/avcodec.lib" "$TMP/out/bin/avutil.lib" "$DEST/bin/"
cp "$SRC/COPYING.LGPLv2.1" "$DEST/"
echo "$VERSION $CONFIG" > "$DEST/VERSION"
echo "FFmpeg $VERSION ($CONFIG) is in $DEST"
