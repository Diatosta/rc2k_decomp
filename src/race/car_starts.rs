//! Race initialisation's first step, the cars' starts (`start_cars`): each car of the race is
//! loaded into the work copy (`car`), set at its start (`start_car`: its checkpoint, its place on
//! the line and to the side, the nearest checkpoint, then `reset_car_motion`: settled on the ground
//! by `settle_car`, its matrix undone, its motion cleared), two blocks cleared, and stored back.
//! Uses `physics::set_handling` and `physics::probe_ground`.

use crate::errors::{BadInput, Refused};
use crate::memory::session::DwordLike;
use crate::memory::session::PlainDword;
use crate::physics::contact_store::ContactWork;
use crate::physics::step_families::{GroundInputs, ResetInputs};
use crate::race::best_line::MOST_CHECKS;
use crate::race::matrices_store::SceneMatrices;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;

/// The most cars `car_blocks` has.
pub const MOST_CARS: u32 = 8;

fn add(cell: &impl DwordLike, by: u32) {
    cell.set(cell.get().wrapping_add(by));
}

fn zero(cells: &[&PlainDword]) {
    for cell in cells {
        cell.set(0);
    }
}

fn wheels(contact: &ContactWork) -> [u32; 4] {
    crate::memory::session::read_first(&contact.probe_height)
}

/// One bisection step of `settle_car`: `angle` moved by `step` towards the higher pair (down when
/// `low` - `high` is negative), the step halved.
fn settle_step(low: u32, high: u32, angle: &impl DwordLike, step: &impl DwordLike) {
    let s = step.get();
    if (low.wrapping_sub(high) as i32) < 0 {
        angle.set(angle.get().wrapping_sub(s));
    } else {
        add(angle, s);
    }
    step.set(s >> 1);
}

/// The matrix (`get_mat_crash`) and the wheels (`physics::probe_ground`).
fn matrix_and_wheels(
    engine_tables: &crate::math::engine_tables::EngineTables,
    ground: &GroundInputs<'_>,
    matrices: &SceneMatrices,
    scene: &crate::scene::owner::Scene,
) -> Result<[u32; 4], Refused> {
    crate::race::stage_prefs::get_mat_crash(engine_tables, matrices, ground.car, scene);
    crate::physics::probe_ground::probe_ground(ground, ground.car, scene)?;
    Ok(wheels(ground.contact))
}

/// Settles the work car on the ground: `physics::set_handling`; the work car's pitch and roll
/// (+0x9C4, +0x9C8) 0, both steps 0x20000000, `settle_mark` 0x80; eight times the matrix
/// (`get_mat_crash`) and the wheels (`physics::probe_ground`), the roll moved by the step towards
/// wheels 0 + 2 against 1 + 3, the pitch towards 0 + 1 against 2 + 3, each step halved; once more
/// the matrix and the wheels, then the car's y and the four wheels and the four after them are
/// lowered by the lowest wheel.
pub fn settle_car(c: &ResetInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let physics = c.physics;
    let shared = c.shared;
    crate::physics::set_handling::set_handling(c.physics.damage(), c.numbers, c.shared)?;
    shared.car().car_pitch.set(0);
    shared.car().car_roll.set(0);
    shared.physics_tables.get().settle_steps[0].set(0x2000_0000u32);
    shared.physics_tables.get().settle_steps[1].set(0x2000_0000u32);
    physics.ground_probe.get().settle_mark.set(0x80);
    for _ in 0..8 {
        let w = matrix_and_wheels(
            &shared.buffers.reads.get().engine_tables,
            &c.ground,
            c.matrices,
            scene,
        )?;
        settle_step(
            w[0].wrapping_add(w[2]),
            w[1].wrapping_add(w[3]),
            &shared.car().car_roll,
            &shared.physics_tables.get().settle_steps[1],
        );
        settle_step(
            w[0].wrapping_add(w[1]),
            w[2].wrapping_add(w[3]),
            &shared.car().car_pitch,
            &shared.physics_tables.get().settle_steps[0],
        );
    }
    let w = matrix_and_wheels(
        &shared.buffers.reads.get().engine_tables,
        &c.ground,
        c.matrices,
        scene,
    )?;
    let mut lowest = 0x64_0000u32;
    for &h in &w {
        if (h.wrapping_sub(lowest) as i32) < 0 {
            lowest = h;
        }
    }
    shared.car().position[1].set(shared.car().position[1].get().wrapping_sub(lowest));
    // `probe_height` .. 20: the four wheels' heights (made by `physics::probe_ground`).
    for (k, h) in w.iter().enumerate() {
        physics.contact.get().probe_height[k].set(h.wrapping_sub(lowest));
    }
    // `probe_y` .. 20: four more lowered with them.
    for cell in &physics.contact.get().probe_y[..4] {
        cell.set(cell.get().wrapping_sub(lowest));
    }
    Ok(())
}

