//! `physics::apply_torques` (0x43F6BF): ebx force records of 0x20 bytes from edi, each
//! by `physics::apply_torque` on the work car (`g_car`), inside `pusha` / `popa`.
//!
//! In:  edi the first record, ebx their number
//! Out: every register kept; no flags read
//!
//! Callers: `physics::apply_wheel_torques`, `physics::apply_contact_torques`. Refuses
//! (before anything) ebx 0 (the original would loop 2^32 times) or above 0x100.

use crate::errors::{BadInput, Refused};
use crate::physics::apply_torque::apply_torque;

/// Refused before anything?
// Converted: `rc2k_core::physics::pushes::record_count`.
pub fn refusal(count: u32) -> Option<&'static str> {
    rc2k_core::physics::pushes::record_count(count, "a torque record count of 0 or above 0x100")
        .err()
        .map(|e| e.0)
}

/// The Rust version: `count` records from `first` on the work car.
pub unsafe fn apply_torques(first: u32, count: u32) -> Result<(), Refused> {
    if let Some(why) = refusal(count) {
        return Err(BadInput(why).into());
    }
    let car = crate::physics::car_store::car_addr();
    for k in 0..count {
        unsafe { apply_torque(first + 0x20 * k, car) };
    }
    Ok(())
}
