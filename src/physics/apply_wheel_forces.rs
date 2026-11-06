//! `physics::apply_wheel_forces` (0x43F544): `physics::apply_forces` on the 0x17 records at `unk_614770` (`physics::clear_forces`', the wheels' among them).
//!
//! In:  nothing
//! Out: esi the first record, ecx the count; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B81D), every physics step.

use crate::errors::Refused;
use crate::physics::apply_forces::apply_forces;
use crate::race::owner_shared::Shared;
const COUNT: usize = 0x17;

/// The Rust version.
pub fn apply_wheel_forces(shared: &Shared) -> Result<(), Refused> {
    apply_forces(shared, &shared.physics_tables.forces[..COUNT])
}
