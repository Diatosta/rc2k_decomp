#!/usr/bin/env bash
# SDL3's shared library for one Android ABI (x86_64 | arm64), from the SDL source the Rust
# build already fetches (the same version as the crate's), with the NDK's cmake toolchain.
set -euo pipefail
source "$(dirname "$0")/env.sh"
abi=${1:?x86_64 or arm64}
d=$([ "$abi" = arm64 ] && echo arm64-v8a || echo "$abi")
src=$(echo ~/.cargo/registry/src/*/sdl3-src-3.4.18/SDL)
out=$RC2K_ANDROID_HOME/sdl3build/$d
mkdir -p "$out" && cd "$out"
cmake -G Ninja "$src" -DCMAKE_TOOLCHAIN_FILE="$ANDROID_NDK_HOME/build/cmake/android.toolchain.cmake" \
    -DANDROID_ABI="$d" -DANDROID_PLATFORM=android-$API -DCMAKE_BUILD_TYPE=Release \
    -DSDL_SHARED=ON -DSDL_STATIC=OFF -DSDL_TESTS=OFF -DSDL_EXAMPLES=OFF -DSDL_INSTALL=OFF >cmake.log
nice -n 19 ninja -j4 >ninja.log
ls -l libSDL3.so
