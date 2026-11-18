# Building from source

*Category: [Building and running](Building-and-running)*

How to build the program, `rc2k`, for each system it runs on. To play you also need the
game's own files from your copy of *Rally Championship 2000*
([Installing the game](Installing-the-game)). Nothing here needs the original program.

All commands are run from the repository's top folder. The repository and the crate are named
`rc2k_decomp`; the program it builds is `rc2k`.

## One command per target

| Target | Command | Run it on | Result |
| --- | --- | --- | --- |
| Linux, x86-64 | `./build.sh linux` | Linux | `dist/linux/rc2k` |
| Windows, 64 bits | `./build.sh windows-x64` | Linux | `dist/windows-x64/`: `rc2k.exe`, `SDL3.dll` |
| Windows, 32 bits | `./build.sh windows` | Linux | `dist/windows/`: the same two files |
| Windows, 64 or 32 bits | `build.ps1 -Target windows-x64` or `-Target windows` | Windows | the same two folders |
| macOS | `./build.sh mac` | a Mac, or Linux with a Mac reached over SSH | `dist/mac/Rally Championship.app` |
| iPhone / iPad (begun, not yet run) | `./build.sh ios` | Linux, with Apple's iOS SDK from Xcode (`tools/ios/README.md`) | `dist/ios/Rally Championship.ipa`, unsigned |
| Anbernic RG34XXSP | `./build.sh handheld` | Linux, with Docker | `dist/handheld/`: `rc2k`, `Rally.sh` |
| Web (experimental) | `./build.sh web` | Linux, with Emscripten | `dist/web/`: `rc2k.js`, `rc2k.wasm`, `index.html`, `unzip.js`, `serve.py` |

`./build.sh` with no target lists them. The switches:

