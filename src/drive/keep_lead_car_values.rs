//! `drive::keep_lead_car_values` (0x4564F5): eight fields of car block 0 (0x70FA70 ..,
//! `drive/lead_car_values.rs`) into 0x64165C .. 0x641678 (`push`/`pop`). The marks read
//! `dword_64165C` (`marks::stage_record`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::share_all_cars` (0x456484), first.
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables; the game's never
//! written), 8 fillings.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::lead_car_values::{PAIRS, move_pairs};
use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn game() -> Vec<(u32, u32)> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    PAIRS.iter().map(|&(f, t)| (a(f), a(t))).collect()
}

/// The Rust version.
pub unsafe fn keep_lead_car_values() {
    unsafe { move_pairs(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { keep_lead_car_values() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
