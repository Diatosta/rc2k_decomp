# How the game runs

*Category: [Game flow](Game-flow)*

This page follows the program from the moment it starts, through the menus, into a race and
back out, and says where in the source each step lives. It is a map for finding your way
around, not a description of every function. The pages linked from it go into each part.

Module paths are written as in the code: `race::replay` is the file `src/race/replay.rs`, and
`drive::race_loop` is `src/drive/race_loop.rs`. Most files hold one function, named after the
file, with a comment at the top that says what it does and who calls it. Reading those comments
is the quickest way to learn a part.

## The three crates

| Crate | What is in it |
| --- | --- |
| `src/` (`rc2k_decomp`) | The game: its state, its flow, drawing, sound, input, and the platform layer underneath. |
| `rc2k_core/` | The game's rules as plain functions with no state of their own and no platform: sizes, formulas, table lookups, file layouts, the event flows. It has no unsafe code and its tests run anywhere. |
| `rc2k_derive/` | Small derive macros used by the state types. |

A function in `src/` often gathers values from the game's state, calls a function in
`rc2k_core/` to decide something, and stores the result. When a comment says the logic is in
`rc2k_core::...`, the rule itself is there.

## The state: owners

The game's variables are not globals. They live in twelve structs called **owners**, one per
area, each defined in its module's `owner.rs` and collected in `src/owners.rs`:

| Owner | Holds |
| --- | --- |
| `Race` (`src/race/owner.rs`) | Everything about the race in progress: cars, physics, course, views, replay, HUD, effects. The largest by far. |
| `Menus` (`src/menu/owner.rs`) | The front end: screens, players, championships, results. |
| `Scene` | The 3D scene: objects, textures, the drawing's working memory. |
| `Gfx` | 2D graphics: the screen, pictures, fonts, the 3D layer's state. |
| `Sound`, `Audio` | The game's side of the sound, and the audio thread's side. |
| `Input` | Keyboard, controllers, force feedback. |
| `Net` | Network play. |
| `Saves` | Options, saved games, best times. |
| `Strings` | The language's texts. |
| `System` | Session flags (quitting, window active), clocks, file work buffers. |
| `Install` | The installer and the installation record. |

They are built once at start-up (`Owners::build`) and never freed. `Owners` is a small struct of
references to all twelve; it is passed down the call chain, and a function takes only the owners
it needs. If you want to know where a variable lives, find its owner's struct and follow the
fields. More in [State and memory](State-and-memory).

## Start-up

`src/bin/rc2k.rs` is `main`. It picks the log file's name (`rc2k.log` unless another is given)
and calls `standalone::run` (`src/standalone.rs`), which never returns.

