//! `physics::damage_step` (0x4AB5D6): the car's damage and wear model for this physics
//! step: `physics::damage_factors`, `physics::damage_wear`, `physics::damage_gauges`
//! (its only code: three calls and `retn`).
//!
//! In:  nothing
//! Out: the registers as the three leave them (read by no caller); no flags read
//!
//! Caller: `sub_43B635` (0x43B952), every physics step. It begins with a call: hooked by
//! a private copy of its 0x10 bytes (the three calls relocated) and a hand-made patch.
//! Self-test: `physics/damage_model.rs` (`damage_test`).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

/// The Rust version (the registers the three leave are read by no caller).
pub fn damage_step() {
    crate::physics::damage_factors::steps();
    crate::physics::damage_wear::steps();
    crate::physics::damage_gauges::steps();
}
