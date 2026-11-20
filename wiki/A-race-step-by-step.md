# A race, step by step

*Category: [Game flow](Game-flow)*

This page follows one race from the moment a menu asks for it to the moment its results go
back to the menu. It covers loading a stage, the race loop, one simulation step in detail, how
a race ends, and the replay viewer, which reuses nearly all of it.

The code is in `src/race/` (the race as a whole: loading, settings, replay files, the frame),
`src/drive/` (the loop, the step, inputs, the finish) and `src/physics/` (the car). The state
is the `Race` owner (`src/race/owner.rs`).

Two words are used throughout. A **step** is one advance of the simulation: one hundredth of a
second, always. A **block** is one second of replay: 100 steps. (The code often says "tick" or
"tenth" for a step and "frame" for a block; the meaning is the same.)

## Into the race

The event flow (see [The front end](The-front-end)) reaches `race::single_rally_race`
(`src/race/single_rally.rs`). It looks the chosen stage up, copies the menu's choices into the
race's own variables, runs the race, and copies the results back into the players' records.

The copying has its own file, `src/race/settings_copy.rs`: `copy_settings_in` before the race
(stage, cars, gearbox, tyres, set-up, damage carried from the last stage) and
`copy_settings_out` after it (times, damage, wear). The menus and the race keep separate
variables, and these two functions are the only bridge.

`race_from_menu` (`src/race/race_end.rs`) frees what the previous race left and calls
`run_race`, the whole life of one stage:

| Order | Call | What happens |
| --- | --- | --- |
| 1 | `copy_settings_in` | The menu's choices become the race's settings. |
| 2 | `race::race_cars::set_up_arcade_cars`, `race::race_shell::set_start_states` | The field of cars and their start states. |
| 3 | `load_stage` | Everything is loaded (next section). |
| 4 | `open_replay` | The replay file `var/tmprpl` is started. |
| 5 | `drive::play_race` | The race loop runs until the race is over. |
| 6 | `close_replay` | The replay's index is written and the file closed. |
| 7 | `tidy_everything` | The stage is freed. |
| 8 | `copy_settings_out` | Results go back to the menu's side. |

## Loading a stage

`load_stage` (`src/race/stage_loading.rs`) shows the loading picture, wakes the cars' sounds,
and calls `loadlevel_init` (`src/race/loadlevel.rs`), which does the loading proper as a list
of named steps. After each step the loading bar moves on (`src/race/loading_bar_frame.rs`); a frame is drawn
for it only when the last one is at least 33 ms old.
In order, roughly:

| Step | What is loaded | Format page |
| --- | --- | --- |
| Objects and course memory | Room for the scene's objects and the course. | |
| Texture information, stage preferences | Which textures and object kinds this stage uses. | [Object and texture tables](Object-and-texture-tables), [Course files](Course-files) |
| The course | The road and land: faces, records, checkpoints. | [Course files](Course-files), [Packed files (SQ)](Packed-files-SQ) |
| Objects | Scenery and car shapes. | [Object files](Object-files) |
| Textures | The pictures the faces are drawn with. | [Textures (PCX)](Textures-PCX) |
| The map | The in-race map. | [Map files](Map-files) |
| Game fonts | The HUD's fonts. | [Fonts](Fonts) |
| Effects | Weather, spray, windscreen effects, skid marks. | [Effects data](Effects-data) |
| Lights, colours, mud | The stage's light tables, fog and sky colours, the cars' mud. | [Mud files](Mud-files) |
| Checkpoints, overlay | The checkpoint tables the timing and the computer driver use; the HUD's panel. | |
| Pace notes | The co-driver's calls along the stage (`race::init_pace_notes`). | |

Back in `load_stage`, the music is started (`src/race/music.rs`), and then `drive::race_init`
prepares the start: the best line through the checkpoints, the cars placed on the start, their
parts and timers set, and fifty settling steps so that every car sits on its springs before the
player sees it.

With `--seed` on the command line, the random numbers are reseeded just before loading, so
that a race can be repeated exactly.

## The race loop

`drive::play_race` calls `drive::race_loop` (`src/drive/race_loop.rs`), which goes round once
for every picture drawn:

1. **Clock and window.** The clock is read. If the window is not the active one, the race
   pauses.
2. **Input update.** `update_input` (`src/input/keys.rs`) turns the key states into "pressed
   this frame".
