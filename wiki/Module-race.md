# Module: race

*Category: [Module reference](Modules)*

A stage, from loading to tidying up, and nearly all the state of a race. It reads the course
and everything on it, opens and closes the replay file, draws the race's views, and holds the
replay viewer. The running of the race itself (the loop and the simulation step) is in
[`drive`](Module-drive); the car model is in [`physics`](Module-physics).

## State

The owner is `Race` (`src/race/owner.rs`), the largest by far. It is divided into groups, each
in its own file: `owner_physics.rs`, `owner_course.rs`, `owner_frame.rs`, `owner_replay.rs`,
`owner_views.rs`, `owner_shared.rs` (what other threads also read). The HUD's and the effects'
state are fields of it. The groups' parts are the many `*_store.rs` files; the cars' records
are in `blocks_store.rs`.

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `single_rally_race` | `src/race/single_rally.rs` | Copies the menu's choices into the race's settings, runs the race, copies the results back (for two players, or three and four on a split screen, each player's side: `state_store::RaceState::car(k)`, `result(k)`, ...). |
| `race_from_menu`, `run_race` | `src/race/race_end.rs` | One stage's whole life: load, record, play, close, tidy. |
| `load_stage` | `src/race/stage_loading.rs` | Shows the loading picture and loads the stage. |
| `loadlevel_init` | `src/race/loadlevel.rs` | The loading steps in order: course, objects, textures, sounds, pace notes. |
| `load_level` | `src/race/course_data.rs` | Reads the course file. |
| `open_replay`, `close_replay` | `src/race/replay.rs` | Starts and finishes the replay file. |
| `viewer_list_store`, `render_row`, `viewer_tip` | `viewer_list_store.rs`, `render_row.rs`, `viewer_tip.rs` | The replay viewer's state: its list of buttons, the render row and the name shown under it. The viewer itself (keys, frame, render dialog) is `menu::replay_viewer`. |
| `open_viewed_replay`, `save_viewed_replay` | files of the same names | Opens a saved replay for the viewer; saves one into a slot. |
| `draw_race_scene` | `src/race/draw_race_scene.rs` | Draws every player's view, then the overlays. |
| `blend` | `src/race/blend.rs` | The frames between two ticks: each car's moving values after every tick, and the blend of the last two put in the blocks while a frame is drawn (the advanced page's "smooth frames"). |
| `show_frame` | `src/race/show_frame.rs` | Puts the finished picture on the screen. |
| `race_keys` | `src/race/frame.rs` | The race's own keys: views and the like. |
| `draw_car` | `src/race/car_draw.rs` | Draws a car from its parts. |
| `draw_skidmarks` | `src/race/skidmarks.rs` | The skid marks on the road. |
| `load_mud`, `init_mud` | `src/race/mud.rs` | Mud that gathers on a car's textures. |
| `tidy_everything` | `src/race/race_end.rs` | Frees the stage. |

## How it is laid out

Flat, about 190 files. Groups by name: `*_store.rs` (state), `owner_*.rs` (the owner's
groups), `replay*.rs` and `viewer*.rs` (replays and the viewer), `course_*.rs`, `level_*.rs`
and `stage_*.rs` (loading), `draw_*.rs`, `frame*.rs` and `view_*.rs` (drawing), `mud*.rs`,
`car_*.rs`. `frame_draw/` holds the pieces of a view's drawing (`course_view.rs`,
`scene_objects.rs`, `lights.rs`, `environment.rs`, `panel.rs`).

## Calls and callers

It uses [`scene`](Module-scene) for all 3D, [`gfx`](Module-gfx), [`files`](Module-files),
[`hud`](Module-hud), [`effects`](Module-effects), [`sound`](Module-sound) and calls
[`drive`](Module-drive) to run the race. It is called by [`menu`](Module-menu)'s event flows.
`physics`, `scene`, `drive`, `effects`, `net` and `hud` all read its state.

## Related pages

[A race, step by step](A-race-step-by-step), [Drawing a frame](Drawing-a-frame),
[Replay files](Replay-files), [Course files](Course-files), [Map files](Map-files),
[Mud files](Mud-files), [Music](Music).
