//! `physics::clear_forces` (0x43F395): `physics::clear_records` on 0x17 records of 0x20 bytes at `unk_614770` (by the look of
//! it the step's forces, a reading).
//!
//! In:  nothing
//! Out: edi past the records, ecx 0; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B71E), every physics step.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::physics::clear_records::clear_records;

const RECORDS_RVA: usize = rva(0x614770);
const COUNT: u32 = 0x17;

/// The Rust version.
pub unsafe fn clear_forces() -> Result<(), Refused> {
    unsafe { clear_records(game_addr(RECORDS_RVA) as u32, COUNT) }
}
