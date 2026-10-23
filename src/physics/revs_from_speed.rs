//! `physics::revs_from_speed` (0x43C101): in gear, `g_carEngineSpeed` = `g_carSpeed`
//! (0x71A184; the car's speed by its use in `sub_43CD79`, a reading) over
//! `g_gearRatios[g_carGear]`. Nothing in neutral.
//!
//! In:  nothing (the current car's work copy, `drive/car.rs`)
//! Out: ebx = `g_carGear` when not 0 (else kept); the rest kept
//!
//! See `physics/gearing.rs`. Callers (8): the gearbox under `sub_4454D2` (`sub_43C2AE`,
//! `sub_43C5C2`, `sub_43C8AC` three times, and unowned code at 0x43C986 ..).
//! Refuses a gear outside -1 ..= 7 (the ratio table), before writing.
//! Self-test: a copy of the original whose variables are the test's (they are the car's,
//! used all through a frame), against the converted version on the same: gears and speeds by lot
//! (the drive kinds too, which it does not read), exact at double precision (one correctly rounded
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { revs_from_speed_plain_at(&GearPlaces::game()) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    gearing_entry(r, &rust)
}
