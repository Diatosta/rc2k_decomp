//! `physics::turn_wheel_angles` (0x449EE9): the wheels' turning angles moved on by the
//! speed (a reading: their spin, for the drawing). Only when the car's kind (`g_car`
//! +0x24, `dword_719FFC`) is below 0x22 and its entry in `dword_5C5B48` is not 0 (else CF
//! and nothing). Then, with `dword_71A22C` 0, s = `g_carDriveSpeed` (else 0), held at
//! 0 or more (`flt_5C5D90`), times `flt_623DA8` and `flt_623DB0`, added to the four
//! floats at `dword_623D60`; and s = `g_carDriveSpeed` less `g_carSpeed` (else 0), held
//! the same, times `flt_623DAC` and `flt_623DB4`, added to the four at `dword_623D70`.
//! Each s goes through `dword_62400C`.
//!
//! In:  nothing
//! Out: every register kept; CF when skipped (no caller reads it: the next is an `fld`)
//!
//! Caller: `sub_43B635` (0x43B91E), every physics step.

use crate::math::x87;
use crate::race::kind_store::KIND_TABLES;
use crate::race::owner_course::Course;
use crate::scene::owner::scene;
use crate::session::Float;
use std::cmp::Ordering;

/// `fld zero; fld s; fcompp`, C0: s below zero or unordered.
fn below(s: u32, zero: u32) -> bool {
    !matches!(
        f32::from_bits(s).partial_cmp(&f32::from_bits(zero)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// One half: `speed` (None with the switch set) held, scaled, added to the four at `to`.
fn add_turn(course: &Course, speed: Option<u32>, f1: &Float, f2: &Float, to: &[Float; 4]) {
    let drawing = &scene().drawing;
    drawing.scene.scratch.set(0);
    if let Some(s) = speed {
        drawing.scene.scratch.set(s);
        if below(drawing.scene.scratch.get(), course.numbers.zero.bits()) {
            drawing
                .scene
                .scratch
                .set(x87::loaded(course.numbers.zero.bits()));
        }
    }
    drawing
        .scene
        .scratch
        .set(x87::mul2(drawing.scene.scratch.get(), f1.bits(), f2.bits()));
    for cell in to {
        cell.set_bits(x87::add(cell.bits(), drawing.scene.scratch.get()));
    }
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub fn turn_wheel_angles(course: &Course) {
    let kind = crate::race::owner::race().shared.car.car_kind.get();
    if kind >= 0x22 || KIND_TABLES.kind_turns_wheels[kind as usize].get() == 0 {
        return;
    }
    let still = crate::race::owner::race().shared.car.all_wheels_off.get() != 0;
    let drive = || x87::loaded(crate::race::owner::race().shared.car.drive_speed_a.bits());
    let v = &crate::race::owner::race().shared.vars;
    add_turn(
        course,
        (!still).then(drive),
        &v.wheel_turn_a[0],
        &v.wheel_turn_b[0],
        &v.mud_thrown_in,
    );
    let slip = || {
        x87::sub(
            crate::race::owner::race().shared.car.drive_speed_a.bits(),
            crate::race::owner::race().shared.car.local_velocity[2].bits(),
        )
    };
    add_turn(
        course,
        (!still).then(slip),
        &v.wheel_turn_a[1],
        &v.wheel_turn_b[1],
        &v.mud_splashes_in,
    );
}
