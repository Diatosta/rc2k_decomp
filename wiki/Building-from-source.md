# Building from source

*Category: [Building and running](Building-and-running)*

How to build the program, `rc2k`, for each system it runs on. To play you also need the
game's own files from your copy of *Rally Championship 2000*
([Installing the game](Installing-the-game)). Nothing here needs the original program.

All commands are run from the repository's top folder. The repository and the crate are named
`rc2k_decomp`; the program it builds is `rc2k`.

## What you need

- **Rust**, through `rustup`. The file `rust-toolchain.toml` pins the version and names the
  targets; `rustup` installs them the first time `cargo` runs here.
- A C and C++ compiler, `make` and `cmake`: SDL and the debug window's library are built from
  source as part of the build on Linux and macOS.
- `curl` and `gpg`: the FFmpeg script downloads FFmpeg's source and checks its signature.

Two libraries are not in the repository. Scripts in `tools/` fetch or build them into
`third_party/`, which git ignores:

| Library | Used for | How it gets there |
| --- | --- | --- |
| SDL3 | window, input, sound output, GPU drawing | Windows: `tools/fetch_sdl3.sh` downloads SDL's ready-made library (a pinned version, checked by its hash). Other systems: built from source by cargo, with no script. On Linux this needs the development packages of the system's video and sound; the list for Ubuntu is in `.github/workflows/build.yml`. |
| FFmpeg (decoders only) | the three films | `tools/build_ffmpeg.sh [x86\|x64\|linux\|a64\|mac]` builds a small FFmpeg with only the decoders the films need. With no argument it builds for 32-bit Windows. |

## Linux

```sh
tools/build_ffmpeg.sh linux
cargo build --release --target x86_64-unknown-linux-gnu --bin rc2k
```

The program is `target/x86_64-unknown-linux-gnu/release/rc2k`. SDL and FFmpeg are linked into
it, so it needs nothing installed beyond the C library.

(The project's own scripts put the Linux build in a folder of its own by setting
`CARGO_TARGET_DIR=target/linux`; the program is then
`target/linux/x86_64-unknown-linux-gnu/release/rc2k`. Either way works.)

## Windows, built on Linux

