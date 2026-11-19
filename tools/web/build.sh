#!/bin/bash
# The web build (`notes/improvements.md`, "Web build"): the game for WebAssembly through
# Emscripten, its threads on the browser's workers. Needs the Emscripten SDK in ~/emsdk and the
# nightly toolchain with `wasm32-unknown-emscripten` and `rust-src` (std is rebuilt with atomics:
# the threads share one memory). Output: target/web/wasm32-unknown-emscripten/release/rc2k.{js,wasm}
# (`--debug` first: an unoptimised build, in .../debug). `./build.sh web` runs this and gathers
# the page's files in dist/web/.
set -e
cd "$(dirname "$0")/../.."
profile=--release
if [ "${1:-}" = --debug ]; then profile=; shift; fi
source ~/emsdk/emsdk_env.sh >/dev/null 2>&1
link=(
  -pthread
  -sUSE_SDL=3
  # Files on the origin's private storage (`src/platform/web.rs`).
  -sWASMFS
  # The game blocks in its own loops: it runs on a worker, not the page's thread.
  -sPROXY_TO_PTHREAD
  -sALLOW_MEMORY_GROWTH=1 -sMAXIMUM_MEMORY=2GB -sINITIAL_MEMORY=256MB
  -sSTACK_SIZE=8MB -sDEFAULT_PTHREAD_STACK_SIZE=4MB
  -sPTHREAD_POOL_SIZE=8
  -sEXIT_RUNTIME=1
  -sEXPORTED_RUNTIME_METHODS=ENV,FS -sFORCE_FILESYSTEM=1
  # SDL's canvas update runs on the page's thread and uses `Browser`.
  -sDEFAULT_LIBRARY_FUNCS_TO_INCLUDE='$Browser'
  # WebGL 2 for the GL renderer (`src/render/gles.rs`), its context made on the GL thread and
  # drawn into an off-screen frame buffer that the frame's commit copies onto the page's canvas;
  # the entry points are found by name.
  # The page's side of the window and display (`library.js`).
  --js-library="$PWD/tools/web/library.js"
  -sMAX_WEBGL_VERSION=2 -sOFFSCREEN_FRAMEBUFFER -sGL_ENABLE_GET_PROC_ADDRESS
)
flags="-C target-feature=+atomics,+bulk-memory,+mutable-globals"
for a in "${link[@]}"; do flags="$flags -C link-arg=$a"; done
RUSTFLAGS="$flags" CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/web}" nice cargo +nightly build $profile \
  -Zbuild-std=std,panic_abort --target wasm32-unknown-emscripten --bin rc2k "$@"
