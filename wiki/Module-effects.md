# Module: effects

*Category: [Module reference](Modules)*

The race's visual effects: rain and snow, the spray, dust, grit and smoke the wheels throw up,
drops and flakes on the windscreen, the lens flare, glows and the cars' shadows. It also holds
a small hidden game of Pong played on the windscreen.

## State

`Effects` (`src/effects/owner.rs`), a field of the race's owner (`Race::effects`). Its parts
are grouped in `src/effects/owner_groups.rs` and the `*_store.rs` files (masks, particle rings,
surfaces, windscreen drops and flakes).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `init_marks` | `src/effects/init_marks.rs` | Sets the effects up while a stage loads (reads the files in `files/m`). |
| `run_frame` | `src/effects/run_frame.rs` | Runs the effects for one picture. |
| `draw_frame` | `src/effects/draw_frame.rs` | Draws them into a view. |
| `tidy_marks` | `src/effects/tidy_marks.rs` | Frees them when the stage ends. |
| `run_rain`, `run_snow` | `src/effects/run_rain.rs`, `src/effects/run_snow.rs` | The weather. |
| `emit_car_particles` | `src/effects/emit_car_particles.rs` | Decides what each car throws up this step. |
| `run_particles` | `src/effects/run_particles.rs` | Moves and draws the particles. |
| `windscreen_drops_frame` | `src/effects/windscreen_drops_frame.rs` | Drops running on the windscreen. |
| `draw_lens_flare` | `src/effects/draw_lens_flare.rs` | The sun's flare. |
| `draw_shadow` | `src/effects/draw_shadow.rs` | A car's shadow on the ground. |
| `load_mask_pictures` | `src/effects/load_mask_pictures.rs` | Loads the mask pictures the effects are cut from. |
| `run_pong` | `src/effects/run_pong.rs` | The hidden Pong. |

## How it is laid out

A flat folder, one function per file. Prefixes: `emit_*` (what makes particles), `run_*` (a
kind of effect's step), `draw_*` (drawing), `windscreen_drops*` and `windscreen_flakes*`,
`pong*`. Words in the code: a "mark" is the game's own name for any effect; a "ring" is a
fixed-size list that overwrites its oldest entry.

## Calls and callers

It draws through [`d3d`](Module-d3d) and uses [`scene`](Module-scene) for projection and
textures and `math` throughout. [`physics`](Module-physics) calls the emitters;
[`race`](Module-race) calls the set-up and the per-frame functions.

## Related pages

[Drawing a frame](Drawing-a-frame), [Effects data](Effects-data), [Mud files](Mud-files).
