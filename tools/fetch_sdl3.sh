#!/usr/bin/env bash
# tools/fetch_sdl3.sh
# Fetches SDL's ready-made library for Visual C (the version pinned here) into
# third_party/sdl3, which is not in git. A build that uses SDL gives the linker
# third_party/sdl3/lib/x86 and puts third_party/sdl3/lib/x86/SDL3.dll next to the program.
set -euo pipefail

VERSION=3.4.18
SHA256=78d84602ae616cfe26b33a73b7d3b9b6b56e8a6650e53d92c9c6ca938f747acd
URL="https://github.com/libsdl-org/SDL/releases/download/release-$VERSION/SDL3-devel-$VERSION-VC.zip"

cd "$(dirname "$0")/.."
DEST=third_party/sdl3

if [ -f "$DEST/VERSION" ] && [ "$(cat "$DEST/VERSION")" = "$VERSION" ]; then
    echo "SDL $VERSION is already in $DEST"
    exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

curl -sSfL -o "$TMP/sdl3.zip" "$URL"
echo "$SHA256  $TMP/sdl3.zip" | sha256sum -c - >/dev/null
unzip -q "$TMP/sdl3.zip" -d "$TMP"

rm -rf "$DEST"
mkdir -p "$DEST"
cp -r "$TMP/SDL3-$VERSION/include" "$TMP/SDL3-$VERSION/lib" "$TMP/SDL3-$VERSION/LICENSE.txt" "$DEST/"
echo "$VERSION" > "$DEST/VERSION"
echo "SDL $VERSION is in $DEST"
