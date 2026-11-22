//! The engine's torque from its speed. s = engine speed / `engine_speed_limit`; for s in 0 ..<
//! 1 (unordered counts as out) the curve's entry i = s x 8 (rounded down), and `engine_force`
//! is the line between (`torque_curve[i]`, `spec_torque[i]`) and (`torque_curve[i + 1]`,
//! `spec_torque[i + 1]`) at s; otherwise `spec_torque[0]`. Then `engine_torque` = engine speed
//! / limit x `torque_ease` (0.25), and `engine_force` is multiplied by `engine_power_factor`
//! (as a float).
//!
//! Returns i on the curve's path.

use crate::math::cell_x87::{CellX87, PlainX87};
use crate::math::x87;
use crate::math::x87::{lerp_table, trunc_mul_to_int};
use crate::physics::figures::TORQUE_EASE;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;
use std::cmp::Ordering;

/// C0 of comparing `a` with `b`: below, or unordered (each a float or its bits).
pub fn below(a: impl crate::math::x87::Flt, b: impl crate::math::x87::Flt) -> bool {
    !matches!(
        a.value().partial_cmp(&b.value()),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The exact version, for `physics::engine_torque_or_cut`, `drive_force`, `free_rev*`,
/// `acceleration_run`.
pub fn engine_torque(course: &Course, shared: &RaceShared, scene: &crate::scene::owner::Scene) {
    let shared_store = shared;
    let s = x87::div(
        &shared_store.car().engine_speed,
        &shared_store.car().engine_speed_limit,
    );
    if below(s, course.numbers.one.get()) && !below(s, ZERO) {
        let i = trunc_mul_to_int(s, course.numbers.eight.get()) as u32;
        scene.draw_work_mut().work_dword = i;
        let k = i as usize;
        let force = lerp_table(
            s,
            crate::physics::figures::TORQUE_CURVE[k],
            crate::physics::figures::TORQUE_CURVE[k + 1],
            shared_store.physics_tables().spec_torque[k],
            shared_store.physics_tables().spec_torque[k + 1],
        );
        shared_store.physics_tables_mut().engine_force = force;
    } else {
        let force = x87::loaded(shared_store.physics_tables().spec_torque[0]);
        shared_store.physics_tables_mut().engine_force = force;
    }
    shared_store
        .physics_tables_mut()
        .engine_torque
        .st(
            shared_store.car().engine_speed.ld() / shared_store.car().engine_speed_limit.ld()
                * crate::math::x87_float::ld(TORQUE_EASE),
        );
    let force = x87::mul(
        shared_store.physics_tables().engine_force,
        &shared_store.car().engine_power_factor,
    );
    shared_store.physics_tables_mut().engine_force = force;
}
