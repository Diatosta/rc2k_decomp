//! `physics::revs_from_wheels` (0x43C0C9): in gear, `g_carEngineSpeed` = `g_carDriveSpeed`
//! over `g_gearRatios[g_carGear]`; with `g_carDriveKind` 1 the two driven speeds' sum
//! times 0.5 (`flt_5C5E40`) over it. Nothing in neutral.
//!
//! In:  nothing (the current car's work copy, `drive/car.rs`)
//!
//! See `physics/gearing.rs`. Callers (8): the gearbox under `drive::race_step` (0x43BF6F,
//! `physics::change_gear`, `physics::manual_gearbox`, `physics::auto_gearbox`,
//! `physics::drive_wheels_step`), `physics::test_drive_run` (the car specification's run, under
//! `drive::race_frame`), 0x428F42, 0x4291C9. Refuses a gear outside -1 ..= 7 (the ratio table),
//! before writing.

use crate::errors::Refused;
use crate::physics::gearing::*;
use crate::race::owner::Race;

/// The Rust version (the exact one, for the gearbox's Rust callers), on the game's
/// variables.
pub fn revs_from_wheels(race: &Race) -> Result<(), Refused> {
    revs_from_wheels_at(&race.course, &GearPlaces::game(&race.shared))
}
