//! The gearbox: `physics::gearbox_step` and what it runs, `physics::manual_gearbox`,
//! `physics::auto_gearbox`, `physics::pick_start_gear`. This file holds the steps they share,
//! on the work car's gear fields (`physics/gearing.rs`). The steps return only what the flow
//! uses (`physics::car_physics_step`, `physics::acceleration_run`).
//!
//! The shift points `LightsAndGear.gear.shift_up_point` .. `shift_ratio_figure`, `rev_limit`
//! and `idle_engine_speed` are set at run time (0 in the file); the tests set them.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::physics::car_store::Car;
use crate::physics::gear_bit::gear_bit;
use crate::physics::gearing::{revs_from_speed_at, revs_from_wheels_at, wheels_from_revs_at};
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use std::cmp::Ordering;

/// C0 of comparing `a` with `b`: below, or unordered (each a float or its bits).
pub fn below(a: impl crate::math::x87::Flt, b: impl crate::math::x87::Flt) -> bool {
    !matches!(
        a.value().partial_cmp(&b.value()),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The common end of a change: `drive_cut_steps` from the tables, the hold 20, both change words 1.
pub fn settle(car: &mut Car) {
    car.drive_cut_steps = crate::physics::figures::DRIVE_CUT_STEPS;
    car.gear_change_hold = rc2k_core::physics::gear_shifts::HOLD;
    car.gear_changed = 1;
    car.gear_change_pending = 1;
}

/// Neutral, then `settle`.
pub fn neutral(car: &mut Car) {
    car.gear = 0;
    settle(car);
}

/// `gear_bit`: true when the gear cannot be taken.
pub fn bit(car: &Car, gear: i32) -> Result<bool, Refused> {
    gear_bit(car, gear)
}

/// `wheels_from_revs`.
pub fn wheels(car: &mut Car, tables: &PhysicsTables) -> Result<(), Refused> {
    { wheels_from_revs_at(car, &tables.gear_ratios) }?;
    Ok(())
}

/// `revs_from_wheels`.
pub fn revs(car: &mut Car, tables: &PhysicsTables, numbers: &Numbers) -> Result<(), Refused> {
    { revs_from_wheels_at(numbers, car, &tables.gear_ratios) }?;
    Ok(())
}

/// `revs_from_speed`.
pub(super) fn revs_speed(car: &mut Car, tables: &PhysicsTables) -> Result<(), Refused> {
    { revs_from_speed_at(car, &tables.gear_ratios) }?;
    Ok(())
}

/// The engine speed over its top .
pub fn over_top(car: &Car) -> f32 {
    x87::div(car.engine_speed, car.engine_speed_limit)
}

/// A gear's ratio; refused outside -1 ..= 7.
pub fn ratio(tables: &PhysicsTables, gear: i32) -> Result<f32, Refused> {
    if !(-1..=7).contains(&gear) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    Ok(tables.gear_ratios[(gear + 1) as usize])
}