3. **Escape**, or a player's pause binding going down (a pad's START sends Escape), asks
   whether to leave (`drive::quit_question`); the answer goes into the cars. A window that is
   not the active one pauses the race until a key (`drive::pause_until_key`).
4. **Race keys.** `race_keys` (`src/race/frame.rs`): view changes and the like.
5. **Simulation.** `drive::race_step` works out how many steps of a hundredth of a second have
   gone by since the last time round and calls `drive::race_tick` that many times. A slow
   frame runs more steps; a fast one may run none. The race therefore runs at the same speed
   whatever the frame rate.
6. **Tyre change.** While a roadside tyre change is under way (after a puncture), the tyre change screen with its timed gauge (`src/drive/tyre_screen.rs`); its prompt shows the controller's button picture ([Controllers](Controllers)).
7. **Picture.** `drive::race_frame` draws and shows one picture (see
   [Drawing a frame](Drawing-a-frame)) and runs the race's sounds for the frame
   (see [Sound and music](Sound-and-music)).

The loop ends when every player is out: finished, retired, or having answered yes to leaving.

## One step

`drive::race_tick` calls `drive::replay_step` (`src/drive/replay_step.rs`). Despite the name it
is the step of every race, recorded or played back.

### Before the cars

- `drive::choose_shown_cars`: which cars the one or two views show.
- `drive::keep_replay_frame`: at the first step of a block, while recording, the cars' whole
  state is copied into the block being built.
- `drive::play_replay_frame`: while playing a replay, at the first step of a block the block
  is read and its state loaded over the cars.

### For each car

The cars are kept in **car blocks**, one per car. A step does not work on a block directly: it
copies the block into one working copy (`drive::load_car`), runs everything on that, and copies
it back (`drive::store_car`). So the physics is written for "the work car" and never needs to
know which car it is.

