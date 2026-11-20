# Car physics

*Category: [Game flow](Game-flow)*

This page describes the car model as the code has it: what one physics step does, where the
forces come from, how the surface changes the grip, how the engine and gearbox work, what
damage and wear do, and how the set-up values and the tuning switches reach the model. It names
the files to read; it does not reproduce the game's tables of figures.

The code is `src/physics/`, with the pure arithmetic in `rc2k_core/src/physics/`. One file
holds one part of the model, and its top comment lists what it reads and writes.

A caution for readers: this is a port of a 1999 model whose source was not available. The
*behaviour* is exact, and many names are certain, but some field names are best guesses from
how a value is used. Where a file's comment says a meaning "is a reading", take it as that.

## The work car

The simulation keeps each car in a **car block** (`Car`, `src/physics/car_store.rs`; the blocks
are in `src/race/blocks_store.rs`). A physics step does not run on a block. The step copies
the current car's block into one working copy, the **work car**, runs on that, and copies it
back (`drive::load_car`, `drive::store_car`). The physics' own constants and scratch values
are in `PhysicsTables` (`src/physics/tables_store.rs`).

The first part of a car block, about 170 fields, is everything needed to carry the car on from
one moment to the next. That part is what a replay stores once a second
(see [Replay files](Replay-files)).

Positions are whole numbers in 1/65536ths of a unit; speeds, forces and matrices are floats.
The arithmetic follows the original's floating-point precision and rounding exactly
(`src/math/`), because a replay only plays back correctly if every step gives the same bits.

## One step, in order

`physics::car_physics_step` (`src/physics/car_physics_step.rs`) runs once per car per step
(a hundredth of a second). Its top comment lists every call; in outline:

| Phase | What happens | Files |
| --- | --- | --- |
| Set-up | The car model's figures are read (`load_car_specs`), gear ratios made (`make_gear_ratios`), the set-up values turned into handling figures (`set_handling`), the race timers run. | `load_car_specs.rs`, `gearing.rs`, `set_handling.rs`, `race_timers.rs` |
| Ground | The ground under the car's probe points is found, and the surface under each wheel. | `ground.rs`, `probe_ground.rs`, `wheel_surfaces.rs` |
| Forces | Forces cleared; wheel weights, downforce, gearbox, engine drive, grip, air drag, springs, the body against the ground. | see below |
| Motion | The forces and torques are summed and applied; the car moves and turns. | `apply_forces.rs`, `apply_torques.rs`, `move_car.rs`, `spin_matrix.rs` |
| Contacts | Up to a fixed number of rounds: ground, edges, walls, other cars, scenery. A round that had to push the car back redoes the motion. | `car_collisions.rs`, `wall_contacts.rs`, `face_contacts.rs`, `contact_force.rs` |
| After | Stuck reset, engine sound, wheel spin, checkpoints, pace notes, count-down, damage and wear, parts falling off, camera, lights. | `reset_when_stuck.rs`, `track_checkpoints.rs`, `damage_step.rs`, `drop_broken_parts.rs`, `camera_frame.rs` |

Forces are gathered as **force records**: a force and the point on the car where it acts. Every
part of the model adds its records; `sum_forces.rs` and the `apply_*` files turn them into the
car's change of speed and of spin.

## The ground

The course is a grid of cells holding faces (triangles and quads of road and land; see
[Course files](Course-files)). `physics::find_ground` (`ground.rs`, `ground_search.rs`) finds
the face under a point: it tries the face found last time, then the cells around the point. It
returns the height there and where on the face the point lies.

The car has a short list of probe points, the wheels among them. Each step, `probe_ground.rs`
finds the ground under each, which gives each wheel's height above or below the road.

## Springs and weight

`wheel_springs.rs`: each wheel's spring is squeezed by how far the wheel is pushed into the
ground. The spring pushes back in proportion, and a damping term opposes the change in squeeze
since the last step. Stiffness and damping come from the set-up (below).