`run` sets up the platform side (SDL's input, the log, the fault reporter), builds the owners,
and calls `start`. `start` is a plain list of steps, each a function further down the same file:

| Step | What it does | Where the work is |
| --- | --- | --- |
| `set_up_fixed_objects`, `seed_clock_and_random` | Links a few fixed records, notes the start time, seeds the random numbers. | `src/menu/`, `src/system/`, `src/math/` |
| `check_machine` | Reads what the game wants to know about the computer (processor features, keyboard). | `src/system/` |
| `read_settings_and_start_log` | Reads `var/ral.cfg`, starts the game's log `var/logfile.txt`. | `src/settings/`, `src/logfile/` |
| `make_save_folders` | Makes `savedata/...` if missing. | `src/standalone.rs` |
| `read_command_line_and_install_state` | The command line's switches, the install record. | `system::read_command_line`, `src/install/` |
| `load_language_and_game_data` | Texts, options, best times, the lists of saved games and replays. | `strings::load_language`, `system::load_game_data`, `src/saves/` |
| `probe_display` | Lists the display's screen modes. | `src/gfx/` |
| `start_workers` | Network set-up; starts the game's threads (below). | `net::set_up`, `threading::start_threads` |
| `choose_screen_modes` | Picks the menu and race screen modes, prepares pixel conversion. | `src/platform/mod.rs`, `src/gfx/` |
| `run_main_loop` | Runs the game until the player quits. | `system::main_loop` |
| `shut_down` | Closes the network, restores settings, shows a fatal report if there is one. | `src/standalone.rs` |

The original program paused for about a second in several places here and timed the
processor's clock at every start. Those waits are gone: the steps run one after another without
sleeping, and the main loop looks for its window every few milliseconds instead of waiting a
full second first.

`system::main_loop` is short. If the program was started from the game's download folder, or
asked to uninstall, it runs the installer (`install::run_installer`, see
[The installer](The-installer)) and leaves. Otherwise it runs the front end
(`menu::run_front_end`), and when that returns it saves the options and best times
(`saves::save_game_data`).

## The front end

`menu::run_front_end` builds the fonts, plays the opening films (`menu::play_films`, decoded in
`src/platform/film.rs`) and enters the main menu. It returns when the player has confirmed
leaving the game.

Every menu screen is the same loop, in `src/menu/screen_loop.rs`. A screen is a value of type
`Screen` that describes what is particular to it: its menu, its background picture, steps to run
before and after. Each screen has its own file, named `..._loop.rs`
(`src/menu/options_loop.rs`, `src/menu/select_car_loop.rs`, ...), which holds that description,
and a name in `ScreenId` (`src/menu/screen_id.rs`).

Each frame, the loop:

1. lets the platform handle window and input events,
2. draws the screen (the routines that lay out screens are in `src/screens/`, text drawing in
   `src/text/`, 2D drawing in `src/gfx/`),
3. reads a key (`menu::handle_keys`),
4. on Enter, runs the highlighted item's action (`menu::run_item_action`). An action usually
   opens another screen (a nested call of the same loop) or starts an event.

So the menus are a tree of nested calls: the main menu's loop calls the options screen's loop,
which calls the controls screen's loop, and Escape returns one level. More in
[The front end](The-front-end).

### Skipping the menus

Started with `--stage` and its companion switches (see [Running and options](Running-and-options)),
the program skips the films, and the main menu's set-up runs a single rally with the car and
stage choices already answered (`menu::launch`; the switches are read in
`rc2k_core::menu::launch`). The race, its results and the menus afterwards are the same code
the menus use. `--replay` runs the replay viewer on a saved replay from the same step.

## From the menus into a race

Choosing an event (a championship, a single rally, a time trial, an arcade game) starts an
**event flow**: a function that walks through the event's screens and races in order. The flows
themselves are in `rc2k_core/src/menu/` (`rounds.rs` for the single rally and the time trial,
`championship.rs`, `arcade_event.rs`, `starters.rs`); on the game's side they run on
`menu::flow_game::Game`, which gives them the game's variables and screens. The game-side
pieces are `src/menu/event_flow.rs`, `start_event.rs`, `run_championship.rs` and
`arcade_event.rs`.

An event flow typically goes: choose players and cars, then for each stage show the stage's
details, **run the race**, show the results, visit the service area (`src/service/`: repairs
and set-up between stages), and at the end show the standings. `menu::run_event_races` runs one
round of races, each player in turn.

Running the race means:

1. `race::single_rally_race` (`src/race/single_rally.rs`) copies the menu's choices (stage,
   cars, players, set-up) into the race's settings.
2. `race_from_menu` in `src/race/race_end.rs` frees what the last race left and calls
   `run_race` in the same file.
3. When the race is over, the results are copied back to the menu's side and the flow carries
   on with the results screens.

## A race

`run_race` is the whole life of one stage:

| Step | Function | What happens |
| --- | --- | --- |
| Load | `load_stage` (`src/race/stage_loading.rs`) | Shows the loading picture; `loadlevel_init` (`src/race/loadlevel.rs`) reads the course, objects, textures, map and pace notes step by step; `drive::race_init` places the cars and settles them. |
| Record | `open_replay` (`src/race/replay.rs`) | Starts the replay file (see [Replay files](Replay-files)). |
| Play | `drive::play_race` → `drive::race_loop` | The race itself, until the players finish or leave. |
| Close | `close_replay` | Finishes the replay file. |
| Tidy | `tidy_everything` (`src/race/race_end.rs`) | Frees the stage. |

`drive::race_loop` goes round once per picture drawn. Each time round:

1. **Keys.** `race_keys` (`src/race/frame.rs`): pause, view changes, and Escape, which asks
   whether to leave.
2. **Simulation.** `drive::race_step` looks at how much time has passed and runs that many
   simulation **steps**, `drive::race_tick` each (the cars' part of a step is
   `drive::replay_step`, whether or not a replay is involved). The simulation always advances
   in steps of one hundredth of a second, however fast pictures are drawn. In a step, each car
   gets its inputs (from a player's controls, the computer driver, the network or a replay),
   and `physics::car_physics_step` moves it. The step also runs the stage's animations,
   checkpoints and times, and records the replay.
3. **Picture.** `drive::race_frame` draws one picture: `race::draw_race_scene` draws each
   player's view (course, objects, cars, then effects such as rain and dust from
   `src/effects/`), then the HUD (`src/hud/`), and `race::show_frame` puts it on the screen
   and waits until the next picture is due on the display (`src/platform/pacing.rs`).

The replay viewer (`race::replay_viewer`) loads a stage the same way and runs the same
simulation steps, with the inputs coming from the replay file. More in
[A race, step by step](A-race-step-by-step) and [Car physics](Car-physics).

## How a picture reaches the screen

The game's 3D code (`src/scene/`, `src/race/frame_draw/`) does its own geometry: it turns, lights
and clips every triangle itself and hands finished, screen-space triangles to a thin layer,
`src/d3d/`. That layer passes them as a list of drawing commands to a **backend**
(`src/render/mod.rs`, the `Backend` trait), which draws them:

