# Rally Championship 2000 Decomp

This project was written with AI coding assistants (Claude) under the maintainer's
direction. The port was compared with the original game function by function, and later
changes are checked with recorded races and replays and in play on PCs and the handheld.

A reimplementation in Rust of *Rally Championship 2000* (developed by Magnetic Fields; the
program `RAL.EXE`). It is a complete program of its own, `rc2k`: none of the original's code runs.
It reads the game's data files from the player's own copy of the game. No part of the
game is included here.

## How it came about

The original was ported one function at a time. Each Rust function was loaded into the
running game in place of the original one and compared with it, until every function
the game reaches was ported and the two behaved alike. The hooks were then
removed and the code converted, module by module, into typed and mostly safe Rust with
nothing left that depends on the original's memory layout, on 32 bits or on Windows. The
platform side (DirectDraw, Direct3D, DirectSound, DirectInput, DirectPlay, the film player)
was not ported one to one: SDL3, a GPU renderer and FFmpeg stand in for it.

The game itself is meant to stay the same for the player: the same menus, physics, stages,
replays and save files. Where the original had bugs, they are fixed (see below).

## What works

- The whole game: menus, every stage, single rallies, championships, the arcade mode,
  split screen, replays, pace notes, the intro films, 3D sound, controllers.
- Network play, standalone to standalone. The original's DirectPlay is replaced by a
  protocol of our own, so it does not play against the original program.
- The installer and uninstaller, working from the files of the CD (or of a download of
  it).
- The developers' debug pages, brought back as a window over the game (Ctrl+F11; Ctrl+F12
  freezes a race). Not in the 32-bit Windows build or on the handheld.
- Old replays and save files are read as they are.

Platforms:

| Platform | State |
| --- | --- |
| Windows, 64 and 32 bits | Built and run (under Wine / Proton during development). |
| Linux, x86-64 | Built and run. One program with SDL and FFmpeg linked in; it needs only glibc. |
| macOS, Intel and Apple silicon | Builds as `Rally Championship.app`. Tested less than the others. |
| Anbernic RG34XXSP (handheld, aarch64 Linux) | Built and run on Knulli and on the stock system. See [tools/anbernic/README.md](tools/anbernic/README.md). |
| Web (WebAssembly) | Experimental: it builds and starts, but does not show a picture yet. |

Races are drawn on the GPU where one can be started (SDL's GPU device on the desktop,
OpenGL ES on the handheld), otherwise by a software rasteriser. `RC2K_RENDERER=software`
forces the rasteriser.

Measured figures:

- On a desktop, a comparison with the original under the same conditions
  had both hold the frame rate; with the GPU renderer this program used between a seventh
  (Linux) and a third (Windows under Proton) of the original's processor time.
- On the handheld (four Cortex-A53 cores, Mali-G31) a night stage ran at about 47 frames
  a second and a stage in rain at about 42 (computer driver).

## Requirements

