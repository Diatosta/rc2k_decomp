# Module: drive

*Category: [Module reference](Modules)*

The race while it runs. `race` loads a stage and tidies it away; `drive` is what happens in
between: the race loop, the fixed simulation step, reading the players' controls, the computer
driver, the countdown, the finish, and the recording and playing of replay blocks.

## State

No owner of its own: it works on the race's owner, `Race` (`src/race/owner.rs`). The replay's
working blocks are in `src/drive/replay_frames_store.rs`, the finish's variables in
`src/drive/finish_store.rs`; both are parts of `Race` (`Race::replay`) and are plain fields, not
cells: each sits in a `RefCell` on the game's thread, borrowed for one statement at a time
(`Replay::finish()` / `finish_mut()`, `replay_frames()` / `replay_frames_mut()`). The drawing
parts and the physics step are handed the one value they read (`no_place`, `step_first`).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `play_race` | `src/drive/play_race.rs` | Runs a loaded stage from the countdown to the end. |
| `race_init` | `src/drive/race_init.rs` | Places the cars on the start line and settles them. |
| `race_loop` | `src/drive/race_loop.rs` | Once round per picture: keys, simulation, picture. |
| `race_step` | `src/drive/race_step.rs` | Works out how many simulation steps are due and runs them. |
| `race_tick` | `src/drive/race_tick.rs` | One simulation step (a hundredth of a second). |
| `replay_step` | `src/drive/replay_step.rs` | The cars' part of a step: inputs, physics, recording. |
| `race_frame` | `src/drive/race_frame.rs` | Draws one picture of the race. |
| `read_car_controls` | `src/drive/read_car_controls.rs` | Turns keys and controller axes into a car's inputs. |
| `drive_computer_car` | `src/drive/drive_computer_car.rs` | The computer driver's steering, throttle and brake. |
| `record_inputs`, `play_inputs` | `src/drive/record_inputs.rs`, `src/drive/play_inputs.rs` | Writes a step's inputs into the replay block, or reads them back. |
| `gather`, `scatter` | `src/drive/replay_state.rs` | Copies the cars' state into a replay block and back. |
| `compare_frame` | `src/drive/replay_check.rs` | Compares the state playback reached with the state stored in the next block. |
| `finish_race` | `src/drive/finish_race.rs` | What happens when a car crosses the line. |
| `tyre_change_screen` | `src/drive/tyre_change_screen.rs` | The tyre change played in the middle of some stages. |

## How it is laid out

A flat folder, one function per file. Groups by name: `race_*` (the loop and its steps),
`replay_*` and `*_replay_*` (blocks), `countdown_*` and `start_*` (the start), `finish_*`,
`tyre_*` (the tyre change), `*_inputs` (controls).

## Calls and callers

It calls [`physics`](Module-physics) for each car, [`race`](Module-race) for the stage's data
and drawing, [`scene`](Module-scene), [`sound`](Module-sound), [`input`](Module-input) and
[`net`](Module-net) (in a network race). It is called by `race` (`run_race` and the replay
viewer).

## Related pages

[A race, step by step](A-race-step-by-step), [Replay files](Replay-files),
[Car physics](Car-physics), [Input](Input).
