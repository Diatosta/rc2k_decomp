//! `physics::wheels_from_revs` (0x43C124): in gear, `g_carDriveSpeed` = `g_carEngineSpeed`
//! times `g_gearRatios[g_carGear]`; with `g_carDriveKind` 1 `g_carDriveSpeed2` the same
//! (`fld; fstp` of it). Nothing in neutral.
//!
//! In:  nothing (the current car's work copy, `drive/car.rs`)
//! Out: ebx = `g_carGear` when not 0 (else kept); the rest kept
//!
//! See `physics/gearing.rs`. Callers (8): the gearbox under `sub_4454D2` (`sub_43B635`,
//! `sub_43C15C`, `sub_43C20D`, `sub_43C2AE`, `sub_43C5C2`, `sub_43C8AC`, `sub_43EA63`, and
//! unowned code at 0x43CA7B). Refuses a gear outside -1 ..= 7 (the ratio table), before
//! writing.

use crate::errors::Refused;
use crate::physics::gearing::*;

/// The Rust version (the exact one, for the gearbox's Rust callers), on the game's
/// variables.
pub fn wheels_from_revs() -> Result<(), Refused> {
    wheels_from_revs_at(&GearPlaces::game())
}
