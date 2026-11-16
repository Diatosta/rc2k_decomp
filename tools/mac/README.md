# The macOS build

Three ways to the Mac program:

- **On a Mac**: `./build.sh mac` (Intel and Apple silicon joined by `lipo` into
  `Rally Championship.app`), or from another machine through SSH (`MAC_HOST`).
- **On Linux, crossed** (below): Intel (`x86_64-apple-darwin`, `env.sh x64`) and Apple
  silicon (`aarch64-apple-darwin`, `env.sh a64`), joined by `llvm-lipo` into the bundle.
- **GitHub Actions**: a macOS runner builds it as on a Mac, for `main` and runs started by
  hand (Mac minutes count ten times on a private repository).

## Crossed on Linux

Needs clang, lld (`ld64.lld`), llvm-ar / llvm-ranlib / llvm-install-name-tool, cmake,
bsdtar / bsdcpio, and Rust's target (`rustup target add x86_64-apple-darwin`).

1. Apple's SDK and clang's Mac runtime (`libclang_rt.osx.a`, which `@available` needs),
   from the Command Line Tools in Apple's software update catalogue. They are Apple's, under
   the Xcode and Apple SDKs Agreement:
   `tools/mac/cross/fetch_tools.sh --accept-apple-license` (into `~/.cache/rc2k_mac`, or
   `RC2K_MAC_TOOLS`). Written from the steps that fetched them by hand on 2026-11-08; the
   script itself not yet run.
2. FFmpeg's decoders: `SDKROOT=... tools/build_ffmpeg.sh mac-x64` and `mac-a64` (into
   `third_party/ffmpeg-mac-x64`, `third_party/ffmpeg-mac-a64`).
3. The program, for each processor (`x64` / `x86_64-apple-darwin`, `a64` /
   `aarch64-apple-darwin`; each in its own shell, `env.sh` sets the target's linker):

   ```sh
   source tools/mac/cross/env.sh x64
   CARGO_TARGET_DIR=target/mac cargo build --release --target x86_64-apple-darwin --bin rc2k
   ```
4. The bundle, as `build.sh mac` makes it on a Mac:

   ```sh
   APP="target/mac/Rally Championship.app"
   mkdir -p "$APP/Contents/MacOS"
   llvm-lipo -create -output "$APP/Contents/MacOS/rc2k" \
       target/mac/x86_64-apple-darwin/release/rc2k target/mac/aarch64-apple-darwin/release/rc2k
   cp tools/mac/Info.plist "$APP/Contents/"
   ```

   SDL and FFmpeg are linked in: the program loads only the system's frameworks and
   libraries. `ld64.lld` signs the arm64 slice ad hoc (linker-signed, as Apple's linker
   does; Apple silicon runs nothing unsigned); the bundle as a whole is not signed on Linux
   (`codesign --force --sign -` on a Mac seals it), nor notarised.

`cc.sh` is the C compiler and linker (clang for the Mac, `ld64.lld`, the runtime),
`toolchain.cmake` SDL's CMake toolchain. SDL is built without its camera (`Cargo.toml`): the
game has no use for one.

## Tested in Darling

[Darling](https://github.com/darlinghq/darling) runs Mac programs on Linux. Built from its
source (stock components, `TARGET_i386=OFF`) with a prefix of its own (`DPREFIX`).

A test build differs from the shipped one only in its linking (`RC2K_MAC_WEAK_FRAMEWORKS=1`,
`cc.sh`):

- every framework weak: Darling's lack some of what SDL imports (most of GameController,
  CoreHaptics, AudioQueue, some display mode calls); the program starts and a missing piece
  is null, met only if used;
- `darling_shim.c` stands in for Darling's broken calls (found 2026-11-08):
  `clock_get_time` on the calendar clock never returns (SDL reads the wall clock through
  it); `__exp10` / `__exp10f` and `__sincos_stret` / `__sincosf_stret` call themselves
  (their source is `pow(10, x)` or sin and cos, which clang folds back into the same call).

```sh
source tools/mac/cross/env.sh x64
RC2K_MAC_WEAK_FRAMEWORKS=1 CARGO_TARGET_DIR=target/mac-darling \
    cargo build --release --target x86_64-apple-darwin --bin rc2k --features remote
```

Run headless with `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy RC2K_RENDERER=software`, and
SDL's controller back ends off (`SDL_JOYSTICK_MFI=0 SDL_JOYSTICK_HIDAPI=0
SDL_JOYSTICK_IOKIT=0 SDL_HAPTIC=0`: they raise Objective-C exceptions on Darling's stubs).
`tools/remote.py` drives it with `RC2K_REMOTE_DARLING=<prefix>` (it starts the game through
`darling shell`, with those switches). Notes:

- Darling sees this machine's files under `/Volumes/SystemRoot`; keep the game's folder
  outside the prefix (files written into the prefix's overlay from Linux are not seen inside)
  and give it its own `files` (a symlink's target would be read inside Darling).
- `timeout` does not reach the program through `darling shell`: end it by its own pid (its
  command line is `./rc2k`).
- A held race (`RC2K_HELD_RACE`, `RC2K_SIM_HASH`, `RC2K_SOUND_HASH`) gives the same hashes
  as the Linux build's with the same `var` and `savedata` (Langton, 1,995 ticks, 2026-11-09).
