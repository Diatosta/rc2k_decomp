//! `physics::apply_torques` (0x43F6BF): ebx force records of 0x20 bytes from edi, each
//! by `physics::apply_torque` on the work car (`g_car`), inside `pusha` / `popa`.
//!
//! In:  edi the first record, ebx their number
//! Out: every register kept; no flags read
//!
//! Callers: `physics::apply_wheel_torques`, `physics::apply_contact_torques`. Refuses
//! (before anything) ebx 0 (the original would loop 2^32 times) or above 0x100.

use crate::errors::Refused;
use crate::physics::apply_torque::apply_torque;
use crate::physics::tables_store::ForceRecord;

/// The Rust version: `records` (a whole table, the wheels' 0x17 or the contacts' 7) on the
/// work car.
pub fn apply_torques(records: &[ForceRecord]) -> Result<(), Refused> {
    for rec in records {
        apply_torque(rec, &crate::physics::car_store::CAR);
    }
    Ok(())
}
