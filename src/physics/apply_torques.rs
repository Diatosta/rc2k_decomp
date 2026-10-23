//! `physics::apply_torques` (0x43F6BF): ebx force records of 0x20 bytes from edi, each
//! by `physics::apply_torque` on the work car (`g_car`), inside `pusha` / `popa`.
//!
//! In:  edi the first record, ebx their number
//! Out: every register kept; no flags read
//!
//! Callers: `physics::apply_wheel_torques`, `physics::apply_contact_torques`.
//! Refuses (before anything) ebx 0 (the original would loop 2^32 times) or above 0x100.
//! Self-test: a copy of the original (its car the test's, its call to a copy of
//! `apply_torque`) on test records, counts 1 .. 7.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva};
use crate::physics::apply_torque::apply_torque;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const CAR_RVA: usize = rva(0x719FD8); // g_car

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
    let car = game_addr(CAR_RVA) as u32;
    for k in 0..count {
        unsafe { apply_torque(first + 0x20 * k, car) };
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { apply_torques(r.edi, r.ebx) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
