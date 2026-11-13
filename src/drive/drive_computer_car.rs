//! `drive::drive_computer_car` (0x462811): the computer's driving of the work car, in
//! `pusha` / `popa`. Its controls (0x71BF88 .. 0x71BFA0: steering, throttle 0x100, brake
//! 0xFF / 0x100, and three switches) start at 0; nothing more below a step count
//! (`run_ticks`) of 800 or with `Car.crash_matrix[5]` below `driver_tilt_limit`. Else, by the
//! checkpoint the car is at (`checkpoint`, n) and its speed (`fistp` of the root of
//! `velocity`² + `Car.velocity[2]`², doubled, `driver_speed`):
//! - the looked-at checkpoint's face's two bytes (+0xA, +0xB) and the line's turn ahead
//!   (`Buffers.checkpoints.angles[2][n + k]`, k by the speed and `wheel_surface_a`: the difference of two of
//!   them >> 16) note a bend (0x71C010, 0x71C038, 0x71C088 by slot `driver_slot`, 0);
//! - a wanted speed from the tables at 0x71BF1C / 0x71BF38 / 0x71BF54 by the bend's turn
//!   (n's, `>> 25`), 25 / 40 / 50 near a bend, eased by `driver_start_blend` above 50; throttle or
//!   brake by the speed against it;
//! - the point aimed at: a checkpoint face's first point (n + 2 .. n + 5 by the speed and
//!   the bend) moved sideways by `bend_speeds`[(n + slot) & 0x1F]` along the line's
//!   normal (`scene::cross_product` with `driver_vector`, `race::normalise_work_vector`);
//! - the steering: the car's forward (row 0's and row 2's z of `g_crashMatrix`) crossed
//!   with the way to that point, its y times 256 `fist`'d, its sign by the product's
//!   (and turned round in reverse) into `driver_steer`;
//! - a reverse count (`driver_reverse_left`), and every 200 steps (`driver_steps`) a check that
//!   the car moved 0x8000 (else reversing for 250 steps, or the reverse tries counted
//!   in `driver_reverse_tries`, the fourth setting `RaceWork.driver_flags[3]`).
//!
//! The checkpoint tables' indices are held to `g_checkpointCount` - 1 (unsigned) as the
//! original holds them. What each value is, is a reading at best.
//!
//! Callers: `drive::read_car_controls`. Refuses (before anything) no checkpoints when the tables
//! are to be read.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87::{
    compare_full, float_hypot_to_int, int_toward_to_int, mul2_to_int, times_fist_compare,
};
use crate::race::norms::normalise_work_vector;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;
use crate::scene::view_matrix::cross_product;
use crate::session::{PlainDword, PlainFloat};
use rc2k_core::drive::computer_driver as driver;

/// `table[n + k]`, the index held to the count less one (unsigned) as the original holds
/// it (through `lea`, `sub` and `shr 2`).
fn held(
    shared: &RaceShared,
    table: fn(&crate::race::checkpoints_store::Checkpoints, usize) -> &crate::session::PlainDword,
    n: u32,
    k: u32,
) -> u32 {
    let i = driver::held_index(n, k, shared.vars.get().checkpoint_count.get());
    table(&shared.buffers.reads.get().checkpoints, i as usize).get()
}

/// A checkpoint face's first point: (x, z).
fn face_point(course: &Course, face: u32) -> Result<(u32, u32), BadInput> {
    let point = crate::race::course_places::face_field::<u32>(course.reads(), face, 4)?;
    Ok((
        crate::race::scene_points::point_word(
            course.reads(),
            &crate::race::scene_points::NO_OWN,
            point,
            0,
        )?,
        crate::race::scene_points::point_word(
            course.reads(),
            &crate::race::scene_points::NO_OWN,
            point,
            2,
        )?,
    ))
}

/// 0x463296 then 0x4632AE: (to - from) in x and z, y 0, as floats, into `out`
/// (the whole numbers written first, then each turned into a float in place).
fn flat_way(from: &[PlainDword], to: &[PlainDword], out: &[PlainFloat]) {
    let x = to[0].get().wrapping_sub(from[0].get());
    let z = to[2].get().wrapping_sub(from[2].get());
    for (o, v) in out.iter().zip([x, 0, z]) {
        o.set(x87::from_int(v as i32));
    }
}

