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
use crate::physics::car_store::CAR;
use crate::race::kind_store::KIND_TABLES;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_store::DRAW;
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
fn add_turn(speed: Option<u32>, f1: &Float, f2: &Float, to: &[Float; 4]) {
    DRAW.scratch.set(0);
    if let Some(s) = speed {
        DRAW.scratch.set(s);
        if below(DRAW.scratch.get(), NUMBERS.zero.bits()) {
            DRAW.scratch.set(x87::loaded(NUMBERS.zero.bits()));
        }
    }
    DRAW.scratch
        .set(x87::mul2(DRAW.scratch.get(), f1.bits(), f2.bits()));
    for cell in to {
        cell.set_bits(x87::add(cell.bits(), DRAW.scratch.get()));
    }
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn turn_wheel_angles() {
    let kind = CAR.car_kind.get();
    if kind >= 0x22 || KIND_TABLES.kind_turns_wheels[kind as usize].get() == 0 {
        return;
    }
    let still = CAR.all_wheels_off.get() != 0;
    let drive = || x87::loaded(CAR.drive_speed_a.bits());
    let v = &RACE_VARS;
    add_turn(
        (!still).then(drive),
        &v.wheel_turn_a[0],
        &v.wheel_turn_b[0],
        &v.mud_thrown_in,
    );
    let slip = || x87::sub(CAR.drive_speed_a.bits(), CAR.local_velocity[2].bits());
    add_turn(
        (!still).then(slip),
        &v.wheel_turn_a[1],
        &v.wheel_turn_b[1],
        &v.mud_splashes_in,
    );
}
