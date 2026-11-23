//! Race initialisation's first step, the cars' starts (`start_cars`): each car of the race is
//! loaded into the work copy (`car`), set at its start (`start_car`: its checkpoint, its place on
//! the line and to the side, the nearest checkpoint, then `reset_car_motion`: settled on the ground
//! by `settle_car`, its matrix undone, its motion cleared), two blocks cleared, and stored back.
//! Uses `physics::set_handling` and `physics::probe_ground`.

use crate::errors::{BadInput, Refused};
use crate::memory::session::DrawPlain;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ProbeLists;
use crate::physics::step_families::{GroundInputs, ResetInputs};
use crate::race::best_line::MOST_CHECKS;
use crate::race::matrices_store::SceneMatrices;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;

/// The most cars `car_blocks` has.
pub const MOST_CARS: u32 = 8;

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
fn settle_step(low: u32, high: u32, angle: &mut u32, step: &mut u32) {
    let s = *step;
    if (low.wrapping_sub(high) as i32) < 0 {
        *angle = angle.wrapping_sub(s);
    } else {
        *angle = angle.wrapping_add(s);
    }
    *step = s >> 1;
}

/// The matrix (`get_mat_crash`) and the wheels (`physics::probe_ground`).
fn matrix_and_wheels(
    engine_tables: crate::math::engine_tables::EngineTablesRef,
    ground: &GroundInputs<'_>,
    matrices: &DrawPlain<SceneMatrices>,
    scene: &crate::scene::owner::Scene,
    car: &mut Car,
) -> Result<[u32; 4], Refused> {
    crate::race::stage_prefs::get_mat_crash(engine_tables, matrices, car, scene);
    crate::physics::probe_ground::probe_ground(ground, car)?;
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
    shared.car_mut().car_pitch = 0;
    shared.car_mut().car_roll = 0;
    shared.physics_tables_mut().settle_steps[0] = 0x2000_0000u32;
    shared.physics_tables_mut().settle_steps[1] = 0x2000_0000u32;
    physics.ground_probe().borrow_mut().settle_mark = 0x80;
    for _ in 0..8 {
        let w = matrix_and_wheels(
            shared.buffers.engine_tables(),
            &c.ground,
            c.matrices,
            scene,
            &mut shared.car_mut(),
        )?;
        settle_step(
            w[0].wrapping_add(w[2]),
            w[1].wrapping_add(w[3]),
            &mut shared.car_mut().car_roll,
            &mut shared.physics_tables_mut().settle_steps[1],
        );
        settle_step(
            w[0].wrapping_add(w[1]),
            w[2].wrapping_add(w[3]),
            &mut shared.car_mut().car_pitch,
            &mut shared.physics_tables_mut().settle_steps[0],
        );
    }
    let w = matrix_and_wheels(
        shared.buffers.engine_tables(),
        &c.ground,
        c.matrices,
        scene,
        &mut shared.car_mut(),
    )?;
    let mut lowest = 0x64_0000u32;
    for &h in &w {
        if (h.wrapping_sub(lowest) as i32) < 0 {
            lowest = h;
        }
    }
    {
        let mut car = shared.car_mut();
        car.position[1] = f32::from_bits(car.position[1].to_bits().wrapping_sub(lowest));
    }
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
    crate::race::start_place::invert_car_matrix(c.course, &mut shared.car_mut(), scene);
    let mut car = shared.car_mut();
    car.velocity = [0.0; 3];
    car.local_velocity = [1.0, 0.0, 0.0];
    car.spin = [0.0; 3];
    let heading = car.car_heading;
    physics.camera_mut().start_heading = heading;
    car.camera_heading = heading;
    let pitch = car.car_pitch;
    physics.camera_mut().start_pitch = pitch;
    car.camera_pitch = pitch;
    let roll = car.car_roll;
    physics.camera_mut().start_roll = roll;
    car.camera_roll_turn = roll;
    car.stuck_count = 0;
    car.drive_speed_a = 0.0;
    car.drive_speed_b = 0.0;
    car.gear = 0;
    car.eased_yaw_spin = 0.0;
    car.grip_force_sum = 0.0;
    car.speed = 0.0;
    car.wall_hit_timer = 0;
    let pending = car.camera_view_return;
    if pending != 0 {
        car.camera_blend_view = pending - 1;
        car.camera_view = pending - 1;
        car.camera_view_return = 0;
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
    let position = &mut shared.car_mut().position;
    position[0] = f32::from_bits(position[0].to_bits().wrapping_add(x));
    position[2] = f32::from_bits(position[2].to_bits().wrapping_add(z));
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
    {
        let mut car = c.shared.car_mut();
        car.notice_count = 0;
        car.camera_switch_active = 0;
        car.map_corner = [0x8000_0000; 2];
        car.engine_sound_revs = 0;
        car.engine_speed = 0.0;
        car.gear = 0;
        car.steering = 0;
        car.steering_raw = 0;
        car.steer_angle = 0;
        car.steer_angle_visual = 0;
        car.throttle = 0;
        car.brake = 0;
        car.handbrake = 0;
    }
    if c.vars.run_kind.get() != 4 || crate::debug::screens::free_play_placed() {
        // The start's pure parts are in `rc2k_core::race::start`.
        use rc2k_core::race::start::{side_offset, start_checkpoint, starts_aside};
        let first = crate::race::race_init::first_check_checkpoint(c.course, c.shared)?;
        let mode: u32 = c.vars.run_kind.get();
        let place = start_checkpoint(first, mode, c.work.borrow().current_car);
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
                move_turned(
                    c.course,
                    c.shared,
                    side_offset(c.work.borrow().current_car),
                    0,
                );
            }
        }
    }
    let nearest =
        crate::race::start_place::nearest_checkpoint(c.physics, c.course, c.shared, scene)?;
    {
        let mut car = c.shared.car_mut();
        car.checkpoint = nearest;
        car.reset_checkpoint = nearest;
        car.last_pace_call = nearest;
    }
    if nearest >= MOST_CHECKS {
        return Err(BadInput("a nearest checkpoint past the lists").into());
    }
    let checks = rc2k_core::race::start::start_checks(crate::race::checkpoints_store::check_count(
        c.shared.checkpoints(),
        nearest as usize,
    ));
    {
        let mut car = c.shared.car_mut();
        car.splits_passed = checks;
        car.splits_recorded = checks;
    }
    reset_car_motion(c, scene)?;
    let mut car = c.shared.car_mut();
    car.reset_key_count = 0;
    car.stopped_steps = 0;
    car.tyre_change_state = 0;
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
        shared.work_mut().current_car = n;
        crate::drive::load_car::load_car(shared)?;
        start_car(
            &crate::physics::step_inputs::StepInputs::of(race).reset(),
            scene,
            net_race,
        )?;
        // Clears the damage limits and the first 18 saved point figures.
        {
            let mut car = shared.car_mut();
            car.part_damage_limits = [0; 7];
            car.saved_point_figures[..18].fill(0);
        }
        crate::drive::store_car::store_car(shared)?;
    }
    Ok(())
}