| Backend | File | Used |
| --- | --- | --- |
| GPU, through SDL's GPU API | `src/render/gpu.rs` | Desktop, by default. |
| OpenGL ES | `src/render/gles.rs` | The handheld. |
| Software rasteriser | `src/render/software.rs`, `src/render/rasteriser.rs` | Fallback, and by choice (`RC2K_RENDERER=software`). |

Menus and text are drawn by the game's own 2D code (`src/gfx/`, `src/text/`) into a picture in
memory, which the platform layer puts on the window. `src/render/choice.rs` decides which backend
starts. More in [Drawing a frame](Drawing-a-frame).

## The platform layer

`src/platform/` is everything between the game and the computer: the window and its events
(`mod.rs`, `events.rs`), the 3D layer's hand-over to the backends (`direct3d.rs`), sound output
(`sound.rs`), controllers (`input.rs`), films (`film.rs`), threads and locks (`sync.rs`),
operating-system facts (`os/`), and bindings to C libraries (`ffi/`). It is built on SDL3. The
game's own code does not call SDL or the operating system directly; if you are porting to a new
system, this is the folder to read (see [Porting to a new platform](Porting-to-a-new-platform)).

## The threads

| Thread | Routine | What it does |
| --- | --- | --- |
| Game (the main thread) | `standalone::run` and everything described above | Start-up, menus, the simulation, and issuing the drawing. |
| Input | `input::input_thread` | Every 20 ms: reads controllers and runs force feedback; counts the key clock. |
| Audio | `audio_thread` (`src/audio/thread.rs`) | Every 20 ms: takes the sound commands the game queued (`src/sound/` is the sending side), feeds the music streams, updates the playing voices. |
| Network | `net::receive_thread` | Waits for messages from the other players and files them for the game thread. During a network game, `src/net/dp_session/` also runs threads for the connections. |
| Shutdown | `threading::shutdown_thread` | Started when the game is asked to quit; waits for the others to end. |
| Drawing helpers | `src/scene/workers.rs` | On a machine with two or more cores, the players' views and the mirror are prepared on separate threads at once, then put together in order. |
| Renderer | `src/render/software.rs`; `src/render/gles.rs`, `src/render/gl_batching.rs` | The software rasteriser splits the picture over the processor's cores. The handheld's OpenGL ES backend has a thread that talks to the GPU and one that prepares its batches. |
| Sound mixer | `src/platform/sound.rs` | SDL calls it to mix the playing sounds into the output. |

The input, audio and network threads are the original game's design, started together by
`threading::start_threads`; they share state with the game thread through the owners, using
locks and atomic values where they meet. The drawing and renderer threads are this project's
additions, and are arranged so that the picture is the same as one thread would draw. More in
[Threads and shared state](Threads-and-shared-state).

## Errors

Two kinds of failure end the program on purpose, and both are worth knowing when you read the
code:

- `FatalError` (`src/errors.rs`): a condition the original game treated as fatal, with its error
  code. `src/fatal/` shows the report.
- `BadInput`: the code met a value it cannot handle sensibly (an index outside a table, a
  damaged file). Rather than carry on with wrong data, the function returns the reason up the
  call chain and the program ends with it in `rc2k.log`.

Most functions return `Result<_, Refused>`, where `Refused` is either of these.

## Where to look for...

| If you want to change or understand | Start in |
| --- | --- |
| A menu screen's layout or items | its `src/menu/..._loop.rs`, and the drawing routine it names in `src/screens/` |
| The order of screens in an event | `rc2k_core/src/menu/rounds.rs`, `championship.rs`; `src/menu/event_flow.rs` |
| Starting a race from the command line | `src/menu/launch.rs`, `rc2k_core/src/menu/launch.rs` |
| How a car drives | `src/physics/` (start at `car_physics_step.rs`) |
| What happens each simulation step | `src/drive/race_tick.rs`, `src/drive/replay_step.rs` |
| Loading a stage | `src/race/loadlevel.rs` |
| What is drawn in a race and in what order | `src/race/draw_race_scene.rs` |
| Weather, dust, windscreen effects | `src/effects/` |
| The in-race display (speed, times, map) | `src/hud/` |
| Sounds and music | `src/sound/` (what the game asks for), `src/audio/` (how it is played) |
| Controllers and key bindings | `src/input/`, `src/platform/input.rs` |
| Options, saved games, best times | `src/saves/` |
| Reading files and paths | `src/files/` |
| The window, the renderers, a new platform | `src/platform/`, `src/render/` |
| The debug window | `src/debug/` |
| Network play | `src/net/` |

Next: [The front end](The-front-end), [A race, step by step](A-race-step-by-step),
[Drawing a frame](Drawing-a-frame), [Threads and shared state](Threads-and-shared-state).
