# Running and options

*Category: [Getting started](Getting-started)*

How to start the program, what can be written on its command line, where its options are
kept, and where to look when something goes wrong.

## Starting it

Start the program **in the game's folder**, the one that holds `files` and `var`. The menu
entries the installer makes do that for you. By hand:

```sh
cd /home/sam/games/rally        # your game's folder
./rc2k                          # Linux; on Windows, rc2k.exe
```

On macOS, open `Rally Championship.app`; it works in the folder that holds the bundle.

The program opens its window, plays the opening films and shows the main menu. It starts
quickly: the pauses the 1999 program made while starting (waiting on the sound card, timing
the processor) are gone.

If the program is started among the CD's files and there is no installation there, it runs the
installer instead ([Installing the game](Installing-the-game)).

## The command line

```
rc2k [log file] [switches]
```

The first word that is not a switch names the program's log file. Without one the log is
`rc2k.log` in the current folder.

`rc2k --help` (or `-h`) prints every switch with a line on what it does, then ends. On Windows
it prints into the terminal the program was started from; started without one, it shows the
list in a message box. A switch starting with `--` that the program does not know ends it at
once with a message naming the switch, before anything loads: a mistyped switch is not passed
over silently. The list comes from one table, `rc2k_core/src/cli.rs`; a switch added to the
program goes there too.

### Starting straight into a race

These switches skip the films and the menus and start an event, as if it had been chosen in the
menus: by default a single rally stage, or with `--mode` a time trial, the arcade game or a new
championship. After the race come its results and then the menus, as usual.

```sh
rc2k --stage langton
rc2k --stage "hamiltons folly" --car felicia --gearbox manual --players 2 --seed 42
rc2k --stage 4-5 --stage-set arcade --then quit
rc2k --launch my_race.txt
rc2k --mode time-trial --stage langton --car felicia --then quit
rc2k --mode arcade --level 3 --car puma
rc2k --mode a8 --players 2 --car octavia --car2 puma
```

