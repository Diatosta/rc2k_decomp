# Installing the game

*Category: [Getting started](Getting-started)*

The program, `rc2k`, is only the game's engine. The pictures, stages, cars, sounds and films
come from your own copy of *Rally Championship 2000*. This page says how the two are put
together.

## What you need

- The files of the game's CD on your disk: the folder that holds the CD's `FILES` folder. A
  copy of the CD, or a download of it, both work.
- The program for your system: a build from the project's releases, or one you built yourself
  ([Building from source](Building-from-source)).

What each system needs to run the program is on [Platforms](Platforms).

### Which download, and where to read on

A release holds one zip per platform:

| Platform | Zip | What is in it | Steps |
| --- | --- | --- | --- |
| Windows, 64 bits | `rc2k-windows-x64.zip` | `rc2k.exe`, `SDL3.dll` | [Installing from the CD's files](#installing-from-the-cds-files) |
| Linux | `rc2k-linux-x64.zip` | `rc2k` | [Installing from the CD's files](#installing-from-the-cds-files) |
| macOS | `rc2k-macos.zip` | `Rally Championship.app` | [Installing from the CD's files](#installing-from-the-cds-files), [The installer on macOS](#the-installer-on-macos) |
| Steam Deck | `rc2k-deck.zip` | `rc2k`, `rally.sh` | [Steam Deck](#steam-deck) |
| Anbernic RG34XXSP | `rc2k-handheld.zip` | `rc2k`, `Rally.sh`, `rally-image.png` | [Anbernic RG34XXSP (Knulli)](#anbernic-rg34xxsp-knulli) |
| Android | `rc2k-android.zip` | `rc2k-arm64.apk`, `rc2k-x86_64.apk` | [Android](Android#installing) |
| iPhone / iPad | `rc2k-ios.zip` | `Rally Championship.ipa`, unsigned | [iOS](iOS#installing) |
| Web | `rc2k-web.zip` | the page, `rc2k.js`, `rc2k.wasm`, `serve.py` | [Web](#web) |

The macOS and iOS builds are made only for stable releases (the main branch), not for the
experimental ones of other branches. `rc2k-linux-x64-debug.zip` and `rc2k-deck-debug.zip` hold
only the programs' symbols (`rc2k.debug`), for readable crash reports; a player does not need
them. There is no released 32-bit Windows program: build it yourself.

## Installing from the CD's files

1. Put the program into the folder that holds the CD's files:

   | System | Files to put there |
   | --- | --- |
   | Windows | `rc2k.exe`, `SDL3.dll` |
   | Linux | `rc2k` (`chmod +x rc2k` if it is not executable) |
   | macOS | `Rally Championship.app` |

2. Start it there. When the program finds itself among the CD's files with no installation, it
   opens the game's own installer.
3. Choose the language and the folder. The installer copies the game, and the program itself,
   into that folder, adds the menu entries and starts the installed copy.

The installer is the 1999 game's, with a few things left out: there is no page for the kind of
installation (it is always the full one), no DirectX set-up and no registration. The whole list
is on [Differences from the original](Differences-from-the-original).

The program does not have to be among the CD's files. Started in a folder with no game data,
it opens a page that asks for the game's folder or a zip of it (pick one, or drop it on the
window); `--game-data <folder or zip>` names it on the command line. What it finds decides
what happens next: the CD's files start the installer, an installed game is played where it
is ([Running and options](Running-and-options)).

### Where the game goes

| System | Folder offered |
| --- | --- |
| Windows | chosen on the installer's drive and folder pages |
| Linux | `$XDG_DATA_HOME/rally-championship` (usually `~/.local/share/rally-championship`) |
| macOS | `~/Library/Application Support/Rally Championship` (what the game writes); the application itself is `/Applications/Rally Championship.app`, or `~/Applications/Rally Championship.app` when the user cannot write to `/Applications` |

### The menu entries

Three entries are made on Windows and Linux: "Rally", "Uninstall Rally" and "View Readme".
macOS has none: the installed application is the entry (see below).

| System | Where they are |
| --- | --- |
| Windows | shortcuts in the Start menu, in a group named "Rally Championship" |
| Linux | desktop entries in `~/.local/share/applications`, the game's with its icon |
| macOS | none; Launchpad and Finder list `Rally Championship.app` itself |

### The installer on macOS

The installer copies its own bundle, `Rally Championship.app`, to `/Applications` (to
`~/Applications` when that cannot be written), puts the game's `files` inside the copy
(`Contents/Resources/files`) and keeps what the game writes (`var`, `savedata`, the logs) in
`~/Library/Application Support/Rally Championship`. The bundle is not written to while the game
runs. At the end the copy is signed again ad hoc (`codesign --force --deep --sign -`, which
ships with macOS), freed of the quarantine mark, registered with Launch Services (so Launchpad
shows it, with the icon and the name "Rally Championship") and opened; the installer then
ends. A program that does not run from a bundle installs as on Linux.

Moved, a bundle that holds `Contents/Resources/files` still runs from its new place: it always
reads its files from itself and writes in Application Support. A bundle placed next to a game
folder (`files`, `var`, `savedata` beside it) runs in that folder as before.

The folder field of the installer's folder page shows a path longer than the box by its end,
with "..." in place of the start.

## What an installed folder holds

| Path | What it is |
| --- | --- |
| `rc2k` / `rc2k.exe` and `SDL3.dll` / `Rally Championship.app` | the program |
| `files/` | the game's data, copied from the CD's `FILES` |
| `var/` | options, best times, the game's log, the installation's record |
| `savedata/` | saved games, saved replays, car set-ups, screenshots |
| `rc2k.log` | the program's log, written anew at each start |
| `rc2k.lock` | empty; the running program holds a lock on it, so that a second copy in the folder knows (`--allow-another` lets one start anyway) |

The program always works in the folder it is started in: it looks for `files` and `var` there.
The menu entries start it in the right folder.

## Using an existing installation of the original

An installed copy of the original game works without the installer. Put the program (and on
Windows `SDL3.dll`) into the folder that holds `files` and `var` and start it there. The
original's saved games, replays, best times and options are read as they are; see
[Options files](Options-files) for how the options are brought over.

## Removing the game

Use "Uninstall Rally". It starts the installed program with the switch `-u`, which asks
whether to remove everything, to remove the game but keep the saved games, or to cancel. The
menu entries are removed too.

On macOS, remove the game by moving `Rally Championship.app` to the Bin (the saved games stay
in Application Support until that folder is deleted), or run `Rally Championship.app/Contents/MacOS/rc2k -u`,
which asks the same questions and removes the application too.

Only the installed copy can remove its own installation: run `rc2k -u` in the installed folder,
not from the CD's files.

## Moving or copying an installation

The installation's record is a file inside the game's folder (`var/install.cfg`, see
[Install records](Install-records)); nothing is written to the Windows registry. So an
installed folder can be moved, copied to another disk or to another machine, and it still
works. The menu entries are not moved with it: they keep pointing at the old place.

## Steam Deck

The Deck's build (`rc2k-deck.zip`, or `./build.sh deck`) is a Linux program made to run on
SteamOS 3: built against an older C library than the `linux` build and for the Deck's
processor. It holds `rc2k` and `rally.sh`, a small script that starts `rc2k` from the folder
the script is in. **It has not yet been run on a real Deck**; the steps below are the Linux
ones, which is what the build is.

1. Switch to desktop mode (the Steam button, Power, Switch to Desktop).
2. Get the CD's files and the zip onto the Deck, and unzip `rc2k` and `rally.sh` into the
   folder that holds the CD's files. If they are not executable: `chmod +x rc2k rally.sh`.
3. Start `rc2k` there (from the file manager, or `./rc2k` in a terminal). The installer runs
   as on Linux and puts the game in `~/.local/share/rally-championship`, with menu entries in
   the desktop's application menu.
4. For gaming mode: the installer copies the program but not `rally.sh`, so copy `rally.sh`
   into the installed folder too. Then in Steam (desktop mode), Games, "Add a Non-Steam Game to
   My Library", Browse, and choose that `rally.sh`. The game then shows in the library in
   gaming mode.

An installed folder copied from a PC works as well: put `rc2k` and `rally.sh` into it and
add `rally.sh` to Steam the same way.

Not covered: the Deck's controls have not been set up or checked by the project (the game
reads the Deck's pad through SDL like any other, see [Controllers](Controllers)), and network
play through Steam needs Valve's library beside the program, which the released zip does not
carry ([Network play](Network-play)).

The same build runs on any x86-64 Linux with glibc 2.35 or later and a processor with AVX2,
so it is also the one for a Linux system too old for the `linux` build.

## Anbernic RG34XXSP (Knulli)

There is no installer step on the handheld: the device gets a folder that is already an
installed game. The build is `rc2k-handheld.zip` from a release, or `./build.sh handheld`
([Building from source](Building-from-source)). It runs on Knulli and on the device's stock
system; the paths below are Knulli's.

You need a PC and the device on the same network, reachable over SSH as `root`, and the
device's address.

1. Install the game on the PC first (above), so that you have an installed folder with
   `files`, `var` and `savedata`.
2. Copy the game's data, once. In the installed folder on the PC, with the device's address
   in place of `address`:

   ```sh
   tar cf - files var savedata | ssh root@address 'mkdir -p /userdata/roms/ports/rally; tar xf - -C /userdata/roms/ports/rally'
   ```

   "Cannot change ownership" messages from the SD card are harmless.
3. Copy the build. In the folder of the unzipped build (or `dist/handheld/`):

   ```sh
   scp rc2k root@address:/userdata/roms/ports/rally/
   scp Rally.sh root@address:/userdata/roms/ports/
   ssh root@address 'chmod +x /userdata/roms/ports/rally/rc2k /userdata/roms/ports/Rally.sh'
   ```

   `rc2k` goes into the game's folder, `Rally.sh` beside that folder: it is the entry the
   device's menu lists.
4. On the device: Ports, "Rally". If the entry is not listed, have the menu read its game
   lists again (or restart the device). MENU + START leaves the game.

`rally-image.png` is the entry's picture in the menu. It is optional, and putting it in by
hand means editing the ports folder's `gamelist.xml`; `./build.sh handheld --deploy` (from a
checkout, with `ANBERNIC_HOST` in `deploy.local`) copies all three files and does that edit.

To update, copy `rc2k` again (step 3). The saved games are in
`/userdata/roms/ports/rally/savedata`, the log in `/userdata/roms/ports/rally/rc2k.log`.

On the stock system the game's folder was `/mnt/mmc/Roms/APPS/rally`; `Rally.sh` names
Knulli's folder, so it has to be edited there. More for developers is in
`tools/anbernic/README.md` in the repository.

## Phones and tablets

Neither phone build has an installer: the app plays a game folder put into its own storage.

- **Android**: install the `.apk`, then give the app a zip of the game's folder. See
  [Android](Android#installing).
- **iPhone and iPad**: the `.ipa` is unsigned; sideload it with AltStore and your own Apple ID,
  then copy the game's folder in with the Files app. See [iOS](iOS#installing).

## Web

The web build is experimental. It is a page and the program as WebAssembly; it holds no game
data, and nothing you give it leaves your machine: the game's files are copied into the
browser's own storage for that page.

You need a desktop browser (it has not been checked on a phone), Python 3 to serve the page,
and the game's folder or a zip of it.

1. Unzip `rc2k-web.zip` (or build it: `./build.sh web`, the result is `dist/web/`).
2. In that folder run `python3 serve.py`. It serves the folder at `http://127.0.0.1:8000/`
   (`python3 serve.py <folder> <port>` for another folder or port) and sends the two headers
   the program's threads need. Opening `index.html` as a file does not work.
3. Open that address. The page takes the game's folder, or its zip (in one or several
   parts).
4. The CD's files start the game's installer, which installs into the browser's storage; start
   the page again afterwards. An installed game's folder is played as it is, with its saved
   games, best times and settings.

The saved games live in the browser's storage, which a browser may clear. The link "Export or
import the saves and settings" on the start page (`saves.html`) saves them as a zip and takes
one back ([Platforms](Platforms)).

There is no network play in the web build. Once the repository's GitHub Pages are switched on
the same page will be served from there, with no download and no `serve.py`; that is not done
yet.