/// Settles the car (`settle_car`), undoes its matrix (`invert_car_matrix`) and clears its motion:
/// the work car's +0x198 .. +0x1A0 0, +0x1A4 1.0, +0x1A8 0, `speed` (+0x1AC) 0, +0x184 ..
/// +0x18C 0; heading, pitch and roll copied to `start_heading` / `58` / `54` and +0x6B0 / +0x6B8 /
/// +0x6B4; +0x1E8, `drive_speed_a`, `drive_speed_b`, `gear`, +0x200, +0x204, +0x1C4,
/// +0x1F4 0; with +0x68C not 0, +0x688 and +0x684 get it less one and +0x68C 0.
pub fn reset_car_motion(
    c: &ResetInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let shared = c.shared;
    let physics = c.physics;
    settle_car(c, scene)?;
    crate::race::start_place::invert_car_matrix(c.course, c.car, scene);
    for v in &shared.car().velocity {
        v.set(0.0);
    }
    shared.car().local_velocity[0].set(1.0);
    shared.car().local_velocity[1].set(0.0);
    shared.car().local_velocity[2].set(0.0);
    for v in &shared.car().spin {
        v.set(0.0);
    }
    let heading = shared.car().car_heading.get();
    physics.camera().start_heading.set(heading);
    shared.car().camera_heading.set(heading);
    let pitch = shared.car().car_pitch.get();
    physics.camera().start_pitch.set(pitch);
    shared.car().camera_pitch.set(pitch);
    let roll = shared.car().car_roll.get();
    physics.camera().start_roll.set(roll);
    shared.car().camera_roll_turn.set(roll);
    shared.car().stuck_count.set(0);
    shared.car().drive_speed_a.set(0.0);
    shared.car().drive_speed_b.set(0.0);
    shared.car().gear.set(0);
    shared.car().eased_yaw_spin.set(0.0);
    shared.car().grip_force_sum.set(0.0);
    shared.car().speed.set(0.0);
    shared.car().wall_hit_timer.set(0);
    let pending = shared.car().camera_view_return.get();
    if pending != 0 {
        shared.car().camera_blend_view.set(pending - 1);
        shared.car().camera_view.set(pending - 1);
        shared.car().camera_view_return.set(0);
    }
    Ok(())
}

/// Rotates (x, 0, z) by the start place's heading with `math::rotate_point_about_y`: x and z are
/// moved. The y it works on is a leftover value; it lands only in `scratch_y`, which every reader
/// writes first.
fn move_turned(course: &Course, shared: &RaceShared, x: u32, z: u32) {
    let [x, _, z, _] = crate::math::rotate_point_about_y::rotate_point_about_y(
        &shared.buffers.reads.get().engine_tables,
        x as i32,
        0,
        z as i32,
        course.course.get().place_heading.get(),
    );
    add(&shared.car().position[0], x);
    add(&shared.car().position[2], z);
}

