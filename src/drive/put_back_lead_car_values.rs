//! `drive::put_back_lead_car_values` (0x456556): the eight values
//! `drive::keep_lead_car_values` kept (0x64165C ..) back into car block 0 (0x70FA70 ..,
//! `drive/lead_car_values.rs`; `push`/`pop`).
//!
//! In:  nothing
//! Out: every register kept; no flags read (`drive::share_all_cars` returns next)
//!
//! Caller: `drive::share_all_cars` (0x456484), last.
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables), 8 fillings.
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
    PAIRS.iter().map(|&(f, t)| (a(t), a(f))).collect()
}

/// The Rust version.
pub unsafe fn put_back_lead_car_values() {
    unsafe { move_pairs(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { put_back_lead_car_values() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