| Switch | What it takes |
| --- | --- |
| `--mode` | The kind of event: `single` (the single rally, the default), `time-trial`, `arcade`, `championship` (the first championship, "Mobil One") or `a8` (the A8 championship). Without `--stage` a launch is refused for the single rally and the time trial; the arcade and the championships bring their own stages. |
| `--level` | The arcade's level, `1` .. `6`, counted as the arcade's level screen shows them (`1` unless given: the screen's start item, then its five level items). A level not opened yet by winning the one before it (or by the Unlock Arcade cheat) is refused, as the screen refuses it. Only with `--mode arcade`. |
| `--stage` | The stage: its name, or `rally-stage` as two numbers in the stage screen's order, both counted from 1 (`4-5` is the fourth rally's fifth stage). The single rally and the time trial need this one; the others are optional. |
| `--stage-set` | The stage screen's set: `simulation` (the default) or `arcade`, or `1` / `2`. The arcade set runs each stage the other way. |
| `--car` | Player 1's car, by name. Without it, the car the player last chose. |
| `--car2` | Player 2's car. Without it, the same as `--car`. |
| `--car3`, `--car4` | Players 3 and 4's cars (with `--players 3` or `4`). Without them, the same as `--car`. |
| `--gearbox` | `manual`, `semi-automatic` or `automatic`, or `1` .. `3`. |
| `--players` | `1`, or `2` for a split screen (in a championship too), or `3` or `4` for the single rally only (all the players on one screen, as the players menu's "on one screen" entries start it). The arcade has one player. |
| `--seed` | A number (decimal, or hexadecimal with `0x`): the race's random numbers start from it, so two runs with the same seed get the same random events. |
| `--then` | `menus` (the default) or `quit`: with `quit` the program saves and ends when the race is over (the first race of a championship or of the arcade game). |
| `--launch` | A file holding the same settings. Switches typed on the command line as well win over the file's. |

Each switch can also be written as `--name=value`.

**The time trial's tune** (only with `--mode time-trial`; each one not given stays as it is):

| Switch | What it takes |
| --- | --- |
| `--tyres` | `slicks`, `intermediate`, `wets`, `dry-gravel`, `wet-gravel` or `snow`, or `1` .. `6` in that order. |
| `--gear-ratio` | `1` .. `5`: the choices the service area offers, counted from the first. |
| `--ride-height` | `0` .. `100`: the slider's position (the number the set-up files hold). |
| `--suspension` | `0` .. `100`: the suspension stiffness slider's position. |
| `--brake-balance` | `0` .. `100`: the brake balance slider's position. |
| `--steering` | `0` .. `100`: the steering sensitivity slider's position. |

```sh
rc2k --mode time-trial --stage langton --car felicia --tyres dry-gravel --steering 60 --then quit
```

**Names.** Stages, cars, gearboxes and stage sets are named in English, whatever language the
game itself runs in. Names are compared without regard to capitals, spaces or punctuation, so
`hamiltons-folly`, `"Hamilton's Folly"` and `hamiltonsfolly` are the same.

- A name typed in full always wins.
- Otherwise a part of a name is enough when only one name contains it: `--car 106`,
  `--car "evo v"`, `--stage hamiltons`.
- Cars go by name only. A number given to `--car` is taken as a part of a name (`106`),
  because a car's place in the list changes with what is unlocked.
- A car that is not unlocked yet can still be named: it is taken from the list of every car,
  as the game's "Unlock Cars" cheat shows it.

**Events.** Each event runs the menus' own flow for it, with the launch's answers to the car and
stage choices; the rest is the player's, as in the menus.

- Single rally and time trial: `--stage` is needed. Two players (`--players 2`) share a split
  screen; the single rally takes `--players 3` and `4` too (for now the screen shows players 1
  and 2 only).
- Time trial: the service area is not shown. The car is left as the player's record has it,
  plus the tune the switches below give (the same for both players). No repairs are made.
  The set-up values are put in as the service area's choices end up: brake balance and steering
  sensitivity are the car's own values; a tyre, gear ratio, ride height or suspension stiffness
  that differs from the car's is chosen and the service area's own work is carried out without
  showing it (new tyres reset the tyre wear). The service time allowance does not limit a
  launch's tune.
- Arcade: `--level` (1 .. 6), `--car`, `--gearbox`. The level's six stages come in their order and a
  failed race retries the level, as in the menus. `--stage`, `--stage-set`, `--car2` and
  `--players 2` are refused: the arcade has fixed stages and one player.
- Championship and A8: a new championship, with the car choice answered; its welcome and the
  screens before the first stage are the player's. `--stage`, `--stage-set` and `--level` are
  refused. A launch never starts or overwrites a saved championship: a saved game is written
  only by the save menu, which a launch does not open, and a launch always begins a new
  championship (to go on with a saved one, load it from the menus).
- `--then quit` ends the program after the first race of any event.

**Refusals.** If a name fits nothing, or fits more than one, the program does not guess. It
stops before the race loads, with a message (in a box, and in `rc2k.log`) that lists the names
to choose from. The same happens for a value that cannot be read, such as `--players 5` or
`--mode rally`, and for a switch the event has no use for, such as `--stage` with
`--mode arcade` (the message says why).

**A launch file** holds one `name = value` line per setting, with the switches' names without
the dashes. `#` starts a comment. For example, `my_race.txt`:

```
# an evening's test race
stage     = langton
stage-set = simulation
car       = octavia
car2      = puma
gearbox   = automatic
players   = 2
seed      = 0x2A
then      = menus
mode      = single
```

A line that is not `name = value`, or names a setting that does not exist, stops the program
with the file's name and the line's number.

### Starting straight into the network lobby

`--host` and `--join` skip the films and the menus and go to the network game's lobby, as if the
network menu's host or join item had been chosen (the settings screen and the connection list
are not shown). In the lobby everything goes on as usual: cars, ready, chat, the host chooses
the stage and starts the race. When the lobby is left the menus come up. See
[Network play](Network-play).

```sh
rc2k --host --name Ann
rc2k --join 192.168.1.20 --name Bo
rc2k --join game.example.org:47700 --then quit
rc2k --host --port 47700
```

