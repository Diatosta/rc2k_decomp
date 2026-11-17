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

- The controller page has no axis mode or dead zone items; an axes list opens a page for each axis that edits its dead
  zone, sensitivity curve, invert and range (and a wheel's rotation), and the buttons page lists
  the steering, accelerate and brake axes and the buttons, with reset car, on two
  pages ([Controllers](Controllers)).

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
- TCP/IP is offered, and Steam in builds made with the `steam` feature: no modem, serial or IPX connection.

More on [Network play](Network-play).

## Files

- Options are kept in a text file, `var/rc2k.toml`. The original's `var/ral.dat` is read when
  the new file does not exist, and written only on request ([Options files](Options-files)).
- The options are saved when a settings page is left, not only when the game closes. The review menu
  after a stage opens the options (audio, display, the players' details and controllers), where the
  original offered fewer.
- Every other file keeps the original's format: replays, saved games, car set-ups, best times
  and `var/ral.cfg`. Old replays and saves are read as they are.
- An error that ends the game is shown on a panel in the game's window (the message, where the
  details are, "Quit"; Up / Down and Enter, or a pad's buttons) in place of the original's
  "Guru Meditation" box or a silent exit; the log keeps the line. Troubles that belong to one file
  no longer end the game: a saved game that is damaged is not loaded, a replay that cannot be
  read is skipped (a notice, then the menu again), and options in `var/rc2k.toml` that are not
  valid are reset to their defaults with a notice. `RC2K_NO_ERROR_DIALOG` (any value) turns the
  panel off for unattended runs, which then end as before.
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
- The volume page has a "film volume" slider (the intro and logo films; the last slider, above the music switch), the sound page a "sound when not in front" row (the original's sounds went quiet while its window was behind), and the network settings page a "port" row ([Options files](Options-files), [Network play](Network-play)).
- The hidden Pong game on the windscreen can be played: the bat follows player 1's steering ([Fixed bugs](Fixed-bugs)).
- The loading screen does not hold up the load: the loading picture is shown once (the
  original showed it once for each page of its screen), and a bar frame is drawn only when
  the last frame of the loading screen is at least 33 ms old, so the bar's last steps may go
  unseen.
- The frames follow the display's refresh: one frame for each refresh (60 a second on a 60 Hz
  display, also when the display's rate is not known). The race moves 100 times a second; on
  displays faster than that the frames between two of its ticks are drawn blended between the
  last two ticks (the advanced page's "smooth frames", on by default, kept as `smooth_frames`
  in `rc2k.toml`; the picture is up to a tick behind the simulation, the simulation itself and
  so every replay and held race is unchanged). With "smooth frames" off a frame shows the last
  tick and the frames are every second or third refresh on displays faster than 100 Hz. The
  windscreen's drops and flakes appear at most once a tick, so a faster display gets no more
  of them. The program does this by
  waiting until each frame is due; with SDL's own switch `SDL_RENDER_VSYNC=1` in the
  environment the showing itself waits for the display instead, at every refresh. The speed
  of the race does not depend on it: its steps come from the clock. The graphics page has a
  "frame limit" row after the original's: no limit (nothing here waits; a display that waits
  for its refresh still does), the display's own pace (the default), 30 or 60 a second, and the
  rates that divide the display's refresh evenly (half, a third, a quarter of it; with smooth
  frames the display's own rate and its half, shown as "(half)", up to 1000 a second). It is kept in
  `var/rc2k.toml`'s `[display]` table as `frame_limit` (`"off"`, `"display"` or a number a second;
  left out for the default, `"display"`). A "3D resolution" row below it draws the race's 3D at
  50, 75, 100, 125, 150, 175 or 200 % of the screen's size: below 100 it is enlarged (nearest
  pixel) over the screen, above 100 it is drawn larger and scaled down with a smoothing filter
  (supersampling anti-aliasing; an exact area average in the software rasteriser, the bilinear
  filter on the GPU renderers, which is the mean of four pixels at 200 %); the
  menus' pictures, the dashboard and the texts stay at full size, while the menus' 3D models (the
  main menu's key and signpost, the helmets, the car carousel) are drawn on the GPU renderers at
  the size the menu's frame has in the window times this share, so they are sharp on a large window
  (the software rasteriser draws them at the frame's own size). It works on the software rasteriser and
  on both GPU renderers, and is kept in `[display]` as `render_scale` (50 to 200 in those steps; left out
  for 100). The handheld's software rasteriser no longer draws races at half size by itself:
  set the row to 50 % there.
- The options menu has an "advanced" item after the original's, opening a page of our own (the in-game options menu, opened after a stage or from the network lobby, has the same item after its display one). Its
  rows: how many parts of a frame may be drawn at once (1 up to the machine's cores, at most 4:
  the mirror beside its view needs 2, the split-screen players' views and mirrors beside each
  other 4; read at each frame, so a change shows at the next one); the renderer (GPU, OpenGL or
  software), swapped in at once on the desktop by Apply: the frame is brought back into the
  canvas, the running backend and the window's renderer are stopped and the chosen ones
  started, and the textures and the canvas go up to the new backend as it first draws them. When
  the chosen one does not start, the one that ran comes back, the choice is kept for the next
  start, and the page says "next start" beside the renderer that was asked for and did not start. On
  the handheld the renderer is only the one tried first at the next start (its GL ES context is
  made once for the program's life), and the page says "next start" beside a value that differs from the running one; and switches of `var/ral.cfg`: frame counter,
  films, screenshots, non-linear steering, automatic neutral (all at once), rain and snow (the
  next race) and music, force feedback (the next start; `nojoy` is not on the page, so the handheld cannot switch its controls off, and it still reads from `ral.cfg` by hand). The page shows the feature,
  not its absence (`nofmv=1` shows as "films: off"). "Apply" sets the live variables, writes
  the changed keys into `var/ral.cfg` (the key's line is replaced, the others kept; a missing
  key is added) and keeps the other two rows in `var/rc2k.toml`'s `[display]` table as
  `renderer` and `draw_threads` (both left out until chosen; the renderer is the weakest of
  the sources after `--renderer` and `RC2K_RENDERER`, and a fallback never rewrites it).
- The developers' debug pages, unreachable in the released game, are a window over the game
  ([The debug window](The-debug-window)). Not in the 32-bit Windows build or on the handheld.

## Not there

- Playing against the original program over a network.
- The original's choice between 3D cards; the program picks its renderer itself
  (`RC2K_RENDERER=software` forces the rasteriser).
- Wide screens: the race's resolution list holds the monitor's own modes, the usual sizes of 4:3, 16:10,
  16:9, 21:9 and 32:9 that fit the desktop, and the game's eight, each shown with its ratio ("2560 x 1080
  (21:9)"); `custom_width` / `custom_height` in `[display]` set any other size. On a screen wider than 4:3 the
  3D keeps its field of view up and down and shows more to the sides (the usual "Hor+"), and the HUD keeps
  its 800 x 600 shape instead of stretching: the dials, the map, the timers and the mirror keep to their
  corners and edges, the progress bar to the middle (two players side by side each keep to their half, one
  above the other the divider spans the screen). The "HUD area" row confines the HUD to a centred 4:3,
  16:9 or 21:9 box. The "screen" row shows the window as a window, full screen or borderless. On a screen
  as wide as 4:3 or narrower everything is as before.
- The window keeps the size of the race's screen mode from the start to the end. The original's menus

  ran at 800 x 600 and switched the display when a race began or ended; here the menus are still drawn
  at 800 x 600 (their pictures, layouts and fonts are made for it) and shown scaled in the window,
  keeping the 4:3 shape and centred, with a linear filter (a whole-number factor stays sharp, so
  1600 x 1200 shows each menu pixel as a 2 x 2 square). On a window of another shape the space around the menu
  shows the menu's own background picture alone (no 3D models, texts or buttons), scaled to cover the
  whole window, strongly blurred and darkened; it is made once each time the background or the
  window's size changes, and follows the menus as they change (black until a menu has drawn a
  background, as on a film's frames, which keep the last one); a race's own bars are black. The menu mode in
  the options files is still read but no longer changes the window. The handheld's panel is
  unchanged.
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
