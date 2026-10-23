//! `physics::apply_forces` (0x43F566): ecx force records of 0x20 bytes from esi, each
//! by `physics::apply_force` on the work car (`g_car`), inside `pusha` / `popa`.
//!
//! In:  esi the first record, ecx their number
//! Out: every register kept; no flags read
//!
//! Callers: `physics::apply_wheel_forces`, `physics::apply_contact_forces`.
//! Refuses (before anything) ecx 0 (the original would loop 2^32 times) or above 0x100.
//! Self-test: a copy of the original (its car the test's, its call to a copy of
//! `apply_force`) on test records, counts 1 .. 7.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva};
use crate::physics::apply_force::apply_force;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

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
    let car = game_addr(rva(0x719FD8)) as u32;
    for k in 0..count {
        unsafe { apply_force(first + 0x20 * k, car) };
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { apply_forces(r.esi, r.ecx) }?;
    Ok(Left::nothing())
}

/// What `apply_force` writes on the work car.
pub fn regions() -> Vec<crate::service::harness::Region> {
    vec![
        (game_addr(rva(0x614B30)) as u32, 12),
        (game_addr(rva(0x62400C)) as u32, 4),
        (game_addr(rva(0x719FD8 + 0x198)) as u32, 0x18),
    ]
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