| Switch | What it takes |
| --- | --- |
| `--host` | Host a session. It takes no value. |
| `--join` | The host to join: its name or its IPv4 number, with `:port` after it if the port is not the usual one. The session found at that address is joined as soon as it is listed; if the host is not up yet, the program waits for it (Escape gives up). |
| `--name` | Your name in the lobby, up to 30 letters, digits, spaces or signs of ASCII. Without it, player 1's name. For this run only. |
| `--port` | The TCP/IP port, 1 .. 65535, the same on every machine. Without it, the saved one (`port` in the `network` table of `var/rc2k.toml`) or 47624. For this run only; not saved. |
| `--then` | As for a race: `menus` (the default) or `quit` when the lobby is left. |

No race or replay switch goes with `--host` or `--join` (a lobby chooses its own race), and
`--host` does not go with `--join`; `--name` and `--port` need one of them. A malformed address
(`--join "a b"`, an IPv6 address, a number above 255, a port outside 1 .. 65535) stops the
program with a message. In a launch file the lines are `join = host`, `name = ...` and
`port = ...`; `--host` is a switch only.

### Starting straight into a replay

`--replay` skips the films and the menus and opens the replay viewer on a replay, as if it had
been chosen in the replays menu. The replay starts playing at once (in the menus the viewer
waits for Enter on its play button; nothing else differs: the same viewer, the same cameras).

```sh
rc2k --replay 0                       # savedata/replay/replay00
rc2k --replay 17 --then quit          # savedata/replay/replay17, then the program ends
rc2k --replay ~/kept/my_best_run      # a replay file anywhere
```

| Switch | What it takes |
| --- | --- |
| `--replay` | A number, `0` .. `99`: a saved replay's slot. Slots are counted from 0, as the replays menu counts them (its pages are captioned `0-19`, `20-39`, ...) and as the files are named: slot `7` is `savedata/replay/replay07`. Anything that is not a number is a replay file's path (a file whose name is a number needs its folder: `./12`). |
| `--then` | `menus` (the default): the viewer stays open when the replay has played, and leaving it leads to the main menu. `quit`: the program saves and ends when the play stops, at the replay's end or at a key. |

A replay holds its own stage, cars and random numbers, so none of the race's switches goes
with `--replay`: together with `--stage`, `--car` or another of them the program stops with a
message. In a launch file the line is `replay = 0`.

A saved replay is copied over the last race's replay (`var/tmprpl`) before it plays, as the
menus do it. A replay file given by its path is opened where it lies and is not changed;
saving it from the viewer asks for a slot.

If the slot holds no replay, or the file is not a replay of this game, the program stops
before anything loads, with a message that says why and lists the saved replays by slot and
name.

### Rendering a replay to a video

`--render-replay` plays a replay without a window on the screen or a sound device, as fast as
the frames can be drawn, and writes the picture and the game's sound as a video file. The replay
runs tick for tick as when it is watched; only the clock is the video's own.

```
rc2k --render-replay savedata/replay/replay00 --out run.mp4 --size 1920x1080 --fps 60
```

| Switch | What it does |
| --- | --- |
| `--render-replay` | A replay's slot or file, as `--replay` takes it. The program ends when the replay has played. No race switch and no `--replay` goes with it. |
| `--out` | The video file. Needed. |
| `--size` | The size of the video's frames, `WIDTHxHEIGHT` (each side 160 .. 16384): the race screen is made this size for the render, whatever the options hold. Without it, the race size of the options. |
| `--fps` | The frames a second, 1 .. 480 (60 if not given). The simulation ticks a hundred times a second; a frame between two ticks shows the blend of them (the options' "smooth frames", forced on here), so 60, 120 or 144 are all smooth. |

The frames are piped as raw pixels to an `ffmpeg` found on the path: H.264 (`libx264`, 4:2:0, an
odd side is cut by one pixel) when it has that encoder, else its own default for the file's
extension. With no `ffmpeg` the frames are written as numbered PNG files in a folder named
after the video (`run_frames/frame_000001.png`, ...) and the log says so; `ffmpeg -framerate 60
-i run_frames/frame_%06d.png run.mp4` makes the video from them; the sound is
`run_frames/audio.wav` (16-bit stereo, 44100 Hz).
`--renderer` chooses the renderer as always: the software renderer needs no graphics card, and
the GPU renderers (`gpu`, `gl`) render at any size too (1920x1080 tried).
The program uses SDL's offscreen video driver unless `SDL_VIDEODRIVER` is set.

**The sound.** It is what the game's mixer makes, taken in step with the frames, so it matches the
picture whatever speed the render runs at: after each frame the audio thread takes one turn (every
sound the race asked for) and the mixer is asked for that frame's stretch of time (44100 samples a
second, 735 a frame at 60 frames a second). The samples go to a WAV file beside the video
(`run.sound.wav`) and, when the frames are done, `ffmpeg` joins it to the picture without
re-encoding the picture (AAC sound, Opus in a `.webm`) and the two parts are removed. If the join
fails the video is kept without sound and the log says so. A render never changes the replay:
the sound is only read from the mixer.

