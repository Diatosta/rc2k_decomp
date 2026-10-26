//! `physics::apply_forces` (0x43F566): ecx force records of 0x20 bytes from esi, each
//! by `physics::apply_force` on the work car (`g_car`), inside `pusha` / `popa`.
//!
//! In:  esi the first record, ecx their number
//! Out: every register kept; no flags read
//!
//! Callers: `physics::apply_wheel_forces`, `physics::apply_contact_forces`. Refuses (before
//! anything) ecx 0 (the original would loop 2^32 times) or above 0x100.

use crate::errors::{BadInput, Refused};
use crate::physics::apply_force::apply_force;

/// Refused before anything?
// Converted: `rc2k_core::physics::pushes::record_count`.
pub fn refusal(count: u32) -> Option<&'static str> {
    rc2k_core::physics::pushes::record_count(count, "a force record count of 0 or above 0x100")
        .err()
        .map(|e| e.0)
}

/// The Rust version: `count` records from `first` on the work car.
pub unsafe fn apply_forces(first: u32, count: u32) -> Result<(), Refused> {
    if let Some(why) = refusal(count) {
        return Err(BadInput(why).into());
    }
    let car = crate::physics::car_store::car_addr();
    for k in 0..count {
        unsafe { apply_force(first + 0x20 * k, car) };
    }
    Ok(())
}
