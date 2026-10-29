//! `physics::apply_contact_forces` (0x43F555): `physics::apply_forces` on the 7 records at `unk_614A50` (`physics::clear_contacts`').
//!
//! In:  nothing
//! Out: esi the first record, ecx the count; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B883), every physics step.

use crate::errors::Refused;
use crate::physics::apply_forces::apply_forces;
use crate::physics::tables_store::PHYSICS;
const COUNT: usize = 7;

/// The Rust version.
pub fn apply_contact_forces() -> Result<(), Refused> {
    apply_forces(&PHYSICS.contacts[..COUNT])
}