**From the replay viewer.** Under the two rows of buttons the stopped viewer has a third row with
one button, the snapshot button's camera picture. The down arrow from the lower row reaches it
(up or left goes back to the buttons) and
Enter renders the whole replay from its start, asking nothing: the screen's size, 60 frames a
second, into `savedata/videos/replay_YYYY-MM-DD_HH-MM-SS.mp4`, with the sound, as above. A green bar
along the foot of the screen shows how far it has come (it is not in the video). Escape stops the
render and keeps what was written. The viewer is back where the play ended.

### How the program runs here

| Switch | Effect |
| --- | --- |
| `--renderer <gpu \| gl \| software>` | What draws the 3D. `gpu` (the default): the GPU API first, then OpenGL 3.3, then the software rasteriser, each if the one before does not start. `gl`: OpenGL (desktop only), the rasteriser if it does not start. `software`: the rasteriser only, which needs no GPU and uses several processor cores. On the handheld `gles` is another name for `gpu`. Any other value stops the program before anything loads, with the names that are valid. The switch wins over the environment variable `RC2K_RENDERER`, and that over the renderer the advanced page keeps in `var/rc2k.toml`. A renderer chosen on the advanced page while the game runs is swapped in at once (desktop) and then outranks all of these until the program ends. The log's `render:` lines say which renderer started. |
| `--allow-another` | Start although another copy of the program already runs in this folder. Without it the second copy stops at once with a message that names this switch. |
| `--instance <1 .. 8>` | Run as that numbered instance of the folder, beside the others, with nothing written by two at once: its own lock (`rc2k.lock`, `rc2k.lock.<n>`; a taken one stops the start), log (`rc2k-<n>.log`) and, from 2 up, its own `var` and `savedata` folders in `instances/<n>/` (the options, best times, saved games, replays, snapshots, the game's log and the debug window's layout). The first start of an instance copies the folder's `var` files (not the game's log) so it begins with the same options; its `savedata` begins empty. Shared and read only: the game's `files`, films and languages. Shared and to be told apart by the player: the network port (give each hosting instance its own with `--port`), the controllers and the sound device. `--instance 1` is the folder's own folders. |
| `--game-data <folder \| zip>` | Where to take the game's data from when the program's folder has none. Without it, the first-run page opens (see below). |

**First run.** A program started in a folder with no game data (no `files/d/gfx/jade.bun`) opens a
page that asks for the game: pick a folder or a zip file on disk, or drop one on the window
(nothing is downloaded). A folder that holds an installed game (its `files`, `var` with the
installation record) is entered and the game runs in it as it is; a folder in the download / CD
layout is entered and the installer runs on it; a zip is first unpacked into
`rally-championship-source` in the user's data folder (`$XDG_DATA_HOME`, `%LOCALAPPDATA%`,
else `~/.local/share`) and then treated as a folder. The game's folder may be one level inside
what was picked. Zips over 4 GB (zip64), encrypted ones and a name that would leave the unpacking
folder are not read. The web build has its own page in the browser.

**Two copies in one folder.** The first copy holds a lock on `rc2k.lock` in the game's folder
while it runs (the system lets it go when the program ends, even if it crashes); a copy that
finds the lock taken stops. With `--allow-another` the copy starts beside it, takes the next
number (`rc2k.lock.2`, up to eight copies) and, when the log's name was not given, writes
`rc2k-2.log` instead of `rc2k.log`, so it does not empty the first copy's log. What the copies
share, and what to expect:

- The options (`var/rc2k.toml`) are written when a copy ends: **the last copy to quit wins**,
  and what the other copies changed in the options screens is lost.
- Saved games, replays, best times and the game's own log (`var/logfile.txt`) are the same
  files for every copy: a save made in one is not seen by a copy that has already read the
  folder, and the later write replaces the earlier. The last race's replay is the exception: a
  copy beside the first (`--allow-another`) keeps it in `var/tmprpl.<number>`, so two copies'
  races (a network race on one machine, say) never write one file.
- Only one copy at a time can host a network game on the default port; the others can join it
  (this is how two copies of the game play each other on one machine).
- A second window needs its own place on the screen and its own sound: the copies do not
  coordinate either.

For two copies that must not touch each other's options and saves, use two game folders.

### The options files' switches

| Switch | Effect |
| --- | --- |
| `--load-old-options` | Read the options from the original's file, `var/ral.dat`, whatever the new file holds. |
| `--save-old-options` | Write the original's file as well when the options are saved. |

See "Options" below.

### The original's switches

The 1999 program's own switches are still read. A switch is recognised by its first two
characters.

| Switch | Effect |
| --- | --- |
| `-v` | Shows the version in a box and ends. |
| `-a` | No sound. |
| `-u` | Uninstalls the game in this folder ([Installing the game](Installing-the-game)). |
| `-p`, `-i` | Start an installation without the installer's pages; the installer's own way of calling the program. Not meant for use by hand. |
| `-c 1`, `-c 2` | In 1999, told the game to pass over the second or the first 3D card. Still read, but today's renderers do not choose between cards. |

## Options

The game's options (screen modes, sound volumes, players, controls, unlocked cars) are set in
the game's own options screens and saved when each of those pages is left (and when the program
ends), so a crash or a killed process keeps the changes made in the pages already left. The review
menu after a stage opens the options too: audio, display, the players' details (names, controllers,
buttons, visual preferences) and the cars' specifications; what a race fixes (the number of players,
the cars) is not changed there.

| File | What it is |
| --- | --- |
| `var/rc2k.toml` | This program's options file: text, one table per area (`display`, `sound`, `network`, a table for each player and others), the values readable (names as texts, the saved connection as a GUID). Read at start, written when the options are saved. |
| `var/ral.dat` | The original program's options file. Read only when `var/rc2k.toml` does not exist yet (its values are brought over and the new file is written at once), or with `--load-old-options`. Written only with `--save-old-options`. |
| `var/ral.cfg` | Switches as `name=value` lines, read at start and never written by the game: the log level, skipping the films, physics tuning. |

The layouts are on [Options files](Options-files) and [Settings files](Settings-files).

## Environment variables

Two are useful to anyone (`RC2K_RENDERER` has a switch of its own, `--renderer`, which wins):

| Variable | Effect |
| --- | --- |
| `RC2K_RENDERER` | Same as `--renderer`, which outranks it: `software` draws with the software rasteriser instead of the GPU. `gl` asks for OpenGL (desktop). `gpu` (or nothing) is the default: the GPU API first, then OpenGL, then the rasteriser. On the handheld `gles` means the same as `gpu`. An unknown value is noted in the log and ignored. |
| `RC2K_MONITOR_SIZES`, `RC2K_DESKTOP_SIZE` | `WxH` lists (comma separated) and one `WxH`: stand in for the display's modes and desktop size in the graphics page's resolution list, for trials on a machine without that display. |
| `RC2K_NO_ERROR_DIALOG` | Any value: an error that ends the program is not shown on a panel waiting for a key; the program ends at once, as before the panel existed. For unattended runs. |
| `RC2K_FRAMES` | A number: the program ends itself after that many pictures and leaves the last one in `rc2k_frame.raw`. For quick automated checks. |

The rest are diagnostics for people working on the code. They change nothing unless set.