/// Sets the work car's start: +0x478, +0x694 0, +0xAD4/+0xAD8 0x80000000, +0x1E4, +0x1D8, +0x1D4 0,
/// +0x644 .. +0x650 and +0x638 .. +0x640 0. Unless `run_kind` is 4: its checkpoint is
/// `first_check_checkpoint` (less the car's number / 2 but with `run_kind` 1 or 3) into
/// `line_place`, `line_fraction` 0 (both 0 with `Finish.no_place`); `place_car_on_line`; without
/// `Finish.no_place` the car is moved back (0, -0x20000 turned by `place_heading`) and, with
/// `session` or (`run_kind` 0 and `players` not 1), to the side (+-0x14000 by the car's number's
/// low bit). Then `nearest_checkpoint` goes into +0x98, +0x94, +0x9C; its check count less one held
/// to 0 .. 100 into +0x90 and +0xAC; `reset_car_motion`; +0x264 .. +0x26C 0. Refused: a nearest
/// checkpoint past the lists (when met).
pub fn start_car(
    c: &ResetInputs<'_>,
    scene: &crate::scene::owner::Scene,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<(), Refused> {
    let lights = &scene.lights;
    let course = &c.course;
    c.car.notice_count.set(0);
    c.car.camera_switch_active.set(0);
    c.car.map_corner[0].set(0x8000_0000);
    c.car.map_corner[1].set(0x8000_0000);
    c.car.engine_sound_revs.set(0);
    c.car.engine_speed.set(0.0);
    c.car.gear.set(0);
    zero(&[
        &c.car.steering,
        &c.car.steering_raw,
        &c.car.steer_angle,
        &c.car.steer_angle_visual,
    ]);
    zero(&[&c.car.throttle, &c.car.brake, &c.car.handbrake]);
    if c.vars.run_kind.get() != 4 || crate::debug::screens::free_play_placed() {
        // The start's pure parts are in `rc2k_core::race::start`.
        use rc2k_core::race::start::{side_offset, start_checkpoint, starts_aside};
        let first = crate::race::race_init::first_check_checkpoint(c.course, c.shared)?;
        let mode: u32 = c.vars.run_kind.get();
        let place = start_checkpoint(first, mode, c.work.current_car.get());
        lights.state.get().line_place.set(place);
        lights.state.get().line_fraction.set(0);
        if c.finish.no_place.get() != 0 {
            lights.state.get().line_place.set(0);
            lights.state.get().line_fraction.set(0);
        }
        crate::race::start_place::place_car_on_line(c.shared, course, scene)?;
        if c.finish.no_place.get() == 0 {
            move_turned(c.course, c.shared, 0, 0xFFFE_0000);
            let side = starts_aside(
                u32::from(net_race.is_on()),
                || c.vars.run_kind.get(),
                || c.view.players.get(),
            );
            if side {
                move_turned(c.course, c.shared, side_offset(c.work.current_car.get()), 0);
            }
        }
    }
    let nearest =
        crate::race::start_place::nearest_checkpoint(c.physics, c.course, c.shared, scene)?;
    c.car.checkpoint.set(nearest);
    c.car.reset_checkpoint.set(nearest);
    c.car.last_pace_call.set(nearest);
    if nearest >= MOST_CHECKS {
        return Err(BadInput("a nearest checkpoint past the lists").into());
    }
    let checks = rc2k_core::race::start::start_checks(
        crate::race::checkpoints_store::check_count(
            &c.shared.buffers.reads.get().checkpoints,
            nearest as usize,
        )
        .get(),
    );
    c.car.splits_passed.set(checks);
    c.car.splits_recorded.set(checks);
    reset_car_motion(c, scene)?;
    zero(&[
        &c.car.reset_key_count,
        &c.car.stopped_steps,
        &c.car.tyre_change_state,
    ]);
    Ok(())
}

/// For each car of the race (`race_cars`): `current_car`, `drive::load_car`, `start_car`, the work
/// car's +0xA3C (8 dwords) and +0xA5C (0x11) cleared, `drive::store_car`. Refused (before
/// anything): no cars or more than 8.
pub fn start_cars(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<(), Refused> {
    let shared = &race.shared;
    let cars: u32 = shared.view.get().race_cars.get();
    if cars == 0 || cars > MOST_CARS {
        return Err(BadInput("no cars or more than eight").into());
    }
    for n in 0..cars {
        shared.work.get().current_car.set(n);
        crate::drive::load_car::load_car(shared)?;
        start_car(
            &crate::physics::step_inputs::StepInputs::of(race).reset(),
            scene,
            net_race,
        )?;
        // Clears +0xA3C (8 dwords) and +0xA5C (0x11).
        for cell in shared
            .car()
            .part_damage_limits
            .iter()
            .chain(&shared.car().saved_point_figures[..18])
        {
            cell.set(0);
        }
        crate::drive::store_car::store_car(shared)?;
    }
    Ok(())
}
