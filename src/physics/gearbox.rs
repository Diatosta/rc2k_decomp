//! The gearbox (IDA 2026-10-18): `physics::gearbox_step` (0x43C15C) and what it runs,
//! `physics::manual_gearbox` (0x43C2AE, and its one-call twin 0x43C2A8),
//! `physics::auto_gearbox` (0x43C5C2), `physics::pick_start_gear` (0x43C8AC). This file:
//! the steps they share, on the game's variables (the work car's gear fields,
//! `physics/gearing.rs`). Plain since 2026-10-23 (phase 3): the registers the originals
//! leave are read by none of their callers (`physics::car_physics_step`,
//! `physics::test_drive_run`), so the steps return only what the flow uses.
//!
//! The shift points `flt_5CB958` .. `flt_5CB964`, `flt_614104` and `flt_6140F8` are set at
//! run time (0 in the file); the tests set them.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::physics::gear_bit::gear_bit;
use crate::physics::gearing::{
    GearPlaces, revs_from_speed_at, revs_from_wheels_at, wheels_from_revs_at,
};
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use std::cmp::Ordering;

/// C0 of comparing `a` with `b`: below, or unordered.
pub fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The common end of a change: +0x1D0 = `dword_613FA4`, the hold 20, both words 1.
pub fn settle(race: &Race) {
    race.shared
        .car
        .drive_cut_steps
        .set(race.shared.physics_tables.drive_cut_steps.get());
    race.shared
        .car
        .gear_change_hold
        .set(rc2k_core::physics::gear_shifts::HOLD);
    race.shared.car.gear_changed.set(1);
    race.shared.car.gear_changed_2.set(1);
}

/// Neutral, then `settle`.
pub fn neutral(race: &Race) {
    race.shared.car.gear.set(0);
    settle(race);
}

/// `gear_bit`: true for CF (the gear cannot be taken).
pub fn bit(shared: &Shared, gear: u32) -> Result<bool, Refused> {
    Ok({ gear_bit(shared, gear) }?.carry)
}

/// `wheels_from_revs` (its ebx, the gear, read by no caller).
pub fn wheels(shared: &Shared) -> Result<(), Refused> {
    { wheels_from_revs_at(&GearPlaces::game(shared)) }?;
    Ok(())
}

/// `revs_from_wheels` (its ebx read by no caller).
pub fn revs(race: &Race) -> Result<(), Refused> {
    { revs_from_wheels_at(&race.course, &GearPlaces::game(&race.shared)) }?;
    Ok(())
}

/// `revs_from_speed` (its ebx read by no caller).
pub fn revs_speed(shared: &Shared) -> Result<(), Refused> {
    { revs_from_speed_at(&GearPlaces::game(shared)) }?;
    Ok(())
}

/// The engine speed over its top (`fld; fdiv; fstp`).
pub fn over_top(race: &Race) -> u32 {
    x87::div(
        race.shared.car.engine_speed.bits(),
        race.shared.car.engine_speed_limit.bits(),
    )
}

/// A gear's ratio; refused outside -1 ..= 7.
pub fn ratio(shared: &Shared, gear: u32) -> Result<u32, Refused> {
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    Ok(shared.physics_tables.gear_ratios[(gear as i32 + 1) as usize].bits())
}
