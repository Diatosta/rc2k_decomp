//! `physics::clear_contacts` (0x43F3D6): `physics::clear_records` on 7 records of 0x20 bytes at `unk_614A50` (by the look of
//! it the step's contacts, a reading).
//!
//! In:  nothing
//! Out: edi past the records, ecx 0; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B85B), every physics step.

use crate::errors::Refused;
use crate::physics::clear_records::clear_records;
use crate::physics::tables_store::PHYSICS;
const COUNT: u32 = 7;

/// The Rust version.
pub fn clear_contacts() -> Result<(), Refused> {
    clear_records((PHYSICS.contacts.as_ptr() as usize) as u32, COUNT)
}
