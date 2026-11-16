# Platforms

*Category: [Getting started](Getting-started)*

What the program runs on, what each system needs, and what is different or missing on each.

## What runs where

| Platform | State |
| --- | --- |
| Windows, 64 and 32 bits | Built and run (under Wine and Proton during development). |
| Linux, x86-64 | Built and run. One program with SDL linked in. |
| macOS, Intel and Apple silicon | Builds as `Rally Championship.app`, one bundle for both processors. Tested less than the others. |
| Anbernic RG34XXSP (handheld, 64-bit ARM Linux) | Built and run on Knulli and on the device's stock system. |
| Web (WebAssembly) | Experimental: it builds and starts, but does not show a picture yet. |

The game itself is the same everywhere: the same menus, stages, physics, replays and save
files. A replay or a saved game made on one system works on the others.

## What each needs

| Platform | Needs |
| --- | --- |
| Windows, 64 or 32 bits | Windows 10 or later. `SDL3.dll` beside `rc2k.exe`. |
| Linux, x86-64 | Only the C library (glibc). Which version depends on the system the program was built on: the project's automatic builds are made on Ubuntu 24.04. |
| macOS | macOS 11.0 (Big Sur) or later. |
| Anbernic RG34XXSP | Knulli or the stock system; glibc 2.35 or later. |

## Drawing

Races and the menus' 3D are drawn on the GPU where a GPU renderer can be started, and
otherwise by the program's own software rasteriser, which needs no GPU at all.

| Platform | GPU renderer | If it cannot start |
| --- | --- | --- |
| Windows, Linux | SDL's GPU device, on Vulkan | software rasteriser |
| macOS | SDL's GPU device, on Metal | software rasteriser |
| Handheld | OpenGL ES 3.0, through the system's `libmali` | software rasteriser, races at half size |

`rc2k.log` says which one started and, if the GPU one did not, why.
`RC2K_RENDERER=software` forces the rasteriser ([Running and options](Running-and-options)).

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

**Handheld.** There is no installer: the program and the game's data are copied over SSH
([Installing the game](Installing-the-game)). The picture goes straight to the device's
screen. The built-in pad works in the menus; MENU + START leaves the game. There is no debug
window. Measured speed is on [Performance](Performance).

**Web.** Unfinished. The build and its test page are in `tools/web/`.

## Network play across platforms

Network play uses the project's own protocol on every system
([Network play](Network-play)). It does not play against the original 1999 program.
