# Platforms

*Category: [Getting started](Getting-started)*

What the program runs on, what each system needs, and what is different or missing on each.

## What runs where

| Platform | State |
| --- | --- |
| Windows, 64 and 32 bits | Built and run (under Wine and Proton during development). |
| Linux, x86-64 | Built and run. One program with SDL linked in. The `linux` build needs the glibc of the system it was built on; for older systems use the Steam Deck build. |
| Steam Deck (SteamOS 3, and any x86-64 Linux with glibc 2.35 or later) | `./build.sh deck`: built in a container of Ubuntu 22.04, tuned for the Deck's Zen 2 processor, with Steam network play; released as `rc2k-deck.zip`. Not yet run on a real Deck. |
| macOS, Intel and Apple silicon | Builds as `Rally Championship.app`, one bundle for both processors. Tested less than the others. |
| Anbernic RG34XXSP (handheld, 64-bit ARM Linux) | Built and run on Knulli and on the device's stock system. |
| Android (phones and tablets, x86-64 and 64-bit ARM) | Begun: it builds as an `.apk`, starts in an emulator, plays a race with touch controls. Not yet run on a phone. See [Android](Android). |
| Web (WebAssembly) | Experimental: menus and races play in a desktop browser (WebGL 2, gamepads, sound, full screen); not yet checked on a phone. |

The game itself is the same everywhere: the same menus, stages, physics, replays and save
files. A replay or a saved game made on one system works on the others.

## What each needs

| Platform | Needs |
| --- | --- |
| Windows, 64 or 32 bits | Windows 10 or later. `SDL3.dll` beside `rc2k.exe`. |
| Linux, x86-64 | Only the C library (glibc). Which version depends on the system the program was built on: the project's automatic `linux` build is made on Ubuntu 24.04 (glibc 2.39). The `deck` build is made on Ubuntu 22.04 (glibc 2.35) and runs on older systems. |
| macOS | macOS 11.0 (Big Sur) or later. |
| Steam Deck | SteamOS 3 (glibc 2.35 or later; the Deck's has more). Run it from desktop mode, or add `rally.sh` as a non-Steam game. A processor with Zen 2's instructions (AVX2) is needed; older ones are not supported by this build, use the `linux` build. |
| Anbernic RG34XXSP | Knulli or the stock system; glibc 2.35 or later. |

## Drawing

Races and the menus' 3D are drawn on the GPU where a GPU renderer can be started, and
otherwise by the program's own software rasteriser, which needs no GPU at all.

| Platform | GPU renderer | If it cannot start |
| --- | --- | --- |
| Windows, Linux | SDL's GPU device, on Vulkan | OpenGL 3.3, then the software rasteriser |
| macOS | SDL's GPU device, on Metal | OpenGL, then the software rasteriser |
| Handheld | OpenGL ES 3.0, through the system's `libmali` | software rasteriser |

`rc2k.log` says which one started and, if the GPU one did not, why.
`--renderer software` (or `RC2K_RENDERER=software`) forces the rasteriser ([Running and options](Running-and-options)).

## Per-platform notes

**Windows, 32 bits.** No [debug window](The-debug-window). Everything else is the same as the
64-bit build.

**Windows in general.** The builds have been run under Wine and Proton. The script that builds
on Windows itself has not yet been tried on a real Windows machine
([Building from source](Building-from-source)).

**Linux.** The installer puts the game in `~/.local/share/rally-championship` and its menu
entries in `~/.local/share/applications`. The game's files are named the Windows way (mixed
capitals, backslashes in paths); the program finds them without regard to case, so nothing
needs renaming.

**macOS.** The bundle is signed ad hoc and not notarised, so the first start may need to be
allowed in the system's security settings. The program works in the folder that holds the
bundle. The Metal renderer has been tested less than the Vulkan one.

**Phones, the web build and the Vita.** Rendering a replay to a video is a desktop feature for
now: it starts a second copy of the program and `ffmpeg`, which these cannot do, so the replay
viewer shows no "render to video" button there ([Running and options](Running-and-options)).

**Handheld.** There is no installer: the program and the game's data are copied over SSH
([Installing the game](Installing-the-game)). The picture goes straight to the device's
screen. The built-in pad works in the menus; MENU + START leaves the game. There is no debug
window. Measured speed is on [Performance](Performance).

**Android.** Not an installer: an app (`rc2k-<abi>.apk`) with the game's files in its private storage. Touch controls in the race and gestures in the menus; the picture, sound and the game are the others'. [Android](Android) says how it is built, how to put the game's files in and what is not done.

**Web.** Unfinished (WebGL 2 picture, gamepads and sound work in headless Chrome; not yet checked on a phone). The build and its page are in `tools/web/`. Once the repository is public the page is published on its GitHub Pages (the `pages` job of the workflow; see [Building from source](Building-from-source)); it holds no game data, so every player picks their own game folder or zip, and it has no network play. The start page takes the game folder or its zip: the CD's files run the installer (into the browser's storage), an already installed game folder is played as it is. Full screen takes the whole page; the resolution list offers the browser window's size and the screen's, and the frame rate follows the display's refresh as on the desktop. The saves live in the
browser's private storage, which a browser may clear, so the page `saves.html` (linked from the
start page) exports the `savedata/` folder, `var/hst.dat` and the settings (`var/ral.dat`,
`var/ral.cfg`, `var/rc2k.toml`) as a zip to download or as text (the zip in base64) to copy, and
imports either back, replacing what is there after a confirmation. Only those files are read
from an imported zip. Close the game's tab first.

## Network play across platforms

Network play uses the project's own protocol on every system
([Network play](Network-play)). It does not play against the original 1999 program.
