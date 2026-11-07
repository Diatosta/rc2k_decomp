#!/usr/bin/env bash
# Runs a command in the handheld's build container, on this repository (tools/anbernic/Dockerfile).
set -euo pipefail
cd "$(dirname "$0")/../.."
# A worktree's third_party is a link into the main checkout: mount where it points too.
extra=()
if [ -L third_party ]; then
    real=$(readlink -f third_party)
    extra=(-v "$real:$real:ro")
fi
exec docker run "${extra[@]}" --rm --user "$(id -u):$(id -g)" -e HOME="$HOME" \
    -e CARGO_TARGET_DIR=target/a64-linux \
    -e CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    -e CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    -e CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
    -e AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar \
    -e CMAKE_SYSTEM_NAME=Linux -e CMAKE_SYSTEM_PROCESSOR=aarch64 \
    -e CARGO_PROFILE_RELEASE_DEBUG=line-tables-only -e CARGO_PROFILE_RELEASE_LTO=fat -e CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
    -e CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C target-cpu=cortex-a53" \
    -e PKG_CONFIG_ALLOW_CROSS=1 -e PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig \
    -e PATH="$HOME/.cargo/bin:/usr/bin:/bin" \
    -v "$HOME/.cargo:$HOME/.cargo" -v "$HOME/.rustup:$HOME/.rustup" \
    -v "$PWD:$PWD" -w "$PWD" rc2k-anbernic "$@"
