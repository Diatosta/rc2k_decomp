#!/usr/bin/env bash
# deploy.sh
set -euo pipefail

cd "$(dirname "$0")"

[ -f deploy.local ] && source deploy.local
: "${GAME_DIR:?Set GAME_DIR, e.g. in deploy.local}"

cargo xwin build --release --xwin-arch x86 --target i686-pc-windows-msvc

# --remove-destination replaces the file by unlinking it first, so a game that is
# still running keeps its old copy mapped instead of having it overwritten in place.
cp --remove-destination \
   target/i686-pc-windows-msvc/release/rc2k_hook.dll \
   "$GAME_DIR/rc2k_hook.asi"

# Only seed the config the first time, so edits made in the game folder survive.
[ -e "$GAME_DIR/hook.toml" ] || cp hook.toml "$GAME_DIR/"

echo "deployed"