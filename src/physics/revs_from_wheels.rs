//! `physics::revs_from_wheels` (0x43C0C9): in gear, `g_carEngineSpeed` = `g_carDriveSpeed`
//! over `g_gearRatios[g_carGear]`; with `g_carDriveKind` 1 the two driven speeds' sum
//! times 0.5 (`flt_5C5E40`) over it. Nothing in neutral.
//!
//! In:  nothing (the current car's work copy, `drive/car.rs`)
//! Out: ebx = `g_carGear` when not 0 (else kept); the rest kept
//!
//! See `physics/gearing.rs`. Callers (8): the gearbox under `sub_4454D2` (`sub_43BF6F`,
//! `sub_43C20D`, `sub_43C2AE`, `sub_43C5C2`, `sub_43DFB1`), `sub_421848` (the car
//! specification's run, under `sub_4541BF`), `sub_428F42`, `sub_4291C9`.
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
use crate::physics::gearing::*;
use crate::screens::harness::Left;

/// The Rust version (the exact one, for the gearbox's Rust callers), on the game's
/// variables.
pub unsafe fn revs_from_wheels() -> Result<Left, Refused> {
    unsafe { revs_from_wheels_at(&GearPlaces::game()) }
}
