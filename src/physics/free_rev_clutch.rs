//! `physics::free_rev_clutch` (0x43EA63): in gear with the clutch out (+0x250,
//! `unk_71A228`): `physics::free_rev`, then `physics::wheels_from_revs`.
//!
//! In:  nothing
//! Out: ebx the gear (`wheels_from_revs`), and what `free_rev` leaves; no flags read
//!
//! Caller: `sub_43B635` (0x43B7B7), every physics step.
//! Refuses (before anything) a gear outside -1 ..= 7.
//! Self-test: `physics::engine_torque`'s (`test_engine_floats`), the converted version,
//! with the clutch word set.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::physics::engine_torque::{engine_entry, g};
use crate::physics::free_rev::{free_rev, free_rev_plain};
use crate::physics::gearing::{GearPlaces, wheels_from_revs_at, wheels_from_revs_plain_at};
use crate::screens::harness::Left;

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn free_rev_clutch(entry: &Regs) -> Result<Left, Refused> {
    let gear = g(0x71A1AC);
    if gear == 0 || g(0x71A228) == 0 {
        return Ok(Left::nothing());
    }
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    let mut r = *entry;
    unsafe {
        free_rev().apply(&mut r);
        wheels_from_revs_at(&GearPlaces::game())?.apply(&mut r);
    }
    Ok(crate::race::stage_prefs::left_of(&r))
}

/// The converted version (`physics::free_rev_plain`, `wheels_from_revs_plain_at`): the
/// hook's.
// Converted: `rc2k_core::physics::{engine::free_rev, gearing::wheels_from_revs}`.
pub unsafe fn free_rev_clutch_plain(entry: &Regs) -> Result<Left, Refused> {
    let gear = g(0x71A1AC);
    if gear == 0 || g(0x71A228) == 0 {
        return Ok(Left::nothing());
    }
    rc2k_core::physics::gearing::ratio_index(gear)?;
    let mut r = *entry;
    unsafe {
        free_rev_plain().apply(&mut r);
        wheels_from_revs_plain_at(&GearPlaces::game())?.apply(&mut r);
    }
    Ok(crate::race::stage_prefs::left_of(&r))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { free_rev_clutch_plain(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    engine_entry(r, &rust)
}
