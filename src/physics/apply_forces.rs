//! `physics::apply_forces` (0x43F566): ecx force records of 0x20 bytes from esi, each
//! by `physics::apply_force` on the work car (`g_car`), inside `pusha` / `popa`.
//!
//! In:  esi the first record, ecx their number
//! Out: every register kept; no flags read
//!
//! Callers: `physics::apply_wheel_forces`, `physics::apply_contact_forces`. Refuses (before
//! anything) ecx 0 (the original would loop 2^32 times) or above 0x100.

use crate::errors::Refused;
use crate::physics::apply_force::apply_force;
use crate::physics::tables_store::ForceRecord;

/// The Rust version: `records` (a whole table, the wheels' 0x17 or the contacts' 7) on the
/// work car.
pub fn apply_forces(records: &[ForceRecord]) -> Result<(), Refused> {
    for rec in records {
        apply_force(rec, &crate::physics::car_store::CAR);
    }
    Ok(())
}
