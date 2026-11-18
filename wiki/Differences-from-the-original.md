# Differences from the original

*Category: [About](About)*

The game is meant to feel like the 1999 program: the same menus, physics, stages, replays and
save files. This page lists everything that is different for the player, grouped by what you
meet, with where to find it and what it is set to at first. Mistakes of the original that were
put right (a sound that stuck on, a bat that never moved, ...) are on [Fixed bugs](Fixed-bugs)
and are not repeated here, except where they also changed what you do.

Where a setting is kept, the page [Options files](Options-files) has its key in `var/rc2k.toml`;
the switches of the command line are on [Running and options](Running-and-options).

## Highlights

- **Wide screens and any resolution**, 32:9 included, with full screen and borderless modes
  (Alt+Enter): [Graphics and the screen](#graphics-and-the-screen).
- **The GPU renderer and a 3D resolution up to 200 %**: [Graphics and the screen](#graphics-and-the-screen).
- **Frame rates above 100** (smooth frames), a frame limit: [Frame rate](#frame-rate).
- **Full play with a pad**, with the controller's own button pictures, profiles per controller,
  wheels, an H shifter and rumble: [Controllers and input](#controllers-and-input).
- **An on-screen keyboard** for typing with a pad: [Controllers and input](#controllers-and-input).
- **Lobby passwords, ping, un-ready and `/kick`** to put a player out: [Network play](#network-play).
- **Computer drivers (bots) in network races**, switched by the host in the lobby: [Network play](#network-play).
- **A player who leaves a network race** has their car taken out of it, with a message to the others, and **the other cars on the map disc**: [Network play](#network-play).
- **A save code**: the player's saved games, car set-ups and best times (not the settings) as text and as QR codes on the options menu's new page, copied, pasted or read with a camera: [Save code](Save-code).
- **Finding hosts over Tailscale**: [Network play](#network-play).
- **Playing through Steam** (in progress): [Network play](#network-play).
- **Replays rendered to a video**, from the viewer's camera button in a third row and from the command line:
  [Replays](#replays).
- **The replay viewer names the highlighted button**: when the highlight moves, its name shows above the
  buttons for a second: [Replays](#replays).
- **A replay format of its own** that records every car of a race, up to eight, the computer's and the other
  machines' cars of a network race included, so that they replay where they were: [Replays](#replays).
- **Three and four players on one screen**, a menu entry of their own in the single rally, the
  screen split in layouts the F2 key cycles: [In progress](#in-progress).
- **Straight into a race, a replay or the lobby** from the command line: [Running](#running).
- **Errors shown on a panel** in the window, and damaged files no longer ending the game:
  [Saves and files](#saves-and-files).
- **Linux, macOS, a handheld and (begun) Android** as well as Windows: [Platforms](#platforms). On Android the race has **touch controls** (a steering area, pedals, handbrake, gears, camera, pause) and the menus take taps and drags: [Android](Android).

## What is underneath

The original was a 32-bit Windows program on DirectX. Its platform side was not copied:

| Original | Here |
| --- | --- |
| DirectDraw and Direct3D | SDL3 for the window; races drawn on the GPU (SDL's GPU device, OpenGL, or OpenGL ES on the handheld) or by a software rasteriser ([Drawing a frame](Drawing-a-frame)). |
| DirectSound | The game's sound code over a small mixer on SDL's sound output ([Sound and music](Sound-and-music)). |
| DirectInput | SDL's keyboard, gamepad, joystick and haptic support ([Input](Input)). |
| DirectPlay | A protocol of this project's own ([Network messages](Network-messages)). |
| The system's film player | The project's own decoders in Rust ([Films](Films)). |
| Windows, 32 bits | Windows (32 and 64 bits), Linux, macOS, a handheld ([Platforms](Platforms)). |

## Graphics and the screen

- **Renderer.** The race is drawn on the GPU by default: SDL's GPU device first, then OpenGL 3.3,
  then the software rasteriser, each if the one before does not start. `--renderer gpu | gl |
  software` (or `RC2K_RENDERER`) chooses; the options' "advanced" page has a "renderer" row that
  swaps it at once on the desktop. On the handheld, OpenGL ES.
- **Mipmapping** (the graphics page's switch, off at first) now works on every renderer: far
  textures are drawn from smaller pre-reduced copies on the GPU renderers and the software one
  alike.
- **3D resolution** (graphics page): the race's 3D is drawn at 50, 75, 100 (the default), 125, 150,
  175 or 200 % of the screen's size. Below 100 it is enlarged; above 100 it is drawn larger and
  scaled down, which smooths the edges (supersampling). The dashboard and the texts stay at full
  size. The handheld no longer draws at half size by itself: set the row to 50 %.
- **Screen sizes.** The resolution list holds the monitor's own modes, the usual sizes of 4:3,
  16:10, 16:9, 21:9 and 32:9 that fit the desktop and the game's eight, each with its ratio. Any
  other size can be set with `custom_width` / `custom_height` in `var/rc2k.toml`. 16-bit screen
  modes are gone: the screen always has 32 bits a pixel (an old 16-bit choice becomes the same
  size at 32 bits).
- **Wide screens.** On a screen wider than 4:3 the 3D keeps its field of view up and down and
  shows more to the sides; the HUD keeps its 800 x 600 shape with its pieces in the corners and
  edges. The graphics page's "HUD area" row (the whole screen at first) confines the HUD to a
  centred 4:3, 16:9 or 21:9 box. With one player the progress bar along the top runs over the
  whole width whatever the box: its end caps, with the clocks, the car's icon and name, keep to
  the screen's edges and its middle stretches between them. With two players the one progress bar
  (shared at the top, or at the divider when one is above the other) does the same, and side by
  side each player's HUD lies in their own half of the screen as one player's lies on a wide
  screen (dial at the left corner, map at the right, the box applying inside the half); in two,
  three and four players' split screens each map disc shows the other players' cars as markers in
  their stage-bar colours (not the computer's), placed for its own player's car; three
  and four players' HUDs are the one-player HUD in each part, laid out by the part's own shape and kept inside the box. The course is not cut at the sides
  by the 4:3 view's cone. The loading screen, the roadside tyre change and the replay viewer keep
  their 4:3 shape in the middle: the loading picture has the menus' blurred wash beside it, the
  tyre change's dark backdrop and the replay viewer's bars span the whole width.
- **Screen row** (graphics page): window (the default), full screen or borderless. **Alt+Enter**
  toggles between a window and full screen at any time.
- **One window size.** The window keeps the race's screen size from start to end; the original
  switched the display between the menus and a race. The menus are still drawn at 800 x 600 and
  shown scaled in the window, centred, keeping the 4:3 shape. On a window of another shape the
  space beside the menu is the menu's own background picture, scaled to cover, blurred and
  darkened. The menus' 3D models (the main menu's key and signpost, the helmets, the car
  carousel) are drawn sharp at the window's size on the GPU renderers.
- **Skid marks.** A tyre mark is no longer laid twice in one place when the frame rate is above
  the simulation's 100 ticks a second (it showed as dark squares).
- **Helmets** on the in-race options page are textured.
- A **loading bar** is shown while a stage loads, and the loading picture is shown once.
- **Screenshots and snapshots are PNG files**, not PCX ([Screenshots](Screenshots)); the numbering
  and folders are as before.
- The rectangle fill the original never drew at 32 bits works, so message boxes have backs and
  the installer's percentage and the tyre change's texts are cleared before they are redrawn.
- The 3D resolution and screen options only change how a frame is drawn: the simulation, and so
  every replay, is the same.

## Frame rate

- **Frames follow the display's refresh**: one frame for each refresh (60 a second when the rate
  is not known), where the original slept a fixed 16 ms. The race still moves 100 times a second
  and its speed does not depend on the frame rate.
- **Smooth frames** (the advanced page, on at first): on displays faster than 100 Hz the frames
  between two of the race's ticks are drawn blended between the last two, so motion is smooth at
  120, 144 or 240 Hz. The picture is up to a tick behind the simulation. Rain and snow on the
  windscreen appear at most once a tick, so a faster display gets no more of them.
- **Frame limit** (graphics page, the display's own pace at first): no limit, the display's pace,
  30 or 60 a second, or the rates that divide the display's refresh (a half, a third, a quarter;
  with smooth frames the display's own rate and its half, up to 1000 a second).
- The program starts in under half a second: the pauses the 1999 start-up made are gone.

## Menus and settings

- **The installer keeps the folder's case**: a folder typed as `C:\Games\RC2k` is made and recorded
  with that spelling (the original lowered every letter); folders are still compared without case.
- **Advanced** (an item after the original's on the options page, and on the in-race options):
  rows for the renderer, the number of drawing threads, "smooth frames", "buttons in the quit
  question", and the switches that lived in `var/ral.cfg` shown as the feature they control:
  frame counter, films, screenshots, non-linear steering, automatic neutral, rain and snow, music
  and force feedback, and, on a second part of the page reached by its "more" button ("first page" comes back), "controls from saved games" (whether a loaded championship brings its own
  keys, controller set-up and gearing, or keeps the players' current ones) (some take effect at the next race or start, and the row says so). "Apply"
  writes them ([Running and options](Running-and-options), [Settings files](Settings-files)).
- **Settings are saved when a page is left**, not only when the game ends, so a crash keeps them.
  Options are kept in a readable text file, `var/rc2k.toml`; the original's `var/ral.dat` is
  read when the new file does not exist ([Options files](Options-files)).
- **The menu after a stage opens the options**: audio, display, advanced, and the players'
  details (names, country, camera, co-driver, units, keys, controller and buttons pages, visual
  preferences). The original offered fewer. The network lobby has the same page ("settings").
- **Film volume** (the volume page's last slider, above the music switch) sets the intro and logo
  films' sound; **sound when not in front** (the sound page, off at first) keeps the sounds going
  while the window is behind.
- **Network settings** have "port" and "host password" rows ([Network play](Network-play)).
- **Typing** (names, saved game, replay and set-up names, network settings, chat) takes any
  character the Windows-1252 code page has, whatever the keyboard's layout (input methods work too;
  the game's own keyboard layouts no longer type into fields).
- **Language texts** this game adds to the original's come from one file per language,
  `files/locale/<language>.rc2k.toml`, with English built in ([Language files](Language-files)).
- The fixed pauses between screens keep their length, but a key or a pad's button ends them and
  the window answers meanwhile; closing the window anywhere ends the game properly.
- The Portuguese build's texts: the tyre change's title is no longer cut at its ends.

## Controllers and input

Everything is on [Controllers](Controllers) and [Input](Input); in short:

- **The whole game works with a pad**: menus, questions, the credits, the replay viewer, the
  network lobby (the shoulders are Tab), any-key waits (South, East, West or Start). A held
  button repeats as a held key does.
- **Controller pictures.** Menus and screens show the buttons of the controller in use as pictures
  (Xbox, PlayStation or Steam Deck set, chosen from what SDL reports; numbers for plain
  joysticks). The controller page's "button icons" row switches them; the advanced page's "buttons
  in the quit question" row does so for the race's leave question and the tyre change's prompt.
- **Engine vibration switch.** The force feedback of a car nearly still follows the engine in the
  original; the controller page's "engine vibration" row (on at first, set for each player and
  controller) leaves that out, so a wheel feels
  only the road: [Controllers](Controllers#engine-vibration).
- **On-screen keyboard** for typing with a pad: below or above the field, one tile a key, the
  typed text in a line at the top (stars for a password), pages of symbols and accented letters.
  A system with its own screen keyboard shows that one instead.
- **Profiles.** Each player keeps a profile for every controller used: bindings and per-axis
  settings. The controller page has "axes" (a live bar for each axis; dead zone, range, response
  curve, invert, and a wheel's full rotation, 900 degrees at first) in place of the original's axis
  mode and dead zone items. The buttons page binds steering, accelerate and brake to any axis
  and the rest to buttons or hat directions, and has a last page of **menu controls** (enter, back,
  up, down, left, right, the third answer, Tab).
- **H shifter**: the "H shifter" switch gives gears 1 to 7 and reverse their own bindings; none
  held is neutral.
- **Learning a control** measures each axis from where it rested, so a pedal that rests at the
  end of its travel (a wheel's clutch) is not taken for a direction held.
- **Wheels** work in the menus: the hat moves, the second button enters, the third goes back.
- **Hotplug.** A controller plugged in while the game runs can be chosen and driven with at once;
  one unplugged and plugged back takes its place again. Force feedback effects are set up again.
- **Rumble.** A pad with only rumble motors rumbles for the engine and knocks, at the strength of
  the controller page's force feedback row.
- **Pause** (a key or a button) asks the same "leave?" question as Escape, where the original
  waited for a key.
- The developers' debug pages are a window over the game ([The debug window](The-debug-window));
  not on 32-bit Windows or the handheld.

## Sound

- The sound page's levels have names of their own in the options file (`general_volume`,
  `menu_music_volume`, ...); a file from an earlier build starts these from their defaults once.
- The game waits less on the audio thread, so a stage starts and a race ends sooner.
- The engine sounds play as the original's plain loops, and there is no reverb: both were tried
  and taken out.
- The game's CD audio is gone (the music is the game's files). The first logo film, `allogo.avi`,
  shown only on 16-bit screens in 1999, always plays.

## Racing and the HUD

- The **tyre change's** title and prompt are drawn at the font's own spacing, centred, with the
  button's picture on a pad; the second player's too in split screen.
- The **pause binding** (a button on a pad and a key on the keyboard) asks the same "leave?"
  question as Escape, in place of the original's plain pause that waited for a key. The race still
  pauses until a key when its window stops being the active one.
- The race's **leave question** is drawn at the timer's size on a dark plate, with the pictures of
  the buttons that answer it.
- The **windscreen Pong** can be played: the bat follows player 1's steering.
- The **computer drivers are shuffled** at a championship's start as the game meant, and a
  race's other **hidden slips** were put right ([Fixed bugs](Fixed-bugs)).
- A race can start straight from the command line ([Running and options](Running-and-options)).

## Replays

- **The viewer names the highlighted button.** The original's viewer has only pictures on its buttons. Here,
  when the highlight moves to another button (the arrows, the pad, and when the viewer opens on its play
  button), the button's name shows centred just under the render button, so it hides nothing of the panel, in the
  style of the car's warnings (their font and its dark outline, no box, as large relative to the screen on a
  5120 x 1440 window as on 800 x 600), for about a second, then fades out over a quarter of a second, on the software
  renderer too ("play", "back 1 second", "set the first
  mark here", "next car or layout", "save the replay", "render to video", ...). The names of the three buttons
  that change something say what pressing them does next: the HUD button of a car ("show car 1's HUD" or "hide
  car 1's HUD"), and the speed button ("replay speed: half"). While a camera is being edited the label tells
  the keys of the edit. Nothing shows while a replay plays, and a video rendered from a replay has no label. The
  texts are our own, in `english.rc2k.toml` and `portuges.rc2k.toml` (`viewer.tip_*`).
- **Replays can be rendered to a video**, with its sound: the viewer's "render to video" button
  (a third row under its two rows of buttons, showing the snapshot button's camera) opens a dialog
  in the error dialog's style before anything is rendered. Its rows, changed with Left and Right
  (Enter steps them on; the mouse clicks them): the video's **size** (the screen's own, or any
  resolution the graphics page lists: the game's sizes, the monitor's modes, the usual sizes that
  fit the desktop and a size set by hand), the **frames a second** (the rates of the graphics
  page's frame limit, and 30 and 60 whatever it lists), the **part** (the whole replay, from the
  place shown to the end, or between the viewer's marks when they are set), the **quality** (low,
  medium, high, best: the encoder's constant rate factor 28, 23, 18, 12), the **encoder** (automatic:
  a graphics card's H.264 encoder when one works (up to 4096 pixels wide), else its AV1 encoder,
  which makes a render about three times faster; or the software one only) and the **sound**. The
  folder and the file's name (`savedata/videos/replay_<date>_<time>.mp4`) are shown; they cannot
  be edited here (`--out` names any file from the command line). "Render" starts, "cancel" or Escape
  leaves the viewer as it was. The size, rate, quality and sound are kept in `var/rc2k.toml`
  (`[display.video]`) for next time; the part is not. The video is **drawn at the size chosen**
  (1440p or 4K are real 1440p and 4K frames, not an enlarged screen): the game starts a second copy
  of itself, with no window and no sound device, on a copy of the replay as it is viewed (edits
  included) in `savedata/videos/`, and the viewer waits behind a panel with a bar, the frames written, the speed, the time left
  and a cancel button. Cancelling (button, Escape or a click) ends the copy and removes everything it
  wrote; when it ends well a notice says the video is ready, and the viewer is where it was. The
  copy does not save the options or the best times. Without an `ffmpeg` on the path the frames are
  numbered pictures. From the command line, `--render-replay <slot|file> --out <video> [--size WxH]
  [--fps N] [--quality low|medium|high|best] [--no-sound] [--from SECONDS] [--to SECONDS]` does it
  without a window (the part ends at the whole second after `--to`). The video is written through an
  `ffmpeg` on the path (H.264 with AAC sound when it has them).
- **`--replay <slot|file>`** starts a saved replay at once, `--then menus` or `--then quit`.
- **Every race is recorded in a replay format of its own** (see [Replay files](Replay-files)): up to eight cars,
  all of them recorded, the computer's cars and the other machines' cars of a network race too (their motion is
  recorded as this machine showed it, so a network replay shows each car where it was and no longer drifts).
  Saving a replay writes it in this format (`replayNN.rc2r` in `savedata/replay`) and, when the original's format
  can hold the race (one or two players, none on another machine), also in the original's beside it, byte for
  byte as the original wrote it. The replays list marks a slot "old" after its name when it holds only the original's file, a new slot shows its name alone; a slot with both
  files loads the new one, and a slot with only the original's file (one copied in) loads that as it was. Old
  files play as they always did. A recording takes about 130 KB a minute for one car and up to about 2 MB a
  minute for eight cars of which seven are driven elsewhere. Every car of such a replay is drawn with its own model (the original's header names four), and in a network race's replay the followed car's headlights light the road. A replay of the new format can be 90 minutes long (the original's was cut at 30); its original-format twin is cut at 30 minutes.
- The viewer's **car item** goes through the **players** only (a race's human players, also those on other
  machines), never a computer car; the computer's cars are still drawn and drive in the replay. It also
  works for a network race's replay and for races of more than four players.
- A replay of **more than two cars** (three or four players on one machine, or a network race with other players
  and bots) is **heard with every car**: each car's engine, tyres, gear changes and knocks, the nearest loudest
  (heard from the followed car in one view). One- and two-player replays sound as before.
- A replay of a **three- or four-player split-screen race** can be watched **split as it was raced**: after the
  single players the viewer's car item offers "all", each player in the race's part of the screen with its HUD.
- A **network race records a replay** like a single race; the lobby then offers "save replay". A replay that
  cannot be saved or read shows a notice instead of ending the game.
- Old replays are read as they are, and replays saved here play in the same way on every system.
  A race of three or four players earlier wrote a replay of version `0x50401`; those are still read.

## Network play

Details on [Network play](Network-play). The protocol is this project's own, so the program plays
against other copies of itself and not against the original; TCP/IP is offered (no modem, serial
or IPX connection), and Steam when Valve's library is beside the program (it is loaded at run time, so there is no separate build).

- **Password**: a host can lock its session; the session list marks it "[locked]" and a joiner is
  asked for the password. It is checked without being sent.
- **Ping** is shown after each player's name in the lobby. **Un-ready**: the ready button becomes
  "not ready" and takes the ready back before the start.
- **Finding hosts**: with no address typed, the search also asks the peers of a Tailscale network
  and the last host joined, so a host over Tailscale is found with no address.
- **The host's stage** is shown to everyone, every machine's loading progress is listed while a
  stage loads, and "NN%" shows a racing player's progress in the lobby; the chat is in one order
  on every machine. The other cars appear on the progress bar.
- **A player who leaves a race** before finishing has their car taken out of it on every other machine and the others are told ("<name> left the race", in the style of the car's warnings: the race's message font with its outline, no box, blinking); a finished player's car and time stay. The map disc shows the other players' cars (not the bots'). The loading screen's list of progress is a framed box inside the picture, at the left and half way down ([Network play](Network-play)).
- **`/kick <name>`** in the chat: the host puts a player out.
- The lobby has a "settings" button and, after a race, "save replay".
- **The lobby's stage name** is on the line of the table's headings, in their font, not over the picture at the top left.
- **Computer drivers in network races**: the host's "bots" button in the lobby adds up to seven computer-driven cars to the race, as the arcade has them, from the grid or, with the "start" button, spread on the course like the arcade's opponents, half a sector apart; they are not on the map disc, the stage bar or the players' table (a label in the lobby tells every player how many there are); they keep driving while a machine is paused ([Network play](Network-play)).
- A port in use is said so; the failure of any start is written to the log.
- **Steam invitations**: accepting one (or "Join game" in the friends list) joins that lobby from any menu; during a race or replay it waits for the menus, and in a network game it is ignored.
- **Command line**: `--host`, `--join`, `--name` and `--port` go straight into the lobby.
- **Several copies on one machine** (`--allow-another`, or `--instance 1..8` with a folder of its
  own for each) play each other.

## Saves and files

- Every file the original wrote keeps its format: replays, saved games, car set-ups, best times,
  `var/ral.cfg`.
- **Save code**: the options menu's "save code" page shows the player's saved games, car set-ups and best
  times (not the settings, not the replays) as a text on the left and a QR code of it on the right, with
  buttons to copy the text, paste one from the clipboard, and read one from a camera. A code that comes in
  is checked whole, and the player is asked, with what is in it beside what is here, before anything is
  replaced. A QR code holds at most about 4,300 letters; a longer code is cut into numbered parts, one QR
  code each, stepped with Left and Right and read by the camera in any order. A saved game takes about
  1,900 letters ([Save code](Save-code)).
- **Errors.** An error that ends the game is shown on a panel in the game's window (the message,
  where the details are, "Quit"; Up / Down and Enter, or a pad's buttons) in place of the
  "Guru Meditation" box. Troubles belonging to one file no longer end the game: a damaged saved
  game is not loaded, a replay that cannot be read is skipped, and options that are not valid are
  reset to their defaults, each with a notice. So are a picture that cannot be read (shown as
  missing), a sound file that holds no sound the game can play (silent; a sound file that is simply not there stays silent with a line in the log only, as the stock game's lists name some it never shipped), and a
  replay whose data breaks while it plays (it stops there, or goes back as below); each is told once, in the language in use. A start-up film (the logos, the intro) that is missing or cannot be played is skipped with a line in the log only. A network
  player's message that cannot be understood is dropped, and the notice comes at the next menu
  frame. A stage whose data cannot be read, or another trouble the game can go back from inside
  a menu or a race, shows the panel with "Main menu" and "Quit" (a stage picture that cannot be read is one of them): the race is torn down as when
  the player quits it, and the game goes on at the main menu.
  `RC2K_NO_ERROR_DIALOG` turns the panel off (the game then goes back to the main menu without
  asking).
- The program writes `rc2k.log` beside the game's `var/logfile.txt` ([Log files](Log-files)).
- On Linux and macOS, file names are matched without regard to case, as Windows does.
- The web build can export its saved games, best times and settings as a zip and import them back.

## Running

- `rc2k --help` lists every switch; an unknown `--` switch stops the program with a message.
- **Straight into a race** (`--stage`, `--car`, `--mode` for time trials, the arcade game and the
  championships, `--launch <file>`), into a replay or into the network lobby, with no menus and
  no films.
- **A folder with no game data** opens a page asking for a folder or a zip (picked or dropped on
  the window), or takes `--game-data <folder|zip>`.
- A second copy in the same folder is refused unless `--allow-another` is given.
- Closing the window ends the game properly, saving the options and the session's records.

## Platforms

- Windows (32 and 64 bits), Linux, macOS and the Anbernic RG34XXSP handheld; see
  [Platforms](Platforms). The program carries the project's icon on every system.
- On macOS the installer makes a self-contained "Rally Championship.app" in Applications.
- Builds: `./build.sh <target>`, and `build.ps1` on Windows itself ([Building from
  source](Building-from-source)).
- The web build is experimental: it draws the 3D on WebGL 2 where the browser has it, takes
  gamepads and plays the sound; it has no network play.

## Installing

- No registry: the installation's record is `var/install.cfg`, so an installed folder can be
  moved or copied ([Install records](Install-records)).
- The installer takes the game from the folder it is started in, in place of asking for the CD,
  and copies it in seconds (the original's copy was slow here). It has no page for the kind of
  installation (always the full one), no DirectX set-up and no registration; it makes ordinary
  shortcuts. It does not copy the CD's `ral.exe`.
- "Uninstall Rally" runs the installed program itself (`rc2k -u`). More on [Installing the
  game](Installing-the-game).

## Not there

- Playing against the original program over a network.
- The original's choice between 3D cards, and what only served 1999's machines: CD audio, the
  raised priority (`noprichange`), the PC-98 keyboard layout, the search for 3D cards
  (`forceno3dcard`, `ignoreprimary`, `ignoresecondary`, `-c`), and the "andrew" computer-name check.
- The twin code paths chosen by processor features (MMX, SSE, 3DNow!): the game computes one way on
  every machine, and `ignoremmx`, `ignorekatmai` and `ignoreamdk3d` are read and not used.

## In progress

- Split screen for three and four players (the original had two): the single rally's players menu
  has an entry of its own for three and for four players on one screen, beside the original's
  in-turn ones (shown by the third and fourth players' helmets, as on the players' details page), and `--players 3` or `4` starts the same race. Each
  player has a view in their part of the screen, the controls of their own record, the one-player HUD
  in their part (stage bar with every player's marker in their colour, place, clock, rev dial, gear and speed, a map disc, each in a picture of its own redrawn as the player's car moves, with the other players' cars as markers; the signs for the first two players) laid out by the part's shape, every piece keeping its shape in a part narrower than 4:3 (one scale factor, the part's width, pieces at the top, bottom or middle of the part), a view that keeps the vertical field of view of a 4:3 one in
  a wide part (the car sits in a part as it sits on a one-player screen of that shape), each part's own car
  picture in the top bar (drawn at half size), the dust, smoke and sparks of every car drawn in the part of the view that
  sees them and cut to it, and a place on the three and four players' results page. The F2 key
  cycles the layouts: with four players quarters, four rows and four columns; with three one wide
  view above two, three rows, three columns and quarters with the whole course's map in the fourth
  quarter (every car a square in its colour on it); the choice
  is kept in `rc2k.toml` (`split_layout`). Every player's pause binding asks the leave question and each
  player's pad rumbles with that player's own car. Not yet: mirrors. Every player has the co-driver: the pace notes are called
  along each player's own car's progress, with a voice of its own for each player (the third and fourth players' voices
  are copies of the voice records, named by their own co-driver voice setting), and each player's pace-note signs
  show on their own part's HUD. Every player's car has its
  own engine, tyre, gear change and knock sounds, mixed together as the original mixes its two cars (each car
  heard at its own place, none placed by another player's view); the third and fourth cars' engine
  sounds are copies of their kinds' own, with a voice each. A race of three or four players keeps a replay (an extra tape for
  the third and fourth car, see [Replay files](Replay-files)); its viewer's car item also shows the
  third and the fourth car alone.
- Steam play has not yet been tried against a running Steam client.
