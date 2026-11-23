//! The computer's driving of the work car: from the checkpoint the car is at and its speed it
//! notes the bend ahead, picks a wanted speed (25 / 40 / 50 near a bend, eased by
//! `driver_start_blend`) and sets throttle or brake against it, aims at a point on a checkpoint
//! face a few checkpoints on (moved sideways by `bend_speeds`) and steers by the cross product of
//! the car's forward and the way to it, and reverses for 250 steps when the car has not moved
//! 0x8000 in 200. Controls start at 0; nothing more below a step count of 800 or with
//! `Car.crash_matrix[5]` below `driver_tilt_limit`. The checkpoint tables' indices are held to
//! the checkpoint count - 1 (unsigned). What each value is, is a reading at best. Refuses
//! (before anything) no checkpoints when the tables are to be read.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87::{
    compare_full, float_hypot_to_int, int_toward_to_int, mul2_to_int, times_fist_compare,
};
use crate::memory::plain_floats::FloatRow;
use crate::race::norms::normalise_work_vector;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;
use crate::scene::view_matrix::cross_of;
use rc2k_core::drive::computer_driver as driver;

/// `table[n + k]`, the index held to the count less one (unsigned).
fn held(
    shared: &RaceShared,
    table: fn(crate::race::checkpoints_store::CheckpointsRef, usize) -> u32,
    n: u32,
    k: u32,
) -> u32 {
    let i = driver::held_index(n, k, shared.vars.get().checkpoint_count.get());
    table(shared.checkpoints(), i as usize)
}

/// A checkpoint face's first point: (x, z).
fn face_point(course: &Course, face: u32) -> Result<(u32, u32), BadInput> {
    let point = crate::race::course_places::face_field::<u32>(course.reads(), face, 4)?;
    Ok((
        crate::race::scene_points::point_word(
            course.reads(),
            &crate::race::scene_points::no_own(),
            point,
            0,
        )?,
        crate::race::scene_points::point_word(
            course.reads(),
            &crate::race::scene_points::no_own(),
            point,
            2,
        )?,
    ))
}

/// (to - from) in x and z, y 0, as floats (the whole numbers turned into floats).
fn flat_way(from: &[u32], to: &[u32]) -> [f32; 3] {
    let x = to[0].wrapping_sub(from[0]);
    let z = to[2].wrapping_sub(from[2]);
    [x, 0, z].map(|v| x87::from_int(v as i32))
}

/// The vector `from` made one long, through the work vector and `race::normalise_work_vector`.
fn normalised(race: &Race, scene: &crate::scene::owner::Scene, from: &[f32]) -> [f32; 3] {
    let mut scratch = race.frame.scratch_mut();
    for k in 0..3 {
        scratch.work_vector[k] = from[k].to_bits();
    }
    normalise_work_vector(&race.course.numbers, &mut scratch, scene);
    std::array::from_fn(|k| f32::from_bits(scratch.work_vector[k]))
}

/// The range of the slot's point in `driver_points_now` / `_then` (four dwords a slot), where
/// `driver_point_now` / `_then` point.
fn slot_point(slot: u32) -> Result<std::ops::Range<usize>, BadInput> {
    let at = 4 * slot as usize;
    if at + 4 > 40 {
        return Err(BadInput("a computer driver's slot past the points"));
    }
    Ok(at..at + 4)
}

