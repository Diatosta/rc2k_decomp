# Module: physics

*Category: [Module reference](Modules)*

The car model. For each car and each simulation step it finds the ground under the wheels,
works out the forces (engine, brakes, grip by surface, air drag, contacts with scenery and
other cars), moves the car, changes gear, wears and damages parts, and cues the car's sounds
and particles. It also places the cameras that follow a car, and steps the pace notes.

The arithmetic follows the original exactly, including the way its floating-point numbers were
rounded, because replays only play back correctly if every step gives the same result.

## State

No owner of its own; its state is the `physics` group of the race's owner (`Race::physics`,
`src/race/owner_physics.rs`). A car's record is `Car` in `src/physics/car_store.rs`; the
surface and force tables are `PhysicsTables` in `src/physics/tables_store.rs`; other parts are
the `*_store.rs` files (camera, contacts, gears, lit faces, ground search).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `car_physics_step` | `src/physics/car_physics_step.rs` | One car, one step: the place to start reading. |
| `find_ground` | `src/physics/find_ground.rs` | Finds the piece of course under a point. |
| `drive_wheels_step` | `src/physics/drive_wheels_step.rs` | The driven wheels: torque, slip, grip. |
| `engine_torque` | `src/physics/engine_torque.rs` | The engine's torque at the current revs. |
| `gearbox_step` | `src/physics/gearbox_step.rs` | Gear changes, manual and automatic. |
| `apply_forces` | `src/physics/apply_forces.rs` | Sums the forces into the car's motion. |
| `move_car` | `src/physics/move_car.rs` | Moves and turns the car by its motion. |
| `car_collisions` | `src/physics/car_collisions.rs` | Cars hitting each other. |
| `damage_step` | `src/physics/damage_step.rs` | Wear and damage. |
| `load_car_specs` | `src/physics/load_car_specs.rs` | Copies a car model's figures into its record. |
| `place_camera` | `src/physics/place_camera.rs` | Puts the view's camera behind, in or above the car. |
| `pace_notes_step` | `src/physics/pace_notes_step.rs` | Decides when the co-driver's next call is due. |
| `Car` | `src/physics/car_store.rs` | A car's record, field by field. |

## How it is laid out

Flat, one function per file (about 230). Groups by name: `*_contact*`, `face_*`, `ground*`
(the course under and around the car), `gear*`, `engine_*`, `free_rev*` (drive train),
`camera_*` and `ease_view_*` (cameras), `damage_*`, `cue_*` and `place_sound_*` (sounds),
`count_*` (timers). `damage_model/` holds reference results for its tests.

## Calls and callers

It uses `math` heavily, the course data of [`race`](Module-race), [`scene`](Module-scene)'s
matrices, and calls [`sound`](Module-sound) and [`effects`](Module-effects). It is called by
[`drive`](Module-drive) every step.

## Related pages

[Car physics](Car-physics), [A race, step by step](A-race-step-by-step),
[Course files](Course-files), [Replay files](Replay-files).
