//! `physics::engine_torque` (0x43CBBB): the engine's torque from its speed. s = engine
//! speed / its top (+0x1DC) into `dword_62400C`; for s in 0 ..= below 1 (`flt_5C5D90`,
//! `flt_5C5D38`; unordered counts as out) the curve's entry i = s x 8 (`fistp` with the
//! rounding set to chop) into `dword_623FB8` (ebx), and `flt_614748` the line between
//! (`flt_613FF0[i]`, `dword_614B7C[i]`) and (`flt_613FF0[i + 1]`, `dword_614B7C[i + 1]`)
//! at s; else `dword_614B7C[0]`. Then `flt_614744` = engine / top x `flt_613F94` (0.25),
//! and `flt_614748` times +0x3F4 (`unk_71A3CC`, as a float).
//!
//! In:  nothing
//! Out: ebx i on the curve's path; the rest kept; no flags read
//!
//! Callers: `physics::engine_torque_or_cut`, `sub_421848`, `sub_43CAAB`, `sub_43EA80`.

use crate::math::x87;
use crate::math::x87_run::{div_then_mul, lerp_table, trunc_mul_to_int};
use crate::race::owner::race;
use crate::scene::owner::scene;
use std::cmp::Ordering;

/// C0 of comparing `a` with `b`: below, or unordered.
pub fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The Rust version (the exact one, for `physics::engine_torque_or_cut`, `drive_force`,
/// `free_rev*`, `test_drive_run`).
pub fn engine_torque() {
    let race = race();
    let shared_store = &race.shared;
    let drawing = &scene().drawing;
    let course = &race.course;
    let s = x87::div(
        shared_store.car.engine_speed.bits(),
        shared_store.car.engine_speed_limit.bits(),
    );
    drawing.scene.scratch.set(s);
    if below(s, course.numbers.one.bits()) && !below(s, course.numbers.zero.bits()) {
        let i = trunc_mul_to_int(s, course.numbers.eight.bits()) as u32;
        drawing.scene.work_dword.set(i);
        let k = i as usize;
        shared_store
            .physics_tables
            .engine_force
            .set_bits(lerp_table(
                s,
                shared_store.physics_tables.torque_curve[k].bits(),
                shared_store.physics_tables.torque_curve[k + 1].bits(),
                shared_store.physics_tables.spec_torque[k].get(),
                shared_store.physics_tables.spec_torque[k + 1].get(),
            ));
    } else {
        shared_store
            .physics_tables
            .engine_force
            .set_bits(x87::loaded(
                shared_store.physics_tables.spec_torque[0].get(),
            ));
    }
    shared_store
        .physics_tables
        .engine_torque
        .set_bits(div_then_mul(
            shared_store.car.engine_speed.bits(),
            shared_store.car.engine_speed_limit.bits(),
            shared_store.physics_tables.torque_ease.bits(),
        ));
    shared_store.physics_tables.engine_force.set_bits(x87::mul(
        shared_store.physics_tables.engine_force.bits(),
        shared_store.car.engine_power_factor.bits(),
    ));
}
