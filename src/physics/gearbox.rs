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
use crate::framework::{game_read, game_write, rva};
use crate::math::x87;
use crate::physics::gear_bit::gear_bit;
use crate::physics::gearing::{
    GearPlaces, revs_from_speed_at, revs_from_wheels_at, wheels_from_revs_at,
};
use std::cmp::Ordering;

pub const GEAR: usize = 0x71A1AC; // g_carGear
pub const ENGINE: usize = 0x71A1B0; // g_carEngineSpeed
pub const LIMIT: usize = 0x71A1B4; // g_car + 0x1DC: the engine's top
pub const SPEED: usize = 0x71A184; // g_carSpeed
pub const HOLD: usize = 0x71A1A4; // g_car + 0x1CC: frames before the next change
pub const CHANGED: usize = 0x71A210; // g_car + 0x238: 1 changed, -1 refused
pub const CHANGED_2: usize = 0x71A214; // g_car + 0x23C
pub const GEAR_COUNT: usize = 0x614C10; // g_gearCount
pub const RATIOS: usize = 0x6140C0; // g_gearRatios (gear -1 at 0x6140BC)
pub const SCRATCH: usize = 0x62400C;

pub fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

pub fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// C0 of comparing `a` with `b`: below, or unordered.
pub fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The common end of a change: +0x1D0 = `dword_613FA4`, the hold 20, both words 1.
pub fn settle() {
    p(0x71A1A8, g(0x613FA4));
    p(HOLD, rc2k_core::physics::gear_shifts::HOLD);
    p(CHANGED, 1);
    p(CHANGED_2, 1);
}

/// Neutral, then `settle`.
pub fn neutral() {
    p(GEAR, 0);
    settle();
}

/// `gear_bit`: true for CF (the gear cannot be taken).
pub fn bit(gear: u32) -> Result<bool, Refused> {
    Ok(unsafe { gear_bit(gear) }?.carry)
}

/// `wheels_from_revs` (its ebx, the gear, read by no caller).
pub fn wheels() -> Result<(), Refused> {
    unsafe { wheels_from_revs_at(&GearPlaces::game()) }?;
    Ok(())
}

/// `revs_from_wheels` (its ebx read by no caller).
pub fn revs() -> Result<(), Refused> {
    unsafe { revs_from_wheels_at(&GearPlaces::game()) }?;
    Ok(())
}

/// `revs_from_speed` (its ebx read by no caller).
pub fn revs_speed() -> Result<(), Refused> {
    unsafe { revs_from_speed_at(&GearPlaces::game()) }?;
    Ok(())
}

/// The engine speed over its top (`fld; fdiv; fstp`).
pub fn over_top() -> u32 {
    x87::div(g(ENGINE), g(LIMIT))
}

/// A gear's ratio; refused outside -1 ..= 7.
pub fn ratio(gear: u32) -> Result<u32, Refused> {
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    Ok(g(RATIOS.wrapping_add(4 * gear as usize)))
}
