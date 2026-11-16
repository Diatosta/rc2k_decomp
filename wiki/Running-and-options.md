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

### Starting straight into a race

These switches skip the films and the menus and start a single rally stage, as if it had been
chosen in the menus. After the race come its results and then the menus, as usual.

```sh
rc2k --stage langton
rc2k --stage "hamiltons folly" --car felicia --gearbox manual --players 2 --seed 42
rc2k --stage 4-5 --stage-set arcade --then quit
rc2k --launch my_race.txt
```

| Switch | What it takes |
| --- | --- |
| `--stage` | The stage: its name, or `rally-stage` as two numbers in the stage screen's order, both counted from 1 (`4-5` is the fourth rally's fifth stage). A launch needs this one; the others are optional. |
| `--stage-set` | The stage screen's set: `simulation` (the default) or `arcade`, or `1` / `2`. The arcade set runs each stage the other way. |
| `--car` | Player 1's car, by name. Without it, the car the player last chose. |
| `--car2` | Player 2's car. Without it, the same as `--car`. |
| `--gearbox` | `manual`, `semi-automatic` or `automatic`, or `1` .. `3`. |
| `--players` | `1`, or `2` for a split screen. |
| `--seed` | A number (decimal, or hexadecimal with `0x`): the race's random numbers start from it, so two runs with the same seed get the same random events. |
| `--then` | `menus` (the default) or `quit`: with `quit` the program saves and ends when the race is over. |
| `--launch` | A file holding the same settings. Switches typed on the command line as well win over the file's. |

Each switch can also be written as `--name=value`.

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

**Refusals.** If a name fits nothing, or fits more than one, the program does not guess. It
stops before the race loads, with a message (in a box, and in `rc2k.log`) that lists the names
to choose from. The same happens for a value that cannot be read, such as `--players 3`.

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
```

A line that is not `name = value`, or names a setting that does not exist, stops the program
with the file's name and the line's number.

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
the game's own options screens and saved when the program ends properly.

| File | What it is |
| --- | --- |
| `var/rc2k.toml` | This program's options file: text, one table per area (`display`, `sound`, `network`, `players` and others). Read at start, written when the options are saved. |
| `var/ral.dat` | The original program's options file. Read only when `var/rc2k.toml` does not exist yet (its values are brought over and the new file is written at once), or with `--load-old-options`. Written only with `--save-old-options`. |
| `var/ral.cfg` | Switches as `name=value` lines, read at start and never written by the game: the log level, skipping the films, physics tuning. |

The layouts are on [Options files](Options-files) and [Settings files](Settings-files).

## Environment variables

Two are useful to anyone:

| Variable | Effect |
| --- | --- |
| `RC2K_RENDERER` | `software` draws with the software rasteriser instead of the GPU. `gpu` (or nothing) is the default: the GPU first, the rasteriser if no GPU renderer starts. On the handheld `gles` means the same as `gpu`. An unknown value is noted in the log and ignored. |
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
| `RC2K_FULL_RESOLUTION` | Handheld, software rasteriser: draw races at full size instead of half size. |
| `RC2K_GPU_CUT` | A list of names (`fog`, `alpha`, `tex`, `blend`, ...): each removes one kind of GPU work, to see what it costs. The picture is wrong under a cut. |
| `RC2K_GPU_TIME` | A number n: every n-th frame, measure how long the GPU still needed after the last command. |
| `RC2K_GL_BENCH`, `RC2K_BREAKS_DUMP`, `RC2K_SEETHRU_AREA`, `RC2K_SEETHRU_DUMP` | Measurements of the handheld's OpenGL ES drawing. |
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
