//! `physics::wheels_from_revs` (0x43C124): in gear, `g_carDriveSpeed` = `g_carEngineSpeed`
//! times `g_gearRatios[g_carGear]`; with `g_carDriveKind` 1 `g_carDriveSpeed2` the same
//! (`fld; fstp` of it). Nothing in neutral.
//!
//! In:  nothing (the current car's work copy, `drive/car.rs`)
//! Out: ebx = `g_carGear` when not 0 (else kept); the rest kept
//!
//! See `physics/gearing.rs`. Callers (8): the gearbox under `sub_4454D2` (`sub_43B635`,
//! `sub_43C15C`, `sub_43C20D`, `sub_43C2AE`, `sub_43C5C2`, `sub_43C8AC`, `sub_43EA63`, and
//! unowned code at 0x43CA7B).
//! Refuses a gear outside -1 ..= 7 (the ratio table), before writing.
//! Self-test: a copy of the original whose variables are the test's (they are the car's,
//! used all through a frame), against the converted version on the same: gears, drive kinds and
//! speeds by lot, exact at double precision (one correctly rounded
//! product or quotient; at extended a sum of far-apart speeds may round otherwise);
//! and the refusal.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::gearing::*;
use crate::screens::harness::Left;

/// The Rust version (the exact one, for the gearbox's Rust callers), on the game's
/// variables.
pub unsafe fn wheels_from_revs() -> Result<Left, Refused> {
    unsafe { wheels_from_revs_at(&GearPlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { wheels_from_revs_plain_at(&GearPlaces::game()) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    gearing_entry(r, &rust)
}