/// Through `g_workVector` and `race::normalise_work_vector`: the vector `from` made one
/// long into `to`.
fn normalised(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    from: &[PlainFloat],
    to: &[PlainFloat],
) {
    let w = &race.frame.scratch().work_vector;
    for k in 0..3 {
        w[k].set(from[k].get().to_bits());
    }
    normalise_work_vector(race.course.numbers.get(), race.frame.scratch(), scene);
    for k in 0..3 {
        to[k].set(f32::from_bits(w[k].get()));
    }
}

/// The slot's point in `points` (`driver_points_now` / `_then`, four dwords a slot), where
/// `driver_point_now` / `_then` point.
fn slot_point(points: &[PlainDword; 40], slot: u32) -> Result<&[PlainDword], BadInput> {
    let at = 4 * slot as usize;
    points
        .get(at..at + 4)
        .ok_or(BadInput("a computer driver's slot past the points"))
}

// Converted: `rc2k_core::drive::computer_driver` (the x87 steps stay).
pub fn drive_computer_car(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let course = &race.course;
    let reaches = race.shared.car().run_ticks.get() >= driver::FIRST_STEP
        && compare_full(
            race.shared.car().crash_matrix[5].get(),
            race.shared.work.get().driver_tilt_limit.get(),
        ) & 0x100
            == 0;
    if reaches && race.shared.vars.get().checkpoint_count.get() == 0 {
        return Err(BadInput("no checkpoints (the tables' indices would be -1)").into());
    }
    let w = race.shared.work.get();
    for v in [
        &w.driver_steer,
        &w.driver_throttle,
        &w.driver_brake,
        &w.driver_flags[0],
        &w.driver_flags[1],
        &w.driver_flags[2],
        &w.driver_lookahead,
        &w.driver_flags[3],
        &w.driver_aim,
        &w.driver_flag_bc,
        // The car's slot too: every computer car uses the per-car arrays' first entry
        // (the game's own slip, `notes/bugs.md`).
        &w.driver_slot,
    ] {
        v.set(0);
    }
    if !reaches {
        return Ok(());
    }
    let slot = race.shared.work.get().driver_slot.get();
    // The slot's points by their byte offset in the tables (the original kept their
    // addresses; it alone read them, as `slot_point` does here).
    race.shared.work.get().driver_point_now.set(slot << 4);
    race.shared.work.get().driver_point_then.set(slot << 4);
    let n = race.shared.car().checkpoint.get();
    let f = held(
        &race.shared,
        crate::race::checkpoints_store::face_run_on,
        n,
        2,
    );
    let (b1, b2) = crate::race::course_places::face_number(race.course.reads(), f)?.bends();
    let (b1, b2) = (u32::from(b1), u32::from(b2));
    race.shared.work.get().driver_bend_a.set(b1);
    race.shared.work.get().driver_bend_b.set(b2);
    if driver::narrow(b1, b2) {
        race.shared.work.get().driver_flag_bc.set(1);
        race.shared.work.get().driver_lookahead.set(1);
    }
    race.shared.work.get().driver_lookahead.set(1);
    let speed = (float_hypot_to_int(
        race.shared.car().velocity[0].get(),
        race.shared.car().velocity[2].get(),
    ) as u32)
        << 1;
    race.shared.work.get().driver_speed.set(speed);
    let edx = race.shared.work.get().driver_slot.get();
    let (first, second, ahead) = driver::lookahead(
        race.shared.work.get().driver_lookahead.get() == 1,
        race.shared.car().wheel_surface_a[0].get(),
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
    race.shared.work.get().driver_turn.set(turn);
    if driver::bend_noted(turn, race.shared.work.get().driver_lookahead.get() == 1) {
        if driver::sharp(turn) {
            race.shared.work.get().driver_bend_noted
                [race.shared.work.get().driver_slot.get() as usize]
                .set(1);
        }
        let to = n.wrapping_add(ahead);
        if driver::bigger_bend(
            turn,
            race.shared.work.get().driver_bend_turn[edx as usize].get(),
        ) {
            race.shared.work.get().driver_bend_turn[edx as usize].set(turn);
            race.shared.work.get().driver_bend_to[edx as usize].set(to);
        }
    }
    let edx = race.shared.work.get().driver_slot.get();
    if driver::bend_passed(n, race.shared.work.get().driver_bend_to[edx as usize].get()) {
        race.shared.work.get().driver_bend_to[edx as usize].set(0);
        race.shared.work.get().driver_bend_turn[edx as usize].set(0);
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
        race.shared.work.get().driver_aim.set(1);
    }
    let table = [0, 7, 14][driver::speed_table(
        race.shared.work.get().driver_lookahead.get() == 1,
        race.shared.car().wheel_surface_a[0].get(),
    )];
    // The tables are bytes (`table_entry` at most 0x18: inside its table's 28).
    let byte = 4 * table + bend as usize;
    let listed = u32::from(
        race.shared.work.get().wanted_speeds[byte / 4]
            .get()
            .to_le_bytes()[byte % 4],
    );
    let s = race.shared.work.get().driver_slot.get();
    let near = race.shared.work.get().driver_bend_to[s as usize].get() != 0;
    let want = driver::wanted_speed(
        listed,
        near,
        near && race.shared.work.get().driver_bend_noted[s as usize].get() != 0,
        speed,
    );
    if !near {
        race.shared.work.get().driver_bend_noted[s as usize].set(0);
    }
    race.shared.work.get().driver_wanted_speed.set(want);
    if driver::eased(want) {
        race.shared
            .work
            .get()
            .driver_wanted_speed
            .set(int_toward_to_int(
                want as i32,
                race.shared.tables().computer_ease_speed.get(),
                race.shared.work.get().driver_start_blend.get(),
            ) as u32);
    }
    let pedals = driver::pedals(speed, race.shared.work.get().driver_wanted_speed.get());
    if let Some(t) = pedals.throttle {
        race.shared.work.get().driver_throttle.set(t);
    }
    if pedals.hard {
        race.shared.work.get().driver_flags[0].set(1);
    }
    if let Some(b) = pedals.brake {
        race.shared.work.get().driver_brake.set(b);
    }
    let k = driver::offset_entry(n, race.shared.work.get().driver_slot.get());
    race.shared
        .work
        .get()
        .driver_speed_scale
        .set(x87::loaded(&race.shared.work.get().bend_speeds[k as usize])); // RaceWork.bend_speeds[0], 0x20 of them
    let (x, z) = face_point(
        &race.course,
        held(
            &race.shared,
            crate::race::checkpoints_store::face_run_on,
            n,
            0,
        ),
    )?;
    race.shared.work.get().driver_from[0].set(x);
    race.shared.work.get().driver_from[2].set(z);
    race.shared.work.get().driver_from[1].set(0);
    let sharp = race.shared.work.get().driver_bend_noted
        [race.shared.work.get().driver_slot.get() as usize]
        .get()
        != 0;
    let aim = driver::aim_ahead(
        sharp,
        !sharp && race.shared.work.get().driver_aim.get() == 1,
        speed,
    );
    let (x, z) = face_point(
        &race.course,
        held(
            &race.shared,
            crate::race::checkpoints_store::face_run_on,
            n,
            aim,
        ),
    )?;
    race.shared.work.get().driver_to[0].set(x);
    race.shared.work.get().driver_to[2].set(z);
    race.shared.work.get().driver_to[1].set(0);
    flat_way(
        &race.shared.work.get().driver_from,
        &race.shared.work.get().driver_to,
        &race.shared.work.get().driver_flat,
    );
    normalised(
        race,
        scene,
        &race.shared.work.get().driver_flat,
        &race.shared.work.get().driver_flat_unit,
    );
    cross_product(
        &race.shared.work.get().driver_flat_unit,
        &race.shared.work.get().driver_vector,
        &race.shared.work.get().driver_ahead,
    );
    normalised(
        race,
        scene,
        &race.shared.work.get().driver_ahead,
        &race.shared.work.get().driver_ahead,
    );
    let (k1, k2) = (
        race.shared.work.get().driver_speed_scale.get(),
        course.numbers.get().k65536.get(),
    );
    race.shared.work.get().driver_step_x.set(mul2_to_int(
        race.shared.work.get().driver_ahead[0].get(),
        k1,
        k2,
    ) as u32);
    race.shared.work.get().driver_step_z.set(mul2_to_int(
        race.shared.work.get().driver_ahead[2].get(),
        k1,
        k2,
    ) as u32);
    race.shared.work.get().driver_to[0].set(
        race.shared.work.get().driver_to[0]
            .get()
            .wrapping_add(race.shared.work.get().driver_step_x.get()),
    );
    race.shared.work.get().driver_to[2].set(
        race.shared.work.get().driver_to[2]
            .get()
            .wrapping_add(race.shared.work.get().driver_step_z.get()),
    );
    let here = slot_point(&race.shared.work.get().driver_points_now, slot)?;
    here[0].set(race.shared.car().position[0].get());
    here[2].set(race.shared.car().position[2].get());
    here[1].set(0);
    flat_way(
        here,
        &race.shared.work.get().driver_to,
        &race.shared.work.get().driver_way,
    );
    normalised(
        race,
        scene,
        &race.shared.work.get().driver_way,
        &race.shared.work.get().driver_way_unit,
    );
    race.shared.work.get().driver_side[2].set(x87::loaded(&race.shared.car().crash_matrix[10]));
    race.shared.work.get().driver_side[0].set(x87::negated(&race.shared.car().crash_matrix[2]));
    race.shared.work.get().driver_side[1].set(0.0);
    cross_product(
        &race.shared.work.get().driver_side,
        &race.shared.work.get().driver_way_unit,
        &race.shared.work.get().driver_work,
    );
    let (i, status) = times_fist_compare(
        race.shared.work.get().driver_work[1].get(),
        course.numbers.get().k256.get(),
        course.numbers.get().zero.get(),
    );
    race.shared.work.get().driver_bend_index.set(i as u32);
    let steer = driver::steering(
        i as u32,
        status & 0x100 != 0,
        race.shared.car().gear.get() == -1,
    );
    race.shared.work.get().driver_steer.set(steer);
    let s = race.shared.work.get().driver_slot.get();
    let (left, reversing, handbrake) = driver::reverse_step(
        race.shared.work.get().driver_reverse_left[s as usize].get(),
        race.shared.car().gear.get(),
    );
    if let Some(left) = left {
        race.shared.work.get().driver_reverse_left[s as usize].set(left);
    }
    if reversing {
        race.shared.work.get().driver_flags[2].set(1);
        race.shared.work.get().driver_brake.set(0x100);
    }
    if handbrake {
        race.shared.work.get().driver_flags[1].set(1);
    }
    let s = race.shared.work.get().driver_slot.get();
    let steps = race.shared.work.get().driver_steps[s as usize]
        .get()
        .wrapping_add(1);
    race.shared.work.get().driver_steps[s as usize].set(steps);
    if !driver::check_due(steps) {
        return Ok(());
    }
    race.shared.work.get().driver_steps[s as usize].set(0);
    let (now, then) = (
        slot_point(&race.shared.work.get().driver_points_now, slot)?,
        slot_point(&race.shared.work.get().driver_points_then, slot)?,
    );
    let (ax, az, cx, cz) = (now[0].get(), now[2].get(), then[0].get(), then[2].get());
    then[0].set(ax);
    then[2].set(az);
    let moved = driver::moved((ax, az), (cx, cz));
    race.shared.work.get().driver_moved.set(moved);
    if !driver::stuck(moved) {
        return Ok(());
    }
    if race.shared.car().gear.get() != -1 {
        race.shared.work.get().driver_reverse_left
            [race.shared.work.get().driver_slot.get() as usize]
            .set(driver::REVERSE_STEPS);
        return Ok(());
    }
    let s = race.shared.work.get().driver_slot.get();
    race.shared.work.get().driver_reverse_left[s as usize].set(0);
    let (tries, fourth) =
        driver::reverse_tries(race.shared.work.get().driver_reverse_tries[s as usize].get());
    if fourth {
        race.shared.work.get().driver_flags[3].set(1);
    }
    race.shared.work.get().driver_reverse_tries[s as usize].set(tries);
    Ok(())
}
