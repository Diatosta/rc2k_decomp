//! Race initialisation's first step, the cars' starts (`start_cars`): each car of the race is
//! loaded into the work copy (`car`), set at its start (`start_car`: its checkpoint, its place on
//! the line and to the side, the nearest checkpoint, then `reset_car_motion`: settled on the ground
//! by `settle_car`, its matrix undone, its motion cleared), two blocks cleared, and stored back.
//! Uses `physics::set_handling` and `physics::probe_ground`.

use crate::errors::{BadInput, Refused};
use crate::memory::session::DrawPlain;
use crate::memory::session::DwordLike;
use crate::memory::session::PlainDword;
use crate::physics::contact_store::ProbeLists;
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

fn wheels(probes: &ProbeLists) -> [u32; 4] {
    [
        probes.probe_height[0],
        probes.probe_height[1],
        probes.probe_height[2],
        probes.probe_height[3],
    ]
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
    engine_tables: crate::math::engine_tables::EngineTablesRef,
    ground: &GroundInputs<'_>,
    matrices: &DrawPlain<SceneMatrices>,
    scene: &crate::scene::owner::Scene,
) -> Result<[u32; 4], Refused> {
    crate::race::stage_prefs::get_mat_crash(engine_tables, matrices, ground.car, scene);
    crate::physics::probe_ground::probe_ground(ground, ground.car)?;
    Ok(wheels(&ground.probes.borrow()))
}

/// Settles the work car on the ground: `physics::set_handling`; the work car's pitch and roll
/// (`car_pitch`, `car_roll`) 0, both steps 0x20000000, `settle_mark` 0x80; eight times the matrix
/// (`get_mat_crash`) and the wheels (`physics::probe_ground`), the roll moved by the step towards
/// wheels 0 + 2 against 1 + 3, the pitch towards 0 + 1 against 2 + 3, each step halved; once more
/// the matrix and the wheels, then the car's y and the four wheels and the four after them are
/// lowered by the lowest wheel.
pub fn settle_car(c: &ResetInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let physics = c.physics;
    let shared = c.shared;
    crate::physics::set_handling::set_handling(&mut c.physics.damage_mut(), c.numbers, c.shared)?;
    shared.car().car_pitch.set(0);
    shared.car().car_roll.set(0);
    shared.physics_tables.get().settle_steps[0].set(0x2000_0000u32);
    shared.physics_tables.get().settle_steps[1].set(0x2000_0000u32);
    physics.ground_probe().borrow_mut().settle_mark = 0x80;
    for _ in 0..8 {
        let w = matrix_and_wheels(shared.buffers.engine_tables(), &c.ground, c.matrices, scene)?;
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
    let w = matrix_and_wheels(shared.buffers.engine_tables(), &c.ground, c.matrices, scene)?;
    let mut lowest = 0x64_0000u32;
    for &h in &w {
        if (h.wrapping_sub(lowest) as i32) < 0 {
            lowest = h;
        }
    }
    shared
        .car()
        .position
        .set_bits_at(1, shared.car().position.bits_at(1).wrapping_sub(lowest));
    // `probe_height` .. 20: the four wheels' heights (made by `physics::probe_ground`).
    let mut probes = physics.probes().borrow_mut();
    for (k, h) in w.iter().enumerate() {
        probes.probe_height[k] = h.wrapping_sub(lowest);
    }
    // `probe_y` .. 20: four more lowered with them.
    for y in &mut probes.probe_y[..4] {
        *y = y.wrapping_sub(lowest);
    }
    Ok(())
}

/// Settles the car (`settle_car`), undoes its matrix (`invert_car_matrix`) and clears its motion:
/// the work car's `velocity` and `spin` 0, `local_velocity` (1.0, 0, 0), `speed` 0; heading, pitch
/// and roll copied to `start_heading` / `start_pitch` / `start_roll` and `camera_heading` /
/// `camera_pitch` / `camera_roll_turn`; `stuck_count`, `drive_speed_a`, `drive_speed_b`, `gear`,
/// `eased_yaw_spin`, `grip_force_sum`, `wall_hit_timer` 0; with `camera_view_return` not 0,
/// `camera_blend_view` and `camera_view` get it less one and `camera_view_return` 0.
pub fn reset_car_motion(
    c: &ResetInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let shared = c.shared;
    let physics = c.physics;
    settle_car(c, scene)?;
    crate::race::start_place::invert_car_matrix(c.course, c.car, scene);
    shared.car().velocity.set([0.0; 3]);
    shared.car().local_velocity.set_at(0, 1.0);
    shared.car().local_velocity.set_at(1, 0.0);
    shared.car().local_velocity.set_at(2, 0.0);
    shared.car().spin.set([0.0; 3]);
    let heading = shared.car().car_heading.get();
    physics.camera_mut().start_heading = heading;
    shared.car().camera_heading.set(heading);
    let pitch = shared.car().car_pitch.get();
    physics.camera_mut().start_pitch = pitch;
    shared.car().camera_pitch.set(pitch);
    let roll = shared.car().car_roll.get();
    physics.camera_mut().start_roll = roll;
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
        shared.buffers.engine_tables(),
        x as i32,
        0,
        z as i32,
        course.course.borrow().place_heading,
    );
    let position = &shared.car().position;
    position.set_bits_at(0, position.bits_at(0).wrapping_add(x));
    position.set_bits_at(2, position.bits_at(2).wrapping_add(z));
}

/// Sets the work car's start: `notice_count`, `camera_switch_active` 0, `map_corner` 0x80000000,
/// the engine's sound revs, engine speed and gear 0, the steering and pedal values 0. Unless
/// `run_kind` is 4: its checkpoint is `first_check_checkpoint` (less the car's number / 2 but with
/// `run_kind` 1 or 3) into `line_place`, `line_fraction` 0 (both 0 with `Finish.no_place`);
/// `place_car_on_line`; without `Finish.no_place` the car is moved back (0, -0x20000 turned by
/// `place_heading`) and, with `session` or (`run_kind` 0 and `players` not 1), to the side
/// (+-0x14000 by the car's number's low bit). Then `nearest_checkpoint` goes into `checkpoint`,
/// `reset_checkpoint`, `last_pace_call`; its check count less one held to 0 .. 100 into
/// `splits_passed` and `splits_recorded`; `reset_car_motion`; `reset_key_count`, `stopped_steps`,
/// `tyre_change_state` 0. Refused: a nearest checkpoint past the lists (when met).
pub fn start_car(
    c: &ResetInputs<'_>,
    scene: &crate::scene::owner::Scene,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<(), Refused> {
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
        scene.line_and_shake_mut().line_place = place;
        scene.line_and_shake_mut().line_fraction = 0;
        if c.no_place != 0 {
            scene.line_and_shake_mut().line_place = 0;
            scene.line_and_shake_mut().line_fraction = 0;
        }
        crate::race::start_place::place_car_on_line(c.shared, course, scene)?;
        if c.no_place == 0 {
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
    let checks = rc2k_core::race::start::start_checks(crate::race::checkpoints_store::check_count(
        c.shared.checkpoints(),
        nearest as usize,
    ));
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
/// car's `part_damage_limits` and first 18 `saved_point_figures` cleared, `drive::store_car`.
/// Refused (before anything): no cars or more than 8.
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
        // Clears the damage limits and the first 18 saved point figures.
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
