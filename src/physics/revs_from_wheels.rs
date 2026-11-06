//! `physics::revs_from_wheels` (0x43C0C9): in gear, `g_carEngineSpeed` = `g_carDriveSpeed`
//! over `g_gearRatios[g_carGear]`; with `g_carDriveKind` 1 the two driven speeds' sum
//! times 0.5 (`flt_5C5E40`) over it. Nothing in neutral.
//!
//! In:  nothing (the current car's work copy, `drive/car.rs`)
//! Out: ebx = `g_carGear` when not 0 (else kept); the rest kept
//!
//! See `physics/gearing.rs`. Callers (8): the gearbox under `sub_4454D2` (`sub_43BF6F`,
//! `sub_43C20D`, `sub_43C2AE`, `sub_43C5C2`, `sub_43DFB1`), `sub_421848` (the car
//! specification's run, under `sub_4541BF`), `sub_428F42`, `sub_4291C9`. Refuses a gear
//! outside -1 ..= 7 (the ratio table), before writing.

use crate::errors::Refused;
use crate::physics::gearing::*;

/// The Rust version (the exact one, for the gearbox's Rust callers), on the game's
/// variables.
pub fn revs_from_wheels() -> Result<(), Refused> {
    let course = &crate::race::owner::race().course;
    revs_from_wheels_at(course, &GearPlaces::game())
}
