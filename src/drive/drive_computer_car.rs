//! `drive::drive_computer_car` (0x462811): the computer's driving of the work car, in
//! `pusha` / `popa`. Its controls (0x71BF88 .. 0x71BFA0: steering, throttle 0x100, brake
//! 0xFF / 0x100, and three switches) start at 0; nothing more below a step count
//! (`dword_719FEC`) of 800 or with `flt_71A0A0` below `flt_71BF70`. Else, by the
//! checkpoint the car is at (`dword_71A070`, n) and its speed (`fistp` of the root of
//! `flt_71A170`² + `flt_71A178`², doubled, `dword_71C00C`):
//! - the looked-at checkpoint's face's two bytes (+0xA, +0xB) and the line's turn ahead
//!   (`dword_9ED808[n + k]`, k by the speed and `dword_71A8F8`: the difference of two of
//!   them >> 16) note a bend (0x71C010, 0x71C038, 0x71C088 by slot `dword_71C0CC`, 0);
//! - a wanted speed from the tables at 0x71BF1C / 0x71BF38 / 0x71BF54 by the bend's turn
//!   (n's, `>> 25`), 25 / 40 / 50 near a bend, eased by `flt_71BFA4` above 50; throttle or
//!   brake by the speed against it;
//! - the point aimed at: a checkpoint face's first point (n + 2 .. n + 5 by the speed and
//!   the bend) moved sideways by `flt_71BE9C[(n + slot) & 0x1F]` along the line's
//!   normal (`scene::cross_product` with `flt_71BF74`, `race::normalise_work_vector`);
//! - the steering: the car's forward (row 0's and row 2's z of `g_crashMatrix`) crossed
//!   with the way to that point, its y times 256 `fist`'d, its sign by the product's
//!   (and turned round in reverse) into `dword_71BF88`;
//! - a reverse count (`dword_71BFD0`), and every 200 steps (`dword_71BFA8`) a check that
//!   the car moved 0x8000 (else reversing for 250 steps, or the reverse tries counted
//!   in `dword_71C060`, the fourth setting `dword_71BFA0`).
//!
//! The checkpoint tables' indices are held to `g_checkpointCount` - 1 (unsigned) as the
//! original holds them. What each value is, is a reading at best.
//!
//! In:  nothing
//! Out: every register kept; no flags read (callers go on with `push`)
//!
//! Callers: `sub_43A35A` (0x43A497, 0x43A4F7). Refuses (before anything) no checkpoints
//! when the tables are to be read.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_run::{
    compare_full, float_hypot_to_int, int_toward_to_int, mul2_to_int, times_fist_compare,
};
use crate::physics::car_store::CAR;
use crate::race::course_store::COURSE;
use crate::race::norms::normalise_work_vector;
use crate::race::numbers_store::NUMBERS;
use crate::race::scratch_store::FRAME_SCRATCH;
use crate::race::tables_store::RACE_TABLES;
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;
use crate::scene::view_matrix::cross_product;
use crate::session::{Dword, Float};
use rc2k_core::drive::computer_driver as driver;

fn at(addr: u32) -> Result<u32, BadInput> {
    crate::race::course_places::read(addr, "the computer driver's data")
}

/// `table[n + k]`, the index held to the count less one (unsigned) as the original holds
/// it (through `lea`, `sub` and `shr 2`).
fn held(table: fn(usize) -> &'static crate::session::Dword, n: u32, k: u32) -> u32 {
    let i = driver::held_index(n, k, RACE_VARS.checkpoint_count.get());
    table(i as usize).get()
}

/// A checkpoint face's first point: (x, z).
fn face_point(face: u32) -> Result<(u32, u32), BadInput> {
    let f = COURSE.course_faces.get().wrapping_add(face << 6);
    let pt = COURSE
        .scene_points
        .get()
        .wrapping_add(at(f + 4)?.wrapping_mul(12));
    Ok((
        crate::race::scene_points::word(pt)?,
        crate::race::scene_points::word(pt + 8)?,
    ))
}