The Windows builds are cross-compiled with
[cargo-xwin](https://github.com/rust-cross/cargo-xwin), which downloads the Windows SDK it
needs. Besides the tools above you need LLVM's `clang-cl`, `lld-link`, `llvm-lib` and
`llvm-nm` (Clang 19 or later).

```sh
cargo install --locked cargo-xwin
tools/fetch_sdl3.sh

# 64 bits
cargo xwin cache xwin --xwin-arch x86_64
tools/build_ffmpeg.sh x64
cargo xwin build --release --xwin-arch x86_64 --target x86_64-pc-windows-msvc --bin rc2k

# 32 bits (no debug window)
cargo xwin cache xwin --xwin-arch x86
tools/build_ffmpeg.sh
cargo xwin build --release --xwin-arch x86 --target i686-pc-windows-msvc --bin rc2k
```

The `cargo xwin cache` line fetches the Windows SDK, which the FFmpeg script uses. Without it
the script stops with "no Windows SDK ...: run a cargo xwin build for it first".

`rc2k.exe` needs three DLLs beside it:

| Build | Program | DLLs |
| --- | --- | --- |
| 64-bit | `target/x86_64-pc-windows-msvc/release/rc2k.exe` | `third_party/sdl3/lib/x64/SDL3.dll`, `third_party/ffmpeg-x64/bin/avcodec-61.dll`, `third_party/ffmpeg-x64/bin/avutil-59.dll` |
| 32-bit | `target/i686-pc-windows-msvc/release/rc2k.exe` | `third_party/sdl3/lib/x86/SDL3.dll`, `third_party/ffmpeg/bin/avcodec-61.dll`, `third_party/ffmpeg/bin/avutil-59.dll` |

## Windows, built on Windows

**This script has not yet been tried on a real Windows machine.**

You need:

- `rustup`;
- Visual Studio 2022, or its Build Tools, with "Desktop development with C++";
- MSYS2, with `pacman -S --needed make diffutils gnupg tar xz unzip curl`;
- a checkout without line-ending conversion (the repository's `.gitattributes` asks git for
  that).

Then, in PowerShell, from the repository's top folder:

```powershell
powershell -ExecutionPolicy Bypass -File tools\windows\build.ps1 -Arch x64
powershell -ExecutionPolicy Bypass -File tools\windows\build.ps1 -Arch x86 -GameDir C:\Games\Rally
```

| Switch | Meaning |
| --- | --- |
| `-Arch x86` or `-Arch x64` | 32 or 64 bits. Without it, 32 bits. |
| `-GameDir <folder>` | Also copy `rc2k.exe` and the three DLLs into the game's folder (it must hold `files`). |
| `-Msys <folder>` | Where MSYS2 is, if not `C:\msys64`. |

The script finds Visual Studio's C++ tools, fetches SDL, builds FFmpeg's decoders with Visual
Studio's own compiler, and runs `cargo build --release` for the target. The result is in
`target\<target>\release`, with the DLLs beside `rc2k.exe`, ready to run.

## macOS

On a Mac with the Command Line Tools, `rustup` with both `apple-darwin` targets, `cmake` and
`gnupg`:

```sh
./deploy_standalone.sh mac        # both processors in one bundle
./deploy_standalone.sh mac x64    # Intel only
./deploy_standalone.sh mac a64    # Apple silicon only
```

This builds FFmpeg and the program, joins the two processors' programs with `lipo` into
`Rally Championship.app`, signs it ad hoc (it is not notarised) and copies it into the game's
folder: `~/rally`, or the folder named in the environment variable `MAC_GAME_DIR`.

`tools/mac/README.md` describes the other ways: from another machine over SSH, and
cross-compiled on Linux.

## The Anbernic RG34XXSP handheld

Needs Docker. The build runs in a container that matches the handheld's stock system
(`tools/anbernic/Dockerfile`); `tools/anbernic/run.sh` runs a command inside it.

```sh
docker build -t rc2k-anbernic tools/anbernic
tools/anbernic/run.sh tools/build_ffmpeg.sh a64
tools/anbernic/run.sh cargo build --release --target aarch64-unknown-linux-gnu --bin rc2k
```

The program is `target/a64-linux/aarch64-unknown-linux-gnu/release/rc2k`.
`./deploy_standalone.sh anbernic` does these steps and copies the result to the device over
SSH. `tools/anbernic/README.md` has the details, including how the game's files get onto the
device the first time.

## The web build (experimental)

Needs the Emscripten SDK in `~/emsdk` and a nightly Rust toolchain with the
`wasm32-unknown-emscripten` target and `rust-src`:

```sh
tools/web/build.sh
python3 tools/web/serve.py [folder] [port]
```

The output is `target/web/wasm32-unknown-emscripten/release/rc2k.js` and `rc2k.wasm`; the page
is `tools/web/index.html`. This build starts but does not show a picture yet.

## Build and copy in one step: `deploy_standalone.sh`

`deploy_standalone.sh` runs the commands above and copies the result where it is wanted.

| Command | Builds | Copies to |
| --- | --- | --- |
| `./deploy_standalone.sh` | Windows, 32 bits | the game's folder: `rc2k.exe` and the three DLLs |
| `./deploy_standalone.sh x64` | Windows, 64 bits | `x64/` in the game's folder |
| `./deploy_standalone.sh linux` | Linux | `linux/` in the game's folder; it also prints the glibc version the build needs |
| `./deploy_standalone.sh mac [x64\|a64]` | macOS | `MAC_GAME_DIR` |
| `./deploy_standalone.sh anbernic [pgo]` | the handheld | the device, over SSH |

### `deploy.local`

The script reads its settings from a file named `deploy.local` in the repository's top
folder. You write it yourself; git ignores it. It is a list of shell variables. An example
with made-up values, showing every variable the scripts read:

```sh
# Where the game is. The program goes into STANDALONE_DIR; if that is left out,
# into the folder above GAME_DIR. GAME_DIR must always be set.
GAME_DIR=/home/sam/games/rally/bin
STANDALONE_DIR=/home/sam/games/rally

# The handheld (deploy_standalone.sh anbernic, tools/anbernic/pgo.sh)
ANBERNIC_HOST=root@192.0.2.20
ANBERNIC_KEY=/home/sam/.ssh/handheld_key          # default: ~/.ssh/rg34xxsp
ANBERNIC_GAME=/userdata/roms/ports/rally          # default: this folder

# A Mac reached over SSH (deploy_standalone.sh mac, run on another machine)
MAC_HOST=sam@192.0.2.30
MAC_KEY=/home/sam/.ssh/mac_key                    # default: ~/.ssh/rc2k_mac
MAC_PORT=22                                       # default: 22
MAC_SRC=rc2k_src                                  # the source's folder on the Mac; default: rc2k_src
MAC_GAME_DIR=/Users/sam/rally                     # the game's folder on the Mac; default: ~/rally
```

Only `GAME_DIR` is needed for the Windows and Linux builds; `mac` on a Mac itself needs none
of them.

One more variable is read from the environment, not from the file: `RC2K_FEATURES` adds cargo
features to the Mac build (for example `RC2K_FEATURES=remote` for a test build).

`STANDALONE_DIR` is the game's folder: the one that holds `files` and `var`.

### A profile-guided handheld build

`tools/anbernic/pgo.sh` builds an instrumented program, drives it on the handheld to collect a
profile, and writes `target/a64-pgo/rc2k.profdata`. `./deploy_standalone.sh anbernic pgo` then
builds with that profile. It needs `llvm-profdata` of the same LLVM version as the Rust
compiler.

## Running it

Start the program **in the game's folder**: it looks for `files` and `var` in the current
folder.

```sh
cd /home/sam/games/rally
linux/rc2k
```

On Windows start `rc2k.exe` there (or `x64\rc2k.exe`); under Wine, `wine rc2k.exe`.

The program writes its log, `rc2k.log`, in the current folder. If something goes wrong at
start-up, the reason is there. Switches, options and environment variables are on
[Running and options](Running-and-options); a quick way to try a build is to start straight
into a race:

```sh
linux/rc2k --stage langton --then quit
```

## The automatic builds

The GitHub workflow `.github/workflows/build.yml` builds Windows (64 bits), Linux and macOS
with the same commands. The Mac build runs only for the main branch and for runs started by
hand.

## Next

[Testing](Testing) lists the checks to run on a change.