`set_wheel_weights.rs` shares the car's weight over the wheels; `set_downforce.rs` adds a
downward force that grows with the square of the speed. `air_drag.rs` opposes the motion,
again with the square of the speed.

## Grip by surface

`wheel_surfaces.rs` works out what is under each wheel. A face names a texture, and the
texture's record (see [Object and texture tables](Object-and-texture-tables)) says what kind of
surface each part of that texture is. So one textured face can be tarmac in the middle and
grass at the edge.

Each surface kind has its own grip and drag figures. The grip also depends on the tyres fitted
(six kinds, chosen in the service area) and on their wear. The broad kinds the tuning switches
know are tarmac, gravel, mud, snow and grass.

`drive_grip_forces.rs` then computes the sideways force at the wheels from the steering angle
and the speed, and limits it to what the surface, the load on the wheels and the kind of drive
allow. Above the limit the car slides. Skid marks (`start_skid_mark.rs`), surface noise
(`surface_noise.rs`) and thrown-up mud (`accumulate_mud.rs`) follow from the same figures.

## Engine and gearbox

| Part | File | In short |
| --- | --- | --- |
| Engine torque | `engine_torque.rs` | A torque curve of nine points over the engine's speed range, read from the car's record; the torque is the line between the two points either side. Scaled by an engine power factor that damage lowers. |
| Drive | `drive_force.rs` | Torque times throttle, through the gear's ratio, becomes the drive at the wheels. With the throttle closed the engine brakes. |
| Driven wheels | `drive_wheels_step.rs` | The driven wheels' speed follows the drive less their drag, and is pulled towards the car's speed; the difference is wheel slip. Front, rear and four-wheel drive share the torque differently (`drive_torque_share.rs`). |
| Revs | `revs_from_wheels.rs`, `wheels_from_revs.rs`, `free_rev.rs` | In gear, the engine's speed follows the wheels; in neutral or on the clutch it revs freely. It is held between idle and the rev limit; below a stall speed the engine stalls (`stall_when_still.rs`). |
| Gearbox | `gearbox_step.rs`, `manual_gearbox.rs`, `auto_gearbox.rs`, `change_gear.rs` | Three kinds: manual, semi-automatic and automatic. The automatic box changes up and down at shares of the engine's limit from the car's record. A change takes a moment during which there is no drive. |
| Gear ratios | `make_gear_ratios.rs` | The ratios are built from the car's record and the "gear ratio" set-up value, which trades top speed for acceleration. |

A car has three to seven forward gears and reverse. `gear_input_rules.rs` decides which gear
requests count at a given moment (none during the start's hold or a tyre change, and the
automatic box reads the two gear inputs as forward and reverse).

## Contacts and crashes

After the car has moved, the contact rounds check it against the world:

- the body's lowest point against the ground (`crash_contact.rs`);
- the edges of the road it has crossed (`crossed_edge_contact.rs`) and walls
  (`wall_contacts.rs`, `wall_push_back.rs`);
- the other cars, corner against box (`car_collisions.rs`, `corner_hits_car.rs`);
- scenery objects (`object_hits_car.rs`) and faces around the car (`face_contacts_around.rs`).

A contact produces a force at the contact point (`contact_force.rs`), which both stops the car
going through and spins it. If a round finds the car inside something, the saved state is
restored and the motion redone with the new forces. If the rounds run out, the car's motion is
limited and a clear place is looked for (`limit_motion.rs`, `find_clear_place.rs`).

How hard and where the car was hit is kept for the damage model and for the sounds.

## Damage and wear

`damage_step.rs` runs the model in three parts every step (`damage_model.rs` describes them):

1. `damage_factors.rs` gathers this step's facts: engine speed against its limit, car speed,
   brake use, the surface, the hit and its side, the load on each wheel.
