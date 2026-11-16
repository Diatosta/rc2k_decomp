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

## Installing from the CD's files

1. Put the program into the folder that holds the CD's files:

   | System | Files to put there |
   | --- | --- |
   | Windows | `rc2k.exe`, `SDL3.dll`, `avcodec-61.dll`, `avutil-59.dll` |
   | Linux | `rc2k` |
   | macOS | `Rally Championship.app` |

2. Start it there. When the program finds itself among the CD's files with no installation, it
   opens the game's own installer.
3. Choose the language and the folder. The installer copies the game, and the program itself,
   into that folder, adds the menu entries and starts the installed copy.

The installer is the 1999 game's, with a few things left out: there is no page for the kind of
installation (it is always the full one), no DirectX set-up and no registration. The whole list
is on [Differences from the original](Differences-from-the-original).

### Where the game goes

| System | Folder offered |
| --- | --- |
| Windows | chosen on the installer's drive and folder pages |
| Linux | `$XDG_DATA_HOME/rally-championship` (usually `~/.local/share/rally-championship`) |
| macOS | `~/Library/Application Support/Rally Championship` |

### The menu entries

Three entries are made: "Rally", "Uninstall Rally" and "View Readme".

| System | Where they are |
| --- | --- |
| Windows | shortcuts in the Start menu, in a group named "Rally Championship" |
| Linux | desktop entries in `~/.local/share/applications` |
| macOS | small application bundles in `~/Applications/Rally Championship/` |

## What an installed folder holds

| Path | What it is |
| --- | --- |
| `rc2k` / `rc2k.exe` and its DLLs / `Rally Championship.app` | the program |
| `files/` | the game's data, copied from the CD's `FILES` |
| `var/` | options, best times, the game's log, the installation's record |
| `savedata/` | saved games, saved replays, car set-ups, screenshots |
| `rc2k.log` | the program's log, written anew at each start |

The program always works in the folder it is started in: it looks for `files` and `var` there.
The menu entries start it in the right folder.

## Using an existing installation of the original

An installed copy of the original game works without the installer. Put the program (and on
Windows its three DLLs) into the folder that holds `files` and `var` and start it there. The
original's saved games, replays, best times and options are read as they are; see
[Options files](Options-files) for how the options are brought over.

## Removing the game

Use "Uninstall Rally". It starts the installed program with the switch `-u`, which asks
whether to remove everything, to remove the game but keep the saved games, or to cancel. The
menu entries are removed too.

Only the installed copy can remove its own installation: run `rc2k -u` in the installed folder,
not from the CD's files.

## Moving or copying an installation

The installation's record is a file inside the game's folder (`var/install.cfg`, see
[Install records](Install-records)); nothing is written to the Windows registry. So an
installed folder can be moved, copied to another disk or to another machine, and it still
works. The menu entries are not moved with it: they keep pointing at the old place.

## The handheld

There is no installer step on the Anbernic RG34XXSP. The program is built on a PC and copied
to the device ([Building from source](Building-from-source)), and the game's data is copied
once from an installed folder on the PC. From that folder, with the device's address in place
of `address`:

```sh
tar cf - files var savedata | ssh root@address 'mkdir -p /userdata/roms/ports/rally; tar xf - -C /userdata/roms/ports/rally'
```

The game then appears under Ports as "Rally". MENU + START leaves it. More in
`tools/anbernic/README.md` in the repository.
