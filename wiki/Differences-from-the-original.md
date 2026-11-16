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
| The system's film player | The project's own decoders in Rust ([Films](Films)). |
| Windows, 32 bits | Windows (32 and 64 bits), Linux, macOS, a handheld ([Platforms](Platforms)). |

## Racing

- The pause binding (a button on a pad and a key on the keyboard) asks the same
  "leave?" question as Escape, in place of the original's plain pause that waited for a key.
  The race still pauses until a key when its window stops being the active one.

## Controllers

- Learning a control on the controller page counts only what moves from where the controller
  rests when the learning starts, so an axis that rests at the end of its travel (a clutch
  pedal) is not taken for a direction held.

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
- The fixed pauses between screens (the service area's and the inspection's, the network's
  "connecting" lines, the installer's messages) keep their length, but a key or a pad's button
  ends them early, and the window answers during them.
- Controllers are read from a race's first moment (the original's input thread slept a second
  at each race's start), and at once after coming back to the window.
- A loading bar is shown while a stage loads.
- The loading screen does not hold up the load: the loading picture is shown once (the
  original showed it once for each page of its screen), and a bar frame is drawn only when
  the last frame of the loading screen is at least 33 ms old, so the bar's last steps may go
  unseen.
- The frames follow the display's refresh: one frame for each refresh (60 a second on a 60 Hz
  display, also when the display's rate is not known), every second or third refresh on
  displays faster than 100 Hz, as the race moves 100 times a second. The program does this by
  waiting until each frame is due; with SDL's own switch `SDL_RENDER_VSYNC=1` in the
  environment the showing itself waits for the display instead, at every refresh. The speed
  of the race does not depend on it: its steps come from the clock.
- The developers' debug pages, unreachable in the released game, are a window over the game
  ([The debug window](The-debug-window)). Not in the 32-bit Windows build or on the handheld.

## Not there

- Playing against the original program over a network.
- The original's choice between 3D cards; the program picks its renderer itself
  (`RC2K_RENDERER=software` forces the rasteriser).
- 16-bit screen modes: the screen always has 32 bits a pixel. A saved 16-bit mode is taken as
  the same size at 32 bits, and a first start picks 800 × 600 at 32 bits. The first logo
  film, which the original showed only on 16-bit screens, always plays.
- Twin code paths chosen by the processor's features (MMX, SSE, 3DNow!): the game computes
  one way on every machine, the way the original took on a processor with MMX and without
  3DNow!, with its SSE switch off. The settings `ignoremmx`, `ignorekatmai` and
  `ignoreamdk3d` are read and not used.
- What only served 1999's machines: CD audio, the process's raised priority (`noprichange`),
  the PC-98 keyboard layout, the search for 3D cards (`forceno3dcard`, `ignoreprimary`,
  `ignoresecondary`, the `-c` switch).
- The web build is experimental and does not show a picture yet.
