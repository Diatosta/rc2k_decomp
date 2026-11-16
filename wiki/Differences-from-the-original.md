# Differences from the original

*Category: [About](About)*

The game is meant to be the same for the player as the 1999 program: the same menus, physics,
stages, replays and save files. This page lists what was changed on purpose. Mistakes of the
original that were put right are on [Fixed bugs](Fixed-bugs).

## The platform

The original was a 32-bit Windows program on DirectX. Its platform side was not copied:

| Original | Here |
| --- | --- |
| DirectDraw and Direct3D | SDL3 for the window; races drawn on the GPU (SDL's GPU device, or OpenGL ES on the handheld) or by a software rasteriser ([Drawing a frame](Drawing-a-frame)). |
| DirectSound | The game's sound code over a small mixer on SDL's sound output ([Sound and music](Sound-and-music)). |
| DirectInput | SDL's keyboard, gamepad, joystick and haptic support ([Input](Input)). |
| DirectPlay | A protocol of this project's own ([Network messages](Network-messages)). |
| The system's film player | FFmpeg's decoders ([Films](Films)). |
| Windows, 32 bits | Windows (32 and 64 bits), Linux, macOS, a handheld ([Platforms](Platforms)). |

## Installing

- No registry. The installation's record is the file `var/install.cfg` in the game's folder,
  so an installed folder can be moved or copied ([Install records](Install-records)).
- The installer takes the game from the folder it is started in, in place of asking for the
  CD.
- The installer has no page for the kind of installation (it is always the full one), no
  DirectX set-up and no registration. It makes ordinary shortcuts.
- "Uninstall Rally" runs the installed program itself (`rc2k -u`).

More on [The installer](The-installer).

## Network play

- The protocol is this project's own, so the program plays against other copies of itself,
  not against the original.
- Only TCP/IP is offered: no modem, serial or IPX connection, and no lobby services.

More on [Network play](Network-play).

## Files

- Options are kept in a text file, `var/rc2k.toml`. The original's `var/ral.dat` is read when
  the new file does not exist, and written only on request ([Options files](Options-files)).
- Every other file keeps the original's format: replays, saved games, car set-ups, best times
  and `var/ral.cfg`. Old replays and saves are read as they are.
- The program writes a log of its own, `rc2k.log`, beside the game's `var/logfile.txt`
  ([Log files](Log-files)).
- On Linux and macOS, file names are matched without regard to case, as Windows does.

## Running

- The start-up does not wait where the original paused and timed the processor; the game's log
  no longer has a clock-speed line.
- A single rally stage can be started straight from the command line
  ([Running and options](Running-and-options)).
- Closing the window ends the game properly, saving the options and the session's records.
- A loading bar is shown while a stage loads.
- The developers' debug pages, unreachable in the released game, are a window over the game
  ([The debug window](The-debug-window)). Not in the 32-bit Windows build or on the handheld.

## Not there

- Playing against the original program over a network.
- The original's choice between 3D cards; the program picks its renderer itself
  (`RC2K_RENDERER=software` forces the rasteriser).
- The web build is experimental and does not show a picture yet.
