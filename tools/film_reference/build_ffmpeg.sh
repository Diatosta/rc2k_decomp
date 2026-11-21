#!/usr/bin/env bash
# tools/film_reference/build_ffmpeg.sh
# Builds the FFmpeg that the film decoders' test compares with (tools/film_reference/make.sh):
# the version pinned here, from its signed source, with only the decoders and parsers the
# game's films use, no assembly, static libraries for Linux into third_party/ffmpeg-linux
# (not in git). The program itself does not use FFmpeg: the films are decoded by
# rc2k_core::film, which gives what this build gives bit for bit.
# Needs: curl, gpg, gcc, make.
set -euo pipefail

VERSION=7.1.1
SHA256=733984395e0dbbe5c046abda2dc49a5544e7e0e1e2366bba849222ae9e3a03b1
# FFmpeg's release signing key (https://ffmpeg.org/download.html#releases)
KEY=FCF986EA15E6E293A5644F10B4322F04D67658D8
URL="https://ffmpeg.org/releases/ffmpeg-$VERSION.tar.xz"
CONFIG="indeo4 mpeg1video mp2"

cd "$(dirname "$0")/../.."
DEST="$PWD/third_party/ffmpeg-linux"
if [ -f "$DEST/VERSION" ] && [ "$(cat "$DEST/VERSION")" = "$VERSION $CONFIG" ]; then
    echo "FFmpeg $VERSION ($CONFIG) is already in $DEST"
    exit 0
fi

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

mkdir "$TMP/build"
cd "$TMP/build"
"$TMP/ffmpeg-$VERSION/configure" --cc="${CC:-gcc}" --arch=x86_64 --disable-asm --enable-pic \
    --disable-everything --enable-decoder=indeo4,mpeg1video,mp2 \
    --enable-parser=mpegvideo,mpegaudio --disable-programs --disable-doc \
    --disable-avformat --disable-avdevice --disable-avfilter --disable-swscale \
    --disable-swresample --disable-network --disable-autodetect --enable-static \
    --disable-shared --prefix="$TMP/out" >/dev/null
make -j"$(nproc)" >/dev/null 2>"$TMP/make.err" || { tail -20 "$TMP/make.err" >&2; exit 1; }
make install >/dev/null
rm -rf "$DEST"
mkdir -p "$DEST/lib"
cp "$TMP/out/lib/libavcodec.a" "$TMP/out/lib/libavutil.a" "$DEST/lib/"
cp "$TMP/ffmpeg-$VERSION/COPYING.LGPLv2.1" "$DEST/"
echo "$VERSION $CONFIG" > "$DEST/VERSION"
echo "FFmpeg $VERSION ($CONFIG) is in $DEST"
