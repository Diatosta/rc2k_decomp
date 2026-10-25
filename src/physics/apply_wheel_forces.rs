//! `physics::apply_wheel_forces` (0x43F544): `physics::apply_forces` on the 0x17 records at `unk_614770` (`physics::clear_forces`', the wheels' among them).
//!
//! In:  nothing
//! Out: esi the first record, ecx the count; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B81D), every physics step.

use crate::errors::Refused;
use crate::physics::apply_forces::apply_forces;
use crate::physics::tables_store::PHYSICS;
const COUNT: u32 = 0x17;

/// The Rust version.
pub unsafe fn apply_wheel_forces() -> Result<(), Refused> {
    let first = (PHYSICS.forces.as_ptr() as usize) as u32;
    unsafe { apply_forces(first, COUNT) }
}