| Switch | Meaning |
| --- | --- |
| `--release` | An optimised build. This is the default. |
| `--debug` | An unoptimised build. |
| `--deploy` | After building, copy `dist/<target>/` where `deploy.local` says ([below](#deploying)). |
| `--arch x64` or `--arch a64` | `mac` only: build for Intel or for Apple silicon alone. |
| `--pgo` | `handheld` only: build with a recorded profile ([below](#a-profile-guided-handheld-build)). |

The layout is the same for every target: `dist/<target>/` holds the program and everything it
needs beside it. Copy the folder's contents into the game's folder (the one that holds `files`
and `var`) and the game runs from there. Each build empties its `dist/<target>/` first. Git
ignores `dist/`. Every target's folder also has `locale/`, the game's own texts of each language
(`assets/locale/*.rc2k.toml`): the installer copies the chosen language's into the game, and
`--deploy` copies the installed language's.

When a tool is missing, the script stops and names it. The sections below say what each
target needs and what the script does for it, so that the steps can also be run by hand.

## What you need

- **Rust**, through `rustup`. The file `rust-toolchain.toml` pins the version and names the
  targets; `rustup` installs them the first time `cargo` runs here.
- A C and C++ compiler, `make` and `cmake`: SDL and the debug window's library are built from
  source as part of the build on Linux and macOS.
- `curl` and `unzip` for the Windows builds: SDL's ready-made library is downloaded.

One library is not in the repository, SDL3; it is fetched or built into `third_party/`, which
git ignores. The films are decoded in Rust (`rc2k_core/src/film/`), so no media library is
needed:

| Library | Used for | How it gets there |
| --- | --- | --- |
| SDL3 | window, input, sound output, GPU drawing | Windows: `tools/fetch_sdl3.sh` downloads SDL's ready-made library (a pinned version, checked by its hash). Other systems: built from source by cargo, with no script. On Linux this needs the development packages of the system's video and sound; the list for Ubuntu is in `.github/workflows/build.yml`. |

`build.sh` and `build.ps1` fetch it themselves.

## Linux

```sh
./build.sh linux
```

Needs cmake and the development packages named above. The script runs:

```sh
CARGO_TARGET_DIR=target/linux cargo build --release --target x86_64-unknown-linux-gnu --bin rc2k
```

and copies the program to `dist/linux/rc2k`. SDL is linked into it, so it needs
nothing installed beyond the C library; the script prints the glibc version the build needs.

## Windows, built on Linux

```sh
./build.sh windows-x64    # 64 bits
./build.sh windows        # 32 bits (no debug window)
```

The Windows builds are cross-compiled with
[cargo-xwin](https://github.com/rust-cross/cargo-xwin) (`cargo install --locked cargo-xwin`),
which downloads the Windows SDK it needs. Besides the tools above you need `unzip` and LLVM's
`clang-cl`, `lld-link`, `llvm-lib` and `llvm-nm` (Clang 19 or later). The script runs, for 64
bits:

```sh
tools/fetch_sdl3.sh
CARGO_TARGET_DIR=target/x64 cargo xwin build --release --xwin-arch x86_64 --target x86_64-pc-windows-msvc --bin rc2k
```

and for 32 bits the same with `--xwin-arch x86 --target i686-pc-windows-msvc` (cargo's
folder: `target`).

The button pictures in `assets/glyphs/` (atlases and indexes, built into the program) are made from
Kenney's Input Prompts pack by `python3 tools/glyphs/pack.py <the pack's folder>` (needs Pillow);
they are committed, so building needs neither the script nor the pack.

The program carries `assets/icon.ico` as its icon: `build.rs` writes it as a resource file and
the linker takes it in, so no resource compiler is needed. Then it gathers the two files:

| Build | Program | DLLs |
| --- | --- | --- |
| 64-bit | `target/x64/x86_64-pc-windows-msvc/release/rc2k.exe` | `third_party/sdl3/lib/x64/SDL3.dll` |
| 32-bit | `target/i686-pc-windows-msvc/release/rc2k.exe` | `third_party/sdl3/lib/x86/SDL3.dll` |

cargo-xwin keeps the SDK of one processor at a time in its cache (`~/.cache/cargo-xwin`), and
fetches the other's again when the build changes between 32 and 64 bits. Do not run a 32-bit
and a 64-bit build at the same moment from one cache: give one of them its own with
`XDG_CACHE_HOME=<folder> ./build.sh windows-x64`.

## Windows, built on Windows

**This script has not yet been tried on a real Windows machine.**

You need:

- `rustup`;
- Visual Studio 2022, or its Build Tools, with "Desktop development with C++";
- a checkout without line-ending conversion (the repository's `.gitattributes` asks git for
  that).

Then, in PowerShell, from the repository's top folder:

```powershell
powershell -ExecutionPolicy Bypass -File build.ps1 -Target windows-x64
powershell -ExecutionPolicy Bypass -File build.ps1 -Target windows -GameDir C:\Games\Rally
```

| Switch | Meaning |
| --- | --- |
| `-Target windows` or `-Target windows-x64` | 32 or 64 bits. Without it, 32 bits. |
| `-Deploy` | Also copy `dist\<target>\` into the game's folder: the one `-GameDir` names, or else the `DEPLOY_WINDOWS` line (`DEPLOY_WINDOWS_X64` for 64 bits) of `deploy.local`. |
| `-GameDir <folder>` | The game's folder. Naming it deploys there, with or without `-Deploy`. |
| `-DebugBuild` | An unoptimised build. |

The script finds Visual Studio's C++ tools, downloads SDL's ready-made library itself (the
version and SHA-256 pinned in `tools/fetch_sdl3.sh`; a download with another hash is refused),
and runs `cargo build` for the target inside Visual Studio's developer environment. The
result is `dist\windows\` or `dist\windows-x64\`, as on Linux. Only Windows' own tools are
used.

## macOS

On a Mac with the Command Line Tools, `rustup` with both `apple-darwin` targets and `cmake`:

```sh
./build.sh mac               # both processors in one bundle
./build.sh mac --arch x64    # Intel only
./build.sh mac --arch a64    # Apple silicon only
```

This builds the program, joins the two processors' programs with `lipo` into
`dist/mac/Rally Championship.app` and signs it ad hoc (it is not notarised). The bundle's
icon is `assets/icon.icns`, made from `assets/icon.png` by `tools/mac/make_icns.py`. With `--deploy`
the bundle is copied into the game's folder: `~/rally`, or the folder named by `MAC_GAME_DIR`.

On Linux the same command builds on a Mac over SSH: it copies the source to the Mac that
`MAC_HOST` in `deploy.local` names, runs `./build.sh mac` there and brings the bundle back
into `dist/mac/`. `tools/mac/README.md` describes cross-compiling on Linux by hand.

## iPhone and iPad

`./build.sh ios` crosses from Linux with clang, lld and Apple's iOS SDK (taken out of Xcode's
`.xip`, which needs an Apple ID to download), and makes an unsigned `.ipa`; `tools/ios/install.sh`
signs it with a free Apple ID (xtool) and installs it on a phone plugged in by USB, for seven
days at a time. The game's folder is copied into the app's Documents folder with the Files
app. `tools/ios/README.md` lists every step. The iOS port is begun: it is checked to compile,
runs on a phone, with the touch screen as menus' taps and a race's on-screen gamepad (see
[Controllers](Controllers); a game controller works too), and plays a copied game folder where it is
(no installer page). The window is drawn in
the screen's own pixels; the race starts at that size (cut to at most 1920 wide), the graphics page
lists it with its fractions, and the race frame covers the whole screen, edge to edge, with the HUD and the mirror kept inside
the part clear of the notch and the home indicator; a menu's 4:3 page is as tall as the screen and
centred across that part.

## The Anbernic RG34XXSP handheld

```sh
./build.sh handheld
```

Needs Docker. The build runs in a container that matches the handheld's stock system
(`tools/anbernic/Dockerfile`); `tools/anbernic/run.sh` runs a command inside it. The script
runs:

```sh
docker build -t rc2k-anbernic tools/anbernic
tools/anbernic/run.sh cargo build --release --target aarch64-unknown-linux-gnu --bin rc2k
```

and puts the program, stripped, in `dist/handheld/rc2k` with the device's menu entry
`Rally.sh` beside it. The program with its line tables stays at
`target/a64-linux/aarch64-unknown-linux-gnu/release/rc2k`. `--deploy` copies both files to the
device over SSH. `tools/anbernic/README.md` has the details, including how the game's files
get onto the device the first time.

## The web build (experimental)

Needs the Emscripten SDK in `~/emsdk` and a nightly Rust toolchain with the
`wasm32-unknown-emscripten` target and `rust-src`:

```sh
./build.sh web
python3 dist/web/serve.py dist/web [port]
```

The script runs `tools/web/build.sh` (the cargo step with Emscripten's link switches; its
output is `target/web/wasm32-unknown-emscripten/release/rc2k.js` and `rc2k.wasm`) and gathers
those two with the page (`tools/web/index.html`, `unzip.js`, `serve.py`) and the saves page (`saves.html`, `saves.js`) in `dist/web/`. This
build starts but does not show a picture yet.

## Deploying

`./build.sh <target> --deploy` builds and then copies `dist/<target>/` where it is wanted:

| Target | Copies to |
| --- | --- |
| `linux` | the folder `DEPLOY_LINUX` names; without the line, `~/.local/share/rally-championship` |
| `windows` | the folder `DEPLOY_WINDOWS` names |
| `windows-x64` | the folder `DEPLOY_WINDOWS_X64` names |
| `web` | the folder `DEPLOY_WEB` names |
| `handheld` | the device `ANBERNIC_HOST` names, over SSH: `rc2k` into the game's folder there, `Rally.sh` beside that folder |
| `mac` | on a Mac, `MAC_GAME_DIR` (default `~/rally`); from Linux, the same folder on the Mac |

A folder is made if it is not there. Files of the same names are replaced, everything else in
the folder is left alone. If the line a target needs is missing, the script stops and names
the line.

### `deploy.local`

The scripts read their settings from a file named `deploy.local` in the repository's top
folder. You write it yourself; git ignores it. It is a list of shell variables, one line per
target. An example with made-up values, showing every variable the build scripts read:

```sh
# Where --deploy copies each target (normally the game's folder: the one with files and var).
DEPLOY_LINUX=/home/sam/games/rally                # default: ~/.local/share/rally-championship
DEPLOY_WINDOWS=/home/sam/games/rally-win
DEPLOY_WINDOWS_X64=/home/sam/games/rally-win/x64  # started from the game's folder: x64/rc2k.exe
DEPLOY_WEB=/srv/www/rally

# The handheld (build.sh handheld --deploy, tools/anbernic/pgo.sh, tools/remote.py --handheld)
ANBERNIC_HOST=root@192.0.2.20
ANBERNIC_KEY=/home/sam/.ssh/handheld_key          # default: ~/.ssh/rg34xxsp (remote.py has none)
ANBERNIC_GAME=/userdata/roms/ports/rally          # default: this folder

# A Mac reached over SSH (build.sh mac, run on another system)
MAC_HOST=sam@192.0.2.30
MAC_KEY=/home/sam/.ssh/mac_key                    # default: ~/.ssh/rc2k_mac
MAC_PORT=22                                       # default: 22
MAC_SRC=rc2k_src                                  # the source's folder on the Mac; default: rc2k_src
MAC_GAME_DIR=/Users/sam/rally                     # the game's folder on the Mac; default: ~/rally
```

No line is needed to build; each `--deploy` needs only its own target's.

One more variable is read from the environment, not from the file: `RC2K_FEATURES` adds cargo
features to a build (for example `RC2K_FEATURES=remote ./build.sh linux` for a test build).
Steam network play ([Network play](Network-play)) is in every desktop build (Windows, Linux, macOS; not the
handheld): Valve's library is not linked but loaded while the program runs, when it is there, so the same build
starts and plays with or without it. `RC2K_STEAM=1 ./build.sh <target>` is for a release that offers Steam: it
copies Valve's library (`libsteam_api.so`, `steam_api.dll` / `steam_api64.dll`, `libsteam_api.dylib`) next to the
program, from the Steamworks SDK's `redistributable_bin` folder (`STEAM_SDK_LOCATION` names the SDK's folder; the
copy `steamworks-sys` 0.13.0 leaves in cargo's registry is used when it is not set). The program is written against
the interface versions of that SDK (see `src/net/steam/flat.rs`), so ship that SDK's library.

### The older script: `deploy_standalone.sh`

`deploy_standalone.sh` is kept as a short wrapper, so that older commands still work:

| Older command | Runs |
| --- | --- |
| `./deploy_standalone.sh` | `./build.sh windows --deploy` |
| `./deploy_standalone.sh x64` | `./build.sh windows-x64 --deploy` |
| `./deploy_standalone.sh linux` | `./build.sh linux --deploy` |
| `./deploy_standalone.sh mac [x64\|a64]` | `./build.sh mac --deploy [--arch x64\|a64]` |
| `./deploy_standalone.sh anbernic [pgo]` | `./build.sh handheld --deploy [--pgo]` |

It also keeps the older `deploy.local` lines working. Where the file has no `DEPLOY_` line for
the target, the wrapper takes the game's folder from `STANDALONE_DIR` (or, without it, the
folder above `GAME_DIR`) and copies the 32-bit Windows program into it, the 64-bit one into
its `x64` folder and the Linux one into its `linux` folder, as it always did.

### A profile-guided handheld build

`tools/anbernic/pgo.sh` builds an instrumented program, drives it on the handheld to collect a
profile, and writes `target/a64-pgo/rc2k.profdata`. `./build.sh handheld --pgo` then
builds with that profile. It needs `llvm-profdata` of the same LLVM version as the Rust
compiler.

## Running it

Start the program **in the game's folder**: it looks for `files` and `var` in the current
folder.

```sh
cd /home/sam/games/rally
./rc2k
```

On Windows start `rc2k.exe` there (or `x64\rc2k.exe`); under Wine, `wine rc2k.exe`.

The program writes its log, `rc2k.log`, in the current folder. If something goes wrong at
start-up, the reason is there. Switches, options and environment variables are on
[Running and options](Running-and-options); a quick way to try a build is to start straight
into a race:

```sh
./rc2k --stage langton --then quit
```

## The automatic builds

The GitHub workflow `.github/workflows/build.yml` builds Windows (64 bits), Linux and macOS
with the same commands. The Mac build runs only for the main branch and for runs started by
hand.

## Next

[Testing](Testing) lists the checks to run on a change.