pub fn drive_computer_car(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let course = &race.course;
    let reaches = race.shared.car().run_ticks >= driver::FIRST_STEP
        && compare_full(
            race.shared.car().crash_matrix.at(5),
            race.shared.work().driver_tilt_limit,
        ) & 0x100
            == 0;
    if reaches && race.shared.vars.get().checkpoint_count.get() == 0 {
        return Err(BadInput("no checkpoints (the tables' indices would be -1)").into());
    }
    // Held to the end: nothing called below reads the race's work variables.
    let mut w = race.shared.work_mut();
    w.driver_steer = 0;
    w.driver_throttle = 0;
    w.driver_brake = 0;
    w.driver_flags[0] = 0;
    w.driver_flags[1] = 0;
    w.driver_flags[2] = 0;
    w.driver_lookahead = 0;
    w.driver_flags[3] = 0;
    w.driver_aim = 0;
    w.driver_flag_bc = 0;
    // The car's slot too: every computer car uses the per-car arrays' first entry
    // (the game's own slip, kept as is).
    w.driver_slot = 0;
    if !reaches {
        return Ok(());
    }
    let slot = w.driver_slot;
    // The slot's points by their byte offset in the tables (only `slot_point` reads them).
    w.driver_point_now = slot << 4;
    w.driver_point_then = slot << 4;
    let n = race.shared.car().checkpoint;
    let f = held(
        &race.shared,
        crate::race::checkpoints_store::face_run_on,
        n,
        2,
    );
    let (b1, b2) = crate::race::course_places::face_number(race.course.reads(), f)?.bends();
    let (b1, b2) = (u32::from(b1), u32::from(b2));
    w.driver_bend_a = b1;
    w.driver_bend_b = b2;
    if driver::narrow(b1, b2) {
        w.driver_flag_bc = 1;
        w.driver_lookahead = 1;
    }
    w.driver_lookahead = 1;
    let speed = (float_hypot_to_int(
        race.shared.car().velocity.at(0),
        race.shared.car().velocity.at(2),
    ) as u32)
        << 1;
    w.driver_speed = speed;
    let driver_slot = w.driver_slot;
    let (first, second, ahead) = driver::lookahead(
        w.driver_lookahead == 1,
        race.shared.car().wheel_surface_a[0],
        speed,
    );
    let turn = driver::turn(
        held(
            &race.shared,
            crate::race::checkpoints_store::angle_run_on,
            n,
            first,
        ),
        held(
            &race.shared,
            crate::race::checkpoints_store::angle_run_on,
            n,
            second,
        ),
    );
    w.driver_turn = turn;
    if driver::bend_noted(turn, w.driver_lookahead == 1) {
        if driver::sharp(turn) {
            let at = w.driver_slot as usize;
            w.driver_bend_noted[at] = 1;
        }
        let to = n.wrapping_add(ahead);
        if driver::bigger_bend(turn, w.driver_bend_turn[driver_slot as usize]) {
            w.driver_bend_turn[driver_slot as usize] = turn;
            w.driver_bend_to[driver_slot as usize] = to;
        }
    }
    let driver_slot = w.driver_slot;
    if driver::bend_passed(n, w.driver_bend_to[driver_slot as usize]) {
        w.driver_bend_to[driver_slot as usize] = 0;
        w.driver_bend_turn[driver_slot as usize] = 0;
    }
    let bend = driver::table_entry(
        held(
            &race.shared,
            crate::race::checkpoints_store::angle_run_on,
            n,
            0,
        ),
        held(
            &race.shared,
            crate::race::checkpoints_store::angle_run_on,
            n,
            4,
        ),
    );
    if driver::tight(bend) {
        w.driver_aim = 1;
    }
    let table = [0, 7, 14][driver::speed_table(
        w.driver_lookahead == 1,
        race.shared.car().wheel_surface_a[0],
    )];
    // The tables are bytes (`table_entry` at most 0x18: inside its table's 28).
    let byte = 4 * table + bend as usize;
    let listed = u32::from(w.wanted_speeds[byte / 4].to_le_bytes()[byte % 4]);
    let s = w.driver_slot;
    let near = w.driver_bend_to[s as usize] != 0;
    let want = driver::wanted_speed(
        listed,
        near,
        near && w.driver_bend_noted[s as usize] != 0,
        speed,
    );
    if !near {
        w.driver_bend_noted[s as usize] = 0;
    }
    w.driver_wanted_speed = want;
    if driver::eased(want) {
        w.driver_wanted_speed = int_toward_to_int(
            want as i32,
            race.shared.tables().computer_ease_speed.get(),
            w.driver_start_blend,
        ) as u32;
    }
    let pedals = driver::pedals(speed, w.driver_wanted_speed);
    if let Some(t) = pedals.throttle {
        w.driver_throttle = t;
    }
    if pedals.hard {
        w.driver_flags[0] = 1;
    }
    if let Some(b) = pedals.brake {
        w.driver_brake = b;
    }
    let k = driver::offset_entry(n, w.driver_slot);
    w.driver_speed_scale = x87::loaded(w.bend_speeds[k as usize]);
    let (x, z) = face_point(
        &race.course,
        held(
            &race.shared,
            crate::race::checkpoints_store::face_run_on,
            n,
            0,
        ),
    )?;
    w.driver_from[0] = x;
    w.driver_from[2] = z;
    w.driver_from[1] = 0;
    let sharp = w.driver_bend_noted[w.driver_slot as usize] != 0;
    let aim = driver::aim_ahead(sharp, !sharp && w.driver_aim == 1, speed);
    let (x, z) = face_point(
        &race.course,
        held(
            &race.shared,
            crate::race::checkpoints_store::face_run_on,
            n,
            aim,
        ),
    )?;
    w.driver_to[0] = x;
    w.driver_to[2] = z;
    w.driver_to[1] = 0;
    let flat = flat_way(&w.driver_from, &w.driver_to);
    w.driver_flat[..3].copy_from_slice(&flat);
    let unit = normalised(race, scene, &w.driver_flat);
    w.driver_flat_unit[..3].copy_from_slice(&unit);
    let ahead = cross_of(&w.driver_flat_unit, &w.driver_vector);
    w.driver_ahead[..3].copy_from_slice(&ahead);
    let ahead = normalised(race, scene, &w.driver_ahead);
    w.driver_ahead[..3].copy_from_slice(&ahead);
    let (k1, k2) = (w.driver_speed_scale, course.numbers.k65536.get());
    w.driver_step_x = mul2_to_int(w.driver_ahead[0], k1, k2) as u32;
    w.driver_step_z = mul2_to_int(w.driver_ahead[2], k1, k2) as u32;
    w.driver_to[0] = w.driver_to[0].wrapping_add(w.driver_step_x);
    w.driver_to[2] = w.driver_to[2].wrapping_add(w.driver_step_z);
    let here = slot_point(slot)?;
    w.driver_points_now[here.start] = race.shared.car().position.bits_at(0);
    w.driver_points_now[here.start + 2] = race.shared.car().position.bits_at(2);
    w.driver_points_now[here.start + 1] = 0;
    let way = flat_way(&w.driver_points_now[here], &w.driver_to);
    w.driver_way[..3].copy_from_slice(&way);
    let unit = normalised(race, scene, &w.driver_way);
    w.driver_way_unit[..3].copy_from_slice(&unit);
    w.driver_side[2] = x87::loaded(race.shared.car().crash_matrix.at(10));
    w.driver_side[0] = x87::negated(race.shared.car().crash_matrix.at(2));
    w.driver_side[1] = 0.0;
    let cross = cross_of(&w.driver_side, &w.driver_way_unit);
    for (k, v) in cross.into_iter().enumerate() {
        w.driver_work[k] = v.to_bits();
    }
    let (i, status) = times_fist_compare(
        w.driver_work[1],
        course.numbers.k256.get(),
        course.numbers.zero.get(),
    );
    w.driver_bend_index = i as u32;
    let steer = driver::steering(i as u32, status & 0x100 != 0, race.shared.car().gear == -1);
    w.driver_steer = steer;
    let s = w.driver_slot;
    let (left, reversing, handbrake) =
        driver::reverse_step(w.driver_reverse_left[s as usize], race.shared.car().gear);
    if let Some(left) = left {
        w.driver_reverse_left[s as usize] = left;
    }
    if reversing {
        w.driver_flags[2] = 1;
        w.driver_brake = 0x100;
    }
    if handbrake {
        w.driver_flags[1] = 1;
    }
    let s = w.driver_slot;
    let steps = w.driver_steps[s as usize].wrapping_add(1);
    w.driver_steps[s as usize] = steps;
    if !driver::check_due(steps) {
        return Ok(());
    }
    w.driver_steps[s as usize] = 0;
    let (now, then) = (slot_point(slot)?, slot_point(slot)?);
    let (ax, az) = (
        w.driver_points_now[now.start],
        w.driver_points_now[now.start + 2],
    );
    let (cx, cz) = (
        w.driver_points_then[then.start],
        w.driver_points_then[then.start + 2],
    );
    w.driver_points_then[then.start] = ax;
    w.driver_points_then[then.start + 2] = az;
    let moved = driver::moved((ax, az), (cx, cz));
    w.driver_moved = moved;
    if !driver::stuck(moved) {
        return Ok(());
    }
    if race.shared.car().gear != -1 {
        let at = w.driver_slot as usize;
        w.driver_reverse_left[at] = driver::REVERSE_STEPS;
        return Ok(());
    }
    let s = w.driver_slot;
    w.driver_reverse_left[s as usize] = 0;
    let (tries, fourth) = driver::reverse_tries(w.driver_reverse_tries[s as usize]);
    if fourth {
        w.driver_flags[3] = 1;
    }
    w.driver_reverse_tries[s as usize] = tries;
    Ok(())
}
