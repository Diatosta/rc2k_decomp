//! `physics::damage_step` (0x4AB5D6): the car's damage and wear model for this physics
//! step: `physics::damage_factors`, `physics::damage_wear`, `physics::damage_gauges`
//! (its only code: three calls and `retn`).
//!
//! In:  nothing
//! Out: the registers as the three leave them; no flags read
//!
//! Caller: `sub_43B635` (0x43B952), every physics step. It begins with a call: hooked by
//! a private copy of its 0x10 bytes (the three calls relocated) and a hand-made patch.
//! Self-test: `physics/damage_model.rs` (`damage_test`).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;

/// The Rust version.
pub unsafe fn damage_step(entry: &Regs) -> Left {
    let mut r = *entry;
    crate::physics::damage_factors::steps(&mut r);
    crate::physics::damage_wear::steps(&mut r);
    crate::physics::damage_gauges::steps(&mut r);
    left_of(&r)
}
