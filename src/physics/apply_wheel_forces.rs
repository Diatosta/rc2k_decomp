//! `physics::apply_wheel_forces` (0x43F544): `physics::apply_forces` on the 0x17 records at `unk_614770` (`physics::clear_forces`', the wheels' among them).
//!
//! In:  nothing
//! Out: esi the first record, ecx the count; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B81D), every physics step.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::physics::apply_forces::apply_forces;

const RECORDS_RVA: usize = rva(0x614770);
const COUNT: u32 = 0x17;

/// The Rust version.
pub unsafe fn apply_wheel_forces() -> Result<(), Refused> {
    let first = game_addr(RECORDS_RVA) as u32;
    unsafe { apply_forces(first, COUNT) }
}
