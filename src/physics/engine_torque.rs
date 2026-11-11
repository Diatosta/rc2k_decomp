//! `physics::engine_torque` (0x43CBBB): the engine's torque from its speed. s = engine speed /
//! `engine_speed_limit`; for s in 0 ..= below 1 (unordered counts as out) the curve's entry
//! i = s x 8 (rounding set to chop), and `engine_force` is the line between (`torque_curve[i]`,
//! `spec_torque[i]`) and (`torque_curve[i + 1]`, `spec_torque[i + 1]`) at s; else `spec_torque[0]`.
//! Then `engine_torque` = engine speed / limit x `torque_ease` (0.25), and `engine_force` is
//! multiplied by `engine_power_factor` (as a float).
//!
//! Returns: i on the curve's path
//!
//! Callers: `physics::engine_torque_or_cut`, `physics::test_drive_run`, `physics::free_rev`.

use crate::math::x87;
use crate::math::x87::{lerp_table, trunc_mul_to_int};
use crate::math::x87_float as bh;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::race::owner::Race;
use std::cmp::Ordering;

/// C0 of comparing `a` with `b`: below, or unordered.
pub fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The exact version, for `physics::engine_torque_or_cut`, `drive_force`, `free_rev*`,
/// `test_drive_run`.
pub fn engine_torque(race: &Race, scene: &crate::scene::owner::Scene) {
    let shared_store = &race.shared;
    let drawing = &scene.drawing;
    let course = &race.course;
    let s = x87::div(
        &shared_store.car.engine_speed,
        &shared_store.car.engine_speed_limit,
    );
    drawing.scene.scratch.set(s);
    if below(s, course.numbers.one.bits()) && !below(s, ZERO) {
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
    shared_store.physics_tables.engine_torque.set_bits(bh::st(
        bh::ld(shared_store.car.engine_speed.bits())
            / bh::ld(shared_store.car.engine_speed_limit.bits())
            * bh::ld(shared_store.physics_tables.torque_ease.bits()),
    ));
    shared_store.physics_tables.engine_force.set_bits(x87::mul(
        &shared_store.physics_tables.engine_force,
        &shared_store.car.engine_power_factor,
    ));
}