/// `sub_463296` then `sub_4632AE`: (to - from) in x and z, y 0, as floats, into `out`
/// (the whole numbers written first, then each turned into a float in place).
fn flat_way(from: &[Dword], to: &[Dword], out: &[Float]) {
    let x = to[0].get().wrapping_sub(from[0].get());
    let z = to[2].get().wrapping_sub(from[2].get());
    for (o, v) in out.iter().zip([x, 0, z]) {
        o.set_bits(x87::from_int(v as i32));
    }
}

/// Through `g_workVector` and `race::normalise_work_vector`: the vector `from` made one
/// long into `to`.
fn normalised(from: &[Float], to: &[Float]) {
    let w = &FRAME_SCRATCH.work_vector;
    for k in 0..3 {
        w[k].set(from[k].bits());
    }
    normalise_work_vector();
    for k in 0..3 {
        to[k].set_bits(w[k].get());
    }
}

/// The slot's point in `points` (`driver_points_now` / `_then`, four dwords a slot), where
/// `driver_point_now` / `_then` point.
fn slot_point(points: &[Dword; 40], slot: u32) -> Result<&[Dword], BadInput> {
    let at = 4 * slot as usize;
    points
        .get(at..at + 4)
        .ok_or(BadInput("a computer driver's slot past the points"))
}

