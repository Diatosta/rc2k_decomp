//! `physics::wheels_from_revs` (0x43C124): in gear, `g_carDriveSpeed` = `g_carEngineSpeed`
//! times `g_gearRatios[g_carGear]`; with `g_carDriveKind` 1 `g_carDriveSpeed2` the same
//! (`fld; fstp` of it). Nothing in neutral.
//!
//! In:  nothing (the current car's work copy, `drive/car.rs`)
//!
//! See `physics/gearing.rs`. Callers (8): the gearbox under `drive::race_step` (0x43B635,
//! `physics::gearbox_step`, `physics::change_gear`, `physics::manual_gearbox`,
//! `physics::auto_gearbox`, `physics::pick_start_gear`, `physics::free_rev_clutch`, and unowned
//! code at 0x43CA7B). Refuses a gear outside -1 ..= 7 (the ratio table), before writing.

use crate::errors::Refused;
use crate::physics::gearing::*;
use crate::race::owner_shared::RaceShared;

/// The exact version, for the gearbox's Rust callers, on the game's variables.
pub fn wheels_from_revs(shared: &RaceShared) -> Result<(), Refused> {
    wheels_from_revs_at(&GearPlaces::game(shared))
}
