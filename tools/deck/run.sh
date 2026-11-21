#!/usr/bin/env bash
# Runs a command in the Steam Deck's build container, on this repository (tools/deck/Dockerfile).
set -euo pipefail
cd "$(dirname "$0")/../.."
# A worktree's third_party is a link into the main checkout: mount where it points too.
extra=()
if [ -L third_party ]; then
    real=$(readlink -f third_party)
    extra=(-v "$real:$real:ro")
fi
# The build folder may be a link to another disk: mount where it points (read-write).
if [ -L target ]; then
    built=$(readlink -f target)
    extra+=(-v "$built:$built")
fi
# A build folder named from outside (CARGO_TARGET_DIR) is mounted too.
target_dir="${CARGO_TARGET_DIR:-target/deck}"
case "$target_dir" in
    /*) mkdir -p "$target_dir"; extra+=(-v "$target_dir:$target_dir") ;;
esac
# CI has no ~/.cargo of its own to point at: HOME is whatever the runner uses, mounted as is.
exec docker run ${extra[@]+"${extra[@]}"} --rm --user "$(id -u):$(id -g)" -e HOME="$HOME" \
    -e CARGO_TARGET_DIR="$target_dir" \
    -e CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C target-cpu=znver2" \
    ${CARGO_BUILD_JOBS:+-e CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS} -e RUSTUP_HOME="$HOME/.rustup" -e CARGO_HOME="$HOME/.cargo" \
    -e PATH="$HOME/.cargo/bin:/usr/bin:/bin" \
    -v "$HOME/.cargo:$HOME/.cargo" -v "$HOME/.rustup:$HOME/.rustup" \
    -v "$PWD:$PWD" -w "$PWD" rc2k-deck "$@"