/// The Rust version.
// Converted: `rc2k_core::drive::computer_driver` (the x87 steps stay).
pub fn drive_computer_car() -> Result<(), Refused> {
    let reaches = CAR.run_ticks.get() >= driver::FIRST_STEP
        && compare_full(
            CAR.crash_matrix[5].bits(),
            RACE_WORK.driver_tilt_limit.bits(),
        ) & 0x100
            == 0;
    if reaches && RACE_VARS.checkpoint_count.get() == 0 {
        return Err(BadInput("no checkpoints (the tables' indices would be -1)").into());
    }
    let w = &RACE_WORK;
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
    let slot = RACE_WORK.driver_slot.get();
    RACE_WORK
        .driver_point_now
        .set((RACE_WORK.driver_points_now[0].addr()).wrapping_add(slot << 4));
    RACE_WORK
        .driver_point_then
        .set((RACE_WORK.driver_points_then[0].addr()).wrapping_add(slot << 4));
    let n = CAR.checkpoint.get();
    let f = COURSE
        .course_faces
        .get()
        .wrapping_add(held(crate::race::checkpoints_store::face, n, 2) << 6);
    let (b1, b2) = crate::race::course_places::face(f, "a checkpoint face")?.bends();
    let (b1, b2) = (u32::from(b1), u32::from(b2));
    RACE_WORK.driver_bend_a.set(b1);
    RACE_WORK.driver_bend_b.set(b2);
    if driver::narrow(b1, b2) {
        RACE_WORK.driver_flag_bc.set(1);
        RACE_WORK.driver_lookahead.set(1);
    }
    RACE_WORK.driver_lookahead.set(1);
    let speed = (float_hypot_to_int(CAR.velocity[0].bits(), CAR.velocity[2].bits()) as u32) << 1;
    RACE_WORK.driver_speed.set(speed);
    let edx = RACE_WORK.driver_slot.get();
    let (first, second, ahead) = driver::lookahead(
        RACE_WORK.driver_lookahead.get() == 1,
        CAR.wheel_surface_a[0].get(),
        speed,
    );
    let turn = driver::turn(
        held(crate::race::checkpoints_store::angle, n, first),
        held(crate::race::checkpoints_store::angle, n, second),
    );
    RACE_WORK.driver_turn.set(turn);
    if driver::bend_noted(turn, RACE_WORK.driver_lookahead.get() == 1) {
        if driver::sharp(turn) {
            RACE_WORK.driver_bend_noted[RACE_WORK.driver_slot.get() as usize].set(1);
        }
        let to = n.wrapping_add(ahead);
        if driver::bigger_bend(turn, RACE_WORK.driver_bend_turn[edx as usize].get()) {
            RACE_WORK.driver_bend_turn[edx as usize].set(turn);
            RACE_WORK.driver_bend_to[edx as usize].set(to);
        }
    }
    let edx = RACE_WORK.driver_slot.get();
    if driver::bend_passed(n, RACE_WORK.driver_bend_to[edx as usize].get()) {
        RACE_WORK.driver_bend_to[edx as usize].set(0);
        RACE_WORK.driver_bend_turn[edx as usize].set(0);
    }
    let bend = driver::table_entry(
        held(crate::race::checkpoints_store::angle, n, 0),
        held(crate::race::checkpoints_store::angle, n, 4),
    );
    if driver::tight(bend) {
        RACE_WORK.driver_aim.set(1);
    }
    let table = [0, 7, 14][driver::speed_table(
        RACE_WORK.driver_lookahead.get() == 1,
        CAR.wheel_surface_a[0].get(),
    )];
    // The tables are bytes (`table_entry` at most 0x18: inside its table's 28).
    let byte = 4 * table + bend as usize;
    let listed = u32::from(RACE_WORK.wanted_speeds[byte / 4].get().to_le_bytes()[byte % 4]);
    let s = RACE_WORK.driver_slot.get();
    let near = RACE_WORK.driver_bend_to[s as usize].get() != 0;
    let want = driver::wanted_speed(
        listed,
        near,
        near && RACE_WORK.driver_bend_noted[s as usize].get() != 0,
        speed,
    );
    if !near {
        RACE_WORK.driver_bend_noted[s as usize].set(0);
    }
    RACE_WORK.driver_wanted_speed.set(want);
    if driver::eased(want) {
        RACE_WORK.driver_wanted_speed.set(int_toward_to_int(
            want as i32,
            RACE_TABLES.unknown_88c.get(),
            RACE_WORK.driver_start_blend.bits(),
        ) as u32);
    }
    let pedals = driver::pedals(speed, RACE_WORK.driver_wanted_speed.get());
    if let Some(t) = pedals.throttle {
        RACE_WORK.driver_throttle.set(t);
    }
    if pedals.hard {
        RACE_WORK.driver_flags[0].set(1);
    }
    if let Some(b) = pedals.brake {
        RACE_WORK.driver_brake.set(b);
    }
    let k = driver::offset_entry(n, RACE_WORK.driver_slot.get());
    RACE_WORK
        .driver_speed_scale
        .set_bits(x87::loaded(RACE_WORK.bend_speeds[k as usize].bits())); // flt_71BE9C, 0x20 of them
    let (x, z) = face_point(held(crate::race::checkpoints_store::face, n, 0))?;
    RACE_WORK.driver_from[0].set(x);
    RACE_WORK.driver_from[2].set(z);
    RACE_WORK.driver_from[1].set(0);
    let sharp = RACE_WORK.driver_bend_noted[RACE_WORK.driver_slot.get() as usize].get() != 0;
    let aim = driver::aim_ahead(sharp, !sharp && RACE_WORK.driver_aim.get() == 1, speed);
    let (x, z) = face_point(held(crate::race::checkpoints_store::face, n, aim))?;
    RACE_WORK.driver_to[0].set(x);
    RACE_WORK.driver_to[2].set(z);
    RACE_WORK.driver_to[1].set(0);
    flat_way(
        &RACE_WORK.driver_from,
        &RACE_WORK.driver_to,
        &RACE_WORK.driver_flat,
    );
    normalised(&RACE_WORK.driver_flat, &RACE_WORK.driver_flat_unit);
    cross_product(
        &RACE_WORK.driver_flat_unit,
        &RACE_WORK.driver_vector,
        &RACE_WORK.driver_ahead,
    );
    normalised(&RACE_WORK.driver_ahead, &RACE_WORK.driver_ahead);
    let (k1, k2) = (RACE_WORK.driver_speed_scale.bits(), NUMBERS.k65536.bits());
    RACE_WORK
        .driver_step_x
        .set(mul2_to_int(RACE_WORK.driver_ahead[0].bits(), k1, k2) as u32);
    RACE_WORK
        .driver_step_z
        .set(mul2_to_int(RACE_WORK.driver_ahead[2].bits(), k1, k2) as u32);
    RACE_WORK.driver_to[0].set(
        RACE_WORK.driver_to[0]
            .get()
            .wrapping_add(RACE_WORK.driver_step_x.get()),
    );
    RACE_WORK.driver_to[2].set(
        RACE_WORK.driver_to[2]
            .get()
            .wrapping_add(RACE_WORK.driver_step_z.get()),
    );
    let here = slot_point(&RACE_WORK.driver_points_now, slot)?;
    here[0].set(CAR.position[0].get());
    here[2].set(CAR.position[2].get());
    here[1].set(0);
    flat_way(here, &RACE_WORK.driver_to, &RACE_WORK.driver_way);
    normalised(&RACE_WORK.driver_way, &RACE_WORK.driver_way_unit);
    RACE_WORK.driver_side[2].set_bits(x87::loaded(CAR.crash_matrix[10].bits()));
    RACE_WORK.driver_side[0].set_bits(x87::negated(CAR.crash_matrix[2].bits()));
    RACE_WORK.driver_side[1].set_bits(0);
    cross_product(
        &RACE_WORK.driver_side,
        &RACE_WORK.driver_way_unit,
        &RACE_WORK.driver_work,
    );
    let (i, status) = times_fist_compare(
        RACE_WORK.driver_work[1].get(),
        NUMBERS.k256.bits(),
        NUMBERS.zero.bits(),
    );
    RACE_WORK.driver_bend_index.set(i as u32);
    let steer = driver::steering(i as u32, status & 0x100 != 0, CAR.gear.get() == u32::MAX);
    RACE_WORK.driver_steer.set(steer);
    let s = RACE_WORK.driver_slot.get();
    let (left, reversing, handbrake) = driver::reverse_step(
        RACE_WORK.driver_reverse_left[s as usize].get(),
        CAR.gear.get(),
    );
    if let Some(left) = left {
        RACE_WORK.driver_reverse_left[s as usize].set(left);
    }
    if reversing {
        RACE_WORK.driver_flags[2].set(1);
        RACE_WORK.driver_brake.set(0x100);
    }
    if handbrake {
        RACE_WORK.driver_flags[1].set(1);
    }
    let s = RACE_WORK.driver_slot.get();
    let steps = RACE_WORK.driver_steps[s as usize].get().wrapping_add(1);
    RACE_WORK.driver_steps[s as usize].set(steps);
    if !driver::check_due(steps) {
        return Ok(());
    }
    RACE_WORK.driver_steps[s as usize].set(0);
    let (now, then) = (
        slot_point(&RACE_WORK.driver_points_now, slot)?,
        slot_point(&RACE_WORK.driver_points_then, slot)?,
    );
    let (ax, az, cx, cz) = (now[0].get(), now[2].get(), then[0].get(), then[2].get());
    then[0].set(ax);
    then[2].set(az);
    let moved = driver::moved((ax, az), (cx, cz));
    RACE_WORK.driver_moved.set(moved);
    if !driver::stuck(moved) {
        return Ok(());
    }
    if CAR.gear.get() != u32::MAX {
        RACE_WORK.driver_reverse_left[RACE_WORK.driver_slot.get() as usize]
            .set(driver::REVERSE_STEPS);
        return Ok(());
    }
    let s = RACE_WORK.driver_slot.get();
    RACE_WORK.driver_reverse_left[s as usize].set(0);
    let (tries, fourth) = driver::reverse_tries(RACE_WORK.driver_reverse_tries[s as usize].get());
    if fourth {
        RACE_WORK.driver_flags[3].set(1);
    }
    RACE_WORK.driver_reverse_tries[s as usize].set(tries);
    Ok(())
}