2. `damage_wear.rs` moves the **gauges**. Each gauge is a figure from zero to a top value.
   There are gauges for brake heat and wear, engine heat, damper wear, tyre wear (one per
   wheel), gear wear (one per gear), and damage to the engine, exhaust, turbo, cooling, clutch,
   differential, gearbox, body, electrics, lights, steering and suspension (per wheel). Heat
   gauges rise with use and fall again; wear and damage only rise.
3. `damage_gauges.rs` sums them into the figures the service area and the HUD show.

`rate_damage.rs` turns gauges into consequences: less engine power, weaker brakes, a failed
handbrake, lost gears, a gearbox stuck in neutral, lights out, a car that will not run at all.
Some failures happen by chance, more likely the worse the gauge. `drop_broken_parts.rs` makes
badly damaged body parts fall off (they then bounce along as loose parts, `move_slots.rs`).
`damage_notices.rs` puts the warnings on the screen.

Between stages the service area repairs chosen parts by setting their gauges back
(see [The front end](The-front-end)). One of the game's cheats is "No Damage".

## The set-up values

The service area's six values reach the model through `set_handling.rs` (and
`make_gear_ratios.rs` for the gears):

| Set-up value | Effect in the model |
| --- | --- |
| Tyres | Which grip figures each surface gives; how fast the tyres wear. |
| Gear ratio | Scales the gear ratios: shorter for acceleration, longer for top speed. |
| Ride height | The springs' rest length, and with it the body's clearance and how the car leans. |
| Suspension stiffness | Spring stiffness and damping. |
| Brake balance | The share of braking between front and rear wheels. |
| Steering sensitivity | How far and how fast the wheels turn for a given input. |

Each value is a whole number on a scale; `set_handling.rs` places it between two constants to
get the handling figure the forces use. Suspension damage shifts a wheel's stiffness and ride
height away from what was set.

## The tuning switches

The game has a set of physics switches read from `var/ral.cfg` (see
[Settings files](Settings-files)): a master switch, gravity, grip for tarmac, gravel, mud, snow
and grass, brake grip, and a few more, each a percentage. `physics::apply_patches`
(`apply_patches.rs`, arithmetic in `rc2k_core/src/physics/patches.rs`) holds each to its
allowed range and works it into the constants once, when the game's data is loaded.

A replay stores the switches it was recorded with, and the viewer puts them in force while it
plays, so a replay still plays back correctly after the player has changed them.

## Trying the physics

- `acceleration_figures.rs` and `acceleration_run.rs` are the game's own "dyno": a car run through its
  gears on a flat road to measure its figures. The debug window's Dyno tab shows the result
  (see [The debug window](The-debug-window)).
- The unit tests in `rc2k_core/src/physics/` check each formula.
- Every replay is a whole-model test: playing it back compares the model's state with the
  recorded state once a second (`src/drive/replay_check.rs`; see [Testing](Testing)).

## Where to look

| To change or understand | Start in |
| --- | --- |
| The order of a step | `src/physics/car_physics_step.rs` |
| A car's fields | `src/physics/car_store.rs` |
| The model's constants | `src/physics/tables_store.rs`, `src/race/car_specs_store.rs` |
| Grip and sliding | `wheel_surfaces.rs`, `drive_grip_forces.rs` |
| Engine and gears | `engine_torque.rs`, `drive_force.rs`, `gearbox_step.rs` |
| Suspension | `wheel_springs.rs`, `set_handling.rs` |
| Collisions | `car_collisions.rs`, `wall_contacts.rs`, `contact_force.rs` |
| Damage | `damage_model.rs`, `rate_damage.rs` |
| The camera behind the car | `camera_frame.rs`, `place_camera.rs` |
| Exact float behaviour | `src/math/x87_float.rs` |

Next: [Drawing a frame](Drawing-a-frame). See also [Module-physics](Module-physics),
[Module-drive](Module-drive), [Built-in tables](Built-in-tables).