| Variable | Effect |
| --- | --- |
| `RC2K_PROFILE` | Starts a sampling profiler. Windows: samples go to `rc2k_profile.txt` beside the log. Handheld: `profile.txt` every 5 seconds. |
| `RC2K_MIRROR_BESIDE`, `RC2K_PLAYERS_BESIDE` | `0` turns off drawing the mirror (or the two players' views) on separate threads; any other value turns it on. By default it is on with two or more processor cores. |
| `RC2K_CAR_PART` | `0` turns off drawing the player's car as a separate piece of its view. |
| `RC2K_DRAW_SERIAL` | Draws the threaded parts one after another on the game's thread, to tell threading problems from merging problems. |
| `RC2K_MIRROR_ALWAYS` | Draws the rear-view mirror in every view, replays included, whatever the options say. |
| `RC2K_2D_QUADS` | `0`: the menus' 2D is uploaded as pixels instead of being drawn by the GPU as rectangles. |
| `RC2K_FAST_PATH` | Handheld: `0` draws every batch with the alpha test and the blend as the game set them, also where they change no pixel. |
| `RC2K_SHADER_KINDS` | Handheld: `0` draws the 3D with one shader program that reads each batch's choices (texture, fog, alpha test, second pass) for every pixel, instead of a program built for each kind of batch. The picture is the same either way. |
| `RC2K_2D_CHECK`, `RC2K_QUAD_CHECK`, `RC2K_CANVAS_CHECK`, `RC2K_BACKGROUND_CHECK`, `RC2K_CHECK_SAME`, `RC2K_FRAME_VIEW_CHECK` | Self-checks of the drawing: the picture is made two ways and compared, with differences logged. |
| `RC2K_COMPARE_FRAME`, `RC2K_COMPARE_OUT` | A file of a frame to compare the GPU's picture with, and where to write the result. |
| `RC2K_DUMP` | Writes a frame's drawing commands to a file. |
| `RC2K_GPU_CUT` | A list of names (`fog`, `alpha`, `tex`, `blend`, ...): each removes one kind of GPU work, to see what it costs. The picture is wrong under a cut. |
| `RC2K_GPU_TIME` | A number n: every n-th frame, measure how long the GPU still needed after the last command. |
| `RC2K_GL_BENCH`, `RC2K_BREAKS_DUMP`, `RC2K_SEETHRU_AREA`, `RC2K_SEETHRU_DUMP` | Measurements of the handheld's OpenGL ES drawing. |
| `RC2K_MIX_WAV` | With a remote build: the sound mixed in a race (fixed 20 ms turns, as `RC2K_MIX_HASH`) written to this WAV file (16-bit stereo, 44100 Hz), to listen to without an audio device. |
| `RC2K_SIM_HASH`, `RC2K_SIM_CHUNKS`, `RC2K_FAN_HASH`, `RC2K_SOUND_HASH`, `RC2K_MIX_HASH`, `RC2K_COURSE_HASH` | Log a running hash of the simulation, the drawing commands, the sound commands, the mixed sound or the loaded course, to compare two runs. |
| `RC2K_DIR` | Web build only: the folder the game's files are in. |

Builds made with the `remote` feature (for automated tests, see [Testing](Testing)) read a few
more: `RC2K_REMOTE`, `RC2K_REMOTE_STICK`, `RC2K_HELD_RACE`, `RC2K_SPEED`, `RC2K_SLOW_FRAMES`,
`RC2K_SHOT_CLOCKS`, `RC2K_MANGLE`, `RC2K_LOAD_TIMES`, `RC2K_LOSE_FRAMES` and
`RC2K_FAIL_RENDERER`. Ordinary builds ignore them.

## The logs

| File | What it holds |
| --- | --- |
| `rc2k.log` (or the name given on the command line) | The program's own log: which renderer started, what a launch asked for, and why the program ended if it ended on an error. Written anew at each start. |
| `var/logfile.txt` | The game's log, as the 1999 program wrote it: the date, the version, what it found out about the machine, then the game's own messages. Written only when the `logfile` setting in `var/ral.cfg` turns it on. |

See [Log files](Log-files) for more.

### Reporting a problem

Send:

- `rc2k.log` from the run that went wrong (it is overwritten at the next start, so copy it
  first), and `var/logfile.txt` if there is one;
- your system, and which build you used (a release, or the commit you built);
- what you did. If it happened in a race, the last race's replay, `var/tmprpl`, is often enough
  to play the problem back ([Replay files](Replay-files)).

When the game stops on an error of its own it shows a report with an error number; the same
number and text are in `rc2k.log`.