| Platform | Needs |
| --- | --- |
| Windows, 64 or 32 bits | Windows 10 or later (the minimum of the pinned Rust toolchain's Windows targets). |
| Linux, x86-64 | glibc only; which version depends on the system the program was built on. The workflow's builds are made on Ubuntu 24.04 (glibc 2.38); a build made on a current rolling distribution needed 2.43. `deploy_standalone.sh linux` prints the version its build needs. |
| macOS | macOS 11.0 (Big Sur) or later, Intel or Apple silicon. |
| Anbernic RG34XXSP | Knulli or the stock system; glibc 2.35 or later. |

Graphics: on the desktop the GPU renderer goes through SDL's GPU device with Vulkan
(Windows, Linux) or Metal (macOS). Where no such device starts, the software rasteriser
draws the game and needs no GPU. On the handheld the GPU renderer needs OpenGL ES 3.0
(the system's `libmali`), with the same fallback.

### Deliberate differences from the original

- No registry: the installation's record is the file `var\install.cfg` in the game's
  folder, so an installed folder can be moved or copied.
- The installer has no page for the kind of installation (it is always the full one), no
  DirectX set-up and no registration; it makes ordinary shortcuts.
- The installer takes the game from the folder it is started in, in place of asking for
  the CD.
- Network play uses its own protocol (see above).
- "Uninstall Rally" runs the installed program itself (`rc2k -u`).

## Installing

You need the files of the game's CD, with its `FILES` folder, on your disk.

1. Get the program for your system: a build from this repository's releases, or one you
   built yourself (below).
2. Put it into the folder that holds the CD's files:
   - Windows: `rc2k.exe`, `SDL3.dll`, `avcodec-61.dll` and `avutil-59.dll`;
   - Linux: `rc2k`;
   - macOS: `Rally Championship.app`.
3. Start it there. Found among the CD's files with no installation, it opens the game's
   installer: choose the language and the folder, and it copies the game, itself included,
   adds the menu entries ("Rally", "Uninstall Rally", "View Readme") and starts the
   installed copy.

The folder offered by default:

| System | Folder |
| --- | --- |
| Windows | chosen on the installer's drive and folder pages |
| Linux | `$XDG_DATA_HOME/rally-championship` (usually `~/.local/share/rally-championship`) |
| macOS | `~/Library/Application Support/Rally Championship` |

Afterwards, start the game from the menu entry, or run `rc2k` in the installed folder.
The program writes its log to `rc2k.log` in the folder it runs in. To remove the game, use
"Uninstall Rally" (it asks whether to keep the saved games).

An existing installation of the original game also works without the installer: put the
program into the folder that holds `files` and `var` and start it there.

### The handheld

There is no installer step on the Anbernic RG34XXSP. The program is built on a PC and
copied over SSH, and the game's data comes from an installed folder on the PC:

```sh
tar cf - files var savedata | ssh root@address 'mkdir -p /userdata/roms/ports/rally; tar xf - -C /userdata/roms/ports/rally'
```

The game then appears under Ports as "Rally"; MENU + START leaves it. Details:
[tools/anbernic/README.md](tools/anbernic/README.md).

## Building

The repository and the crate are named `rc2k_decomp`. The Rust toolchain is pinned in
`rust-toolchain.toml`. All builds described here are made
on Linux, except the Mac's, which can also be made on a Mac. SDL and FFmpeg are not in the
repository; the scripts in `tools/` fetch or build them into `third_party/`:

- `tools/fetch_sdl3.sh` fetches SDL's ready-made Windows library (a pinned version,
  checked by its hash). The other systems build SDL from source through the `sdl3-sys`
  crate, which needs cmake and the usual development packages of the system's video and
  sound (the list for Ubuntu is in `.github/workflows/build.yml`).
- `tools/build_ffmpeg.sh [x86|x64|linux|a64|mac]` builds FFmpeg from its signed source
  with only the three decoders the game's films need. It needs curl, gpg and make; for
  Windows also clang-cl, lld-link, llvm-lib and llvm-nm, and the Windows SDK that a first
  `cargo xwin` run fetches.

### Linux

```sh
tools/build_ffmpeg.sh linux
cargo build --release --target x86_64-unknown-linux-gnu --bin rc2k
```

The program is `target/x86_64-unknown-linux-gnu/release/rc2k`.

### Windows (crossed from Linux with cargo-xwin)

```sh
cargo install --locked cargo-xwin
tools/fetch_sdl3.sh

# 64 bits
cargo xwin cache xwin --xwin-arch x86_64
tools/build_ffmpeg.sh x64
cargo xwin build --release --xwin-arch x86_64 --target x86_64-pc-windows-msvc --bin rc2k

# 32 bits (no debug window)
tools/build_ffmpeg.sh
cargo xwin build --release --xwin-arch x86 --target i686-pc-windows-msvc --bin rc2k
```

Next to `rc2k.exe` go `SDL3.dll` (`third_party/sdl3/lib/x64` or `lib/x86`) and
`avcodec-61.dll` and `avutil-59.dll` (`third_party/ffmpeg-x64/bin` or
`third_party/ffmpeg/bin`).

### macOS

On a Mac with the Command Line Tools, rustup's two `apple-darwin` targets, cmake and
gnupg:

```sh
./deploy_standalone.sh mac        # both processors in one bundle; `mac x64` or `mac a64` for one
```

It builds FFmpeg, both programs, joins them with `lipo` into `Rally Championship.app`,
signs it ad hoc (it is not notarised) and copies it into `MAC_GAME_DIR` (default
`~/rally`). Crossing from Linux is described in [tools/mac/README.md](tools/mac/README.md).

### Anbernic RG34XXSP

Needs Docker. The build runs in a container of the handheld's stock system
(`tools/anbernic/Dockerfile`); `tools/anbernic/run.sh` runs a command in it.

```sh
docker build -t rc2k-anbernic tools/anbernic
tools/anbernic/run.sh tools/build_ffmpeg.sh a64
tools/anbernic/run.sh cargo build --release --target aarch64-unknown-linux-gnu --bin rc2k
```

The program is `target/a64-linux/aarch64-unknown-linux-gnu/release/rc2k`.
`./deploy_standalone.sh anbernic` does these steps and copies the result to the device
(`ANBERNIC_HOST` in `deploy.local`).

### Web (experimental)

Needs the Emscripten SDK in `~/emsdk` and a nightly toolchain with the
`wasm32-unknown-emscripten` target and `rust-src`:

```sh
tools/web/build.sh
python3 tools/web/serve.py [folder] [port]
```

The output is `target/web/wasm32-unknown-emscripten/release/rc2k.js` and `rc2k.wasm`; the
page is `tools/web/index.html`. This build is unfinished.

### Building and copying in one step

`deploy_standalone.sh` builds and copies the program into a game folder. It reads
`deploy.local` (not in git), which sets `GAME_DIR`; the program goes into the folder above
it, or into `STANDALONE_DIR` if set.

| Command | Builds | Copies to |
| --- | --- | --- |
| `./deploy_standalone.sh` | Windows, 32 bits | the game's folder |
| `./deploy_standalone.sh x64` | Windows, 64 bits | `x64/` in it |
| `./deploy_standalone.sh linux` | Linux | `linux/` in it |
| `./deploy_standalone.sh mac [x64\|a64]` | macOS | `MAC_GAME_DIR` |
| `./deploy_standalone.sh anbernic [pgo]` | the handheld | the device, over SSH |

The GitHub workflow `.github/workflows/build.yml` builds Windows (64 bits), Linux and
macOS the same way.

### Checks

```sh
cargo fmt
cargo clippy --target i686-pc-windows-msvc
cargo clippy --target i686-pc-windows-msvc --tests -p rc2k_decomp
CARGO_TARGET_DIR=target/linux cargo clippy --target x86_64-unknown-linux-gnu -p rc2k_decomp --tests
cargo test -p rc2k_core
cargo xwin test -p rc2k_core --xwin-arch x86 --target i686-pc-windows-msvc    # run under Wine
CARGO_TARGET_DIR=target/linux cargo test --target x86_64-unknown-linux-gnu -p rc2k_decomp
```

Clippy must give no warnings, and `rc2k_core`'s tests must pass on both Linux and Windows.
Changes to the installer are also run through `tools/instest.sh` (Linux) and
`tools/winstest.sh` (Windows under Wine), which install from a copy of the CD's files into
a throwaway home. `tools/fuzz.sh` fuzzes the readers of outside data (pictures, packed
files, texts, network messages). `tools/remote.py` drives a build made with
`--features remote` (keys, pad, screenshots) for automated runs.

## Layout

| Path | What it holds |
| --- | --- |
| `src/` | The game: one folder per area (`race`, `physics`, `scene`, `menu`, `screens`, `hud`, `effects`, `audio`, `sound`, `net`, `saves`, `install`, `gfx`, `files`, ...), the renderers in `render/`, each system's own code in `platform/`, the debug window in `debug/`. The program's entry is `src/bin/rc2k.rs`. |
| `rc2k_core/` | The game's logic with no platform and no `unsafe`; most unit tests are here. |
| `rc2k_derive/` | Derive macros for the typed cells and byte views the game's records use. |
| `tools/` | Build scripts (FFmpeg, SDL, shaders), the handheld's, the Mac's and the web build's tooling, the installer tests, the remote test driver. |
| `fuzz/` | Fuzz targets for `rc2k_core`'s readers. |
| `data/vectors/` | Recorded inputs and results that the tests compare against. |
| `wiki/` | The source of the project's wiki. |
| `ida_db/` | The IDA database of the original program that the port was read from; kept as a record of the research. |

<!-- The wiki links point at the repository's wiki/ files for now; to be switched to the
     published GitHub wiki later. -->
More about the code and the game's files is in the [wiki](wiki/Home.md).

## Bugs of the original that are fixed

During the port every slip of the original was kept, so that the two could be compared.
They have been fixed since. Some that a player could meet:

- Driving through mud could end the game with a "Guru Meditation" error, when the
  particles ran out of room.
- On processors with more than one core, engine or ambient sounds could keep playing over
  the results and the menus after a stage.
- With the Dolby sound mode there was no music in races, and the effects were quieter and
  harsher than in the other modes.
- Pressing Escape on the "leave?" question to say no could still end a championship.
- After a race left through the quit question, the intro film could play over and over.
- A championship's car, gearing and controls stayed as the player's own settings after
  it, and were saved.
- In a split-screen single rally only the first player's car was repaired between stages.
- In network races the other players' cars jerked and at times jumped back along the
  road.
- On today's processors the frame rate display showed nothing below about 28 frames a
  second.
- Closing the window left the game running, and lost the options and the session's
  records.

The full list, with what each bug was and how it was fixed, is in
[wiki/Fixed-bugs.md](wiki/Fixed-bugs.md).

## Credits

- Diatosta: author and maintainer.
- The original *Rally Championship 2000* is by Magnetic Fields (Software Design) Ltd.
- Libraries: [SDL](https://libsdl.org) (window, input, sound, GPU), [FFmpeg](https://ffmpeg.org)
  (the films' decoders), [Dear ImGui](https://github.com/ocornut/imgui) (the debug
  window), and the Rust crates `rayon` and `miniz_oxide`.
- Written with the help of AI coding assistants (Claude), as said at the top.

## Licence and legal

The project's own code is under the GNU General Public License, version 3 or (at your
option) any later version (GPL-3.0-or-later); see [LICENSE](LICENSE).

*Rally Championship 2000* and its data belong to their owners. This repository contains
neither the original program nor any of the game's data files (pictures, stages, cars,
sounds, films); they are needed from your own copy of the game. The project is not
affiliated with or endorsed by the game's developers or publishers.

Some of the original program's own built-in data is present in the source, because the
game needs it: tables, menu definitions, constants and texts (such as its copyright
lines). These are not covered by the project's licence and remain their owners'.

Builds link SDL3 (zlib licence) and FFmpeg's decoders (LGPL 2.1); the debug window uses
Dear ImGui (MIT). The Linux, macOS and handheld builds link FFmpeg statically, which the
GPL allows (LGPL 2.1 code may be combined into a GPL program).