| Order | Function | What it does |
| --- | --- | --- |
| 1 | `drive::car_input_words` | The car's raw inputs: a player's device readings (`drive::device_axes`), or, in a network game, what the car's owner sent. |
| 2 | `drive::record_inputs_if`, `drive::play_inputs_if` | Recording: the players' inputs are added to the replay block. Playing: they are replaced by the block's. |
| 3 | `drive::load_car` | Block to work car. |
| 4 | `drive::read_car_controls` | Inputs to controls: keys into buttons, lights, view change (not while a replay plays: the viewer's camera stays). For a computer-driven car, `drive::drive_computer_car` supplies the controls instead. |
| 5 | `physics::gear_input_rules`, `drive::steer_inputs`, `drive::pedal_inputs` | Gear requests; steering eased towards the asked angle (keys) or taken from the device; pedals likewise. |
| 6 | `drive::reset_request` | The reset key, held for a second at low speed, puts the car back on the road for a time penalty. |
| 7 | `physics::car_physics_step` | The car moves: see [Car physics](Car-physics). |
| 8 | `net::race_step` | In a network game: send this car's messages, or glide a remote car (see [Network internals](Network-internals)). |
| 9 | `drive::store_car` | Work car back to its block, its clocks one step on. |

A car that is not running (no driver) skips 4 to 8.

### The computer driver

`drive::drive_computer_car` drives from the checkpoints. From the checkpoint the car is at and
its speed, it looks at the bend ahead and picks a speed to aim for, then sets throttle or brake
against it. It steers towards a point on a checkpoint a few checkpoints on. If the car has not
moved for a while, it reverses for a couple of seconds. The same code drives the arcade
mode's rival cars and the car in the attract mode behind the credits.

### Inside the physics step: timing and checkpoints

Three parts of `physics::car_physics_step` matter for the race rather than for the car:

- **Timers** (`physics::race_timers`): the count-down at the start
  (`physics::count_down_start`), the stage clock, and split times as the car passes split
  points (`physics::take_split`). Passing the last one sets the car's finished flag. A car that
  moves off in gear before the start is put back with a penalty (`physics::false_start`).
- **Checkpoints** (`physics::track_checkpoints`): the course is a chain of checkpoints across
  the road. Each step the car's nearest checkpoint is found near its last one
  (`physics::find_car_checkpoint`). That gives its progress, feeds the "wrong way" and
  off-line warnings (`physics::off_line_warnings`), and tells the reset where to put the car.
- **Pace notes** (`physics::pace_notes_step`): looking a distance ahead that grows with speed,
  the co-driver's calls for the checkpoints coming up are queued as speech.

A car stuck for seven seconds is put back on the road by itself
(`physics::reset_when_stuck`).

### After the cars

| Function | What it does |
| --- | --- |
| `drive::replay_check` (`note`) | A consistency check: see "The replay viewer" below. |
| `drive::animate_stage` | Moving scenery and animated lights. |
| `drive::race_mode_cars` | In the arcade mode, starts the rival cars behind the player (`drive::start_cars_behind`). |
| `drive::countdown_beeps` | The start's beeps. |
| `drive::two_car_order` | Who leads, in a two-car race. |
| `drive::flush_replay_if` | At the last step of a block, while recording: the block is packed and written to the replay file. |
| `drive::replay_clock_tick` | The replay's step and block counters move on. |

The replay file's layout, and why it stores state once a second and inputs every step, is on
[Replay files](Replay-files).

## The end of a race

A car is finished when it passes the last split point. The stage time stops; the race goes on
for a moment so the car can roll out, then every player is out and `race_loop` returns.

`close_replay` writes the replay's index. `tidy_everything` stops the race's sounds and frees
the stage's objects, textures and course. `copy_settings_out` hands the times, the damage and
the wear back to the menu's side, and `single_rally_race` puts them in the players' records.
The event flow then shows the results pages.

At the end of a championship there is a short scene of the winner's car
(`menu::championship_finish`, `drive::finish_race`, `drive::finish_run`): a stage is loaded
with one car and a camera flies round it.

## The replay viewer

`menu::replay_viewer` (`src/menu/replay_viewer/mod.rs`) plays the last race's replay or a saved
one. It opens the file (`src/race/open_viewed_replay.rs`), copies the header's settings into
the race's variables, and loads the stage exactly as a race does. Then it runs the same
`replay_step`. The differences are few:

- at the first step of each block the stored state is loaded over the cars, and
- each step the two players' inputs come from the block instead of from the devices.

Everything else, the physics included, runs for real. This is why a replay can be watched from
any camera, and why the viewer can jump to any second.

It is also a test. Just before a block's stored state is loaded, `drive::replay_check`
(`compare_frame`) compares it with the state the physics has just reached by itself. If the
two differ, the physics no longer behaves as it did when the replay was recorded, and the
difference is logged. Every old replay is in this way a check of the physics; see
[Testing](Testing).

The viewer's view item (the box with the car number) chooses which car is shown: car 1, car 2, or
both side by side or one above the other. A replay with two cars or more offers it, a network
race's included (a network replay opens on the car of the machine that saved it); only a
one-player replay with a single car has nothing to choose. Playing a replay while the network
lobby is still open shows the cars the view names, not the lobby machine's own car.

From the viewer a replay can be saved to a slot with a name and with the cameras chosen while
watching (`src/race/save_viewed_replay.rs`).

## Where to look

| To change or understand | Start in |
| --- | --- |
| The order of a race's life | `run_race` in `src/race/race_end.rs` |
| What is loaded, in what order | `src/race/loadlevel.rs`, `src/race/stage_loading.rs` |
| The start: placing and settling cars | `src/drive/race_init.rs` |
| The loop, pausing, quitting | `src/drive/race_loop.rs`, `pause_until_key.rs`, `quit_question.rs` |
| How many steps a frame runs | `src/drive/race_step.rs` |
| The step's order | `src/drive/replay_step.rs` |
| A player's inputs | `src/drive/car_input_words.rs`, `steer_inputs.rs`, `pedal_inputs.rs` |
| The computer driver | `src/drive/drive_computer_car.rs` |
| Timing, splits, the start | `src/physics/race_timers.rs`, `take_split.rs`, `count_down_start.rs`, `false_start.rs` |
| Checkpoints and resets | `src/physics/track_checkpoints.rs`, `reset_car.rs` |
| Recording and playing replays | `src/race/replay.rs`, `src/drive/replay_state.rs`, `src/drive/replay_check.rs` |
| What goes between menu and race | `src/race/settings_copy.rs` |

Next: [Car physics](Car-physics), [Drawing a frame](Drawing-a-frame). See also
[Module-race](Module-race), [Module-drive](Module-drive), [Module-physics](Module-physics).
