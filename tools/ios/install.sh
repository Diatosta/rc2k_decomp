#!/usr/bin/env bash
# Signs dist/ios/'Rally Championship.ipa' with the owner's free Apple ID and installs it on the
# iPhone plugged in by USB (tools/ios/README.md, steps 5 to 7). Needs xtool (logged in by
# `xtool auth login`) and usbmuxd running. Usage: tools/ios/install.sh [path/to.ipa]
set -euo pipefail
cd "$(dirname "$0")/../.."
ipa=${1:-"dist/ios/Rally Championship.ipa"}
[ -f "$ipa" ] || { echo "install.sh: no $ipa: run ./build.sh ios first" >&2; exit 1; }
command -v xtool >/dev/null || { echo "install.sh: no xtool (tools/ios/README.md, step 1)" >&2; exit 1; }
xtool devices
# xtool makes the App ID and the development certificate and profile for the Apple ID, signs
# the app with them and sends it to the phone.
xtool install "$ipa"
