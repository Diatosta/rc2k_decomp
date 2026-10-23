//! `race::draw_car_entry` (0x455105): car ebx & 7's place turned by the view; drawn
//! (`race::draw_car_shined`) unless behind or aside, each counted (see
//! `race/car_draw.rs`).
//!
//! In:  ebx = the car (low three bits)
//! Out: ebx the car, esi its block, eax as it leaves it, then as the drawing leaves them
//!      (its caller `race::draw_list_entry` keeps all)
//!
//! Refuses (before anything) a car without its block. Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose drawing call goes to a stand-in, against the Rust
//! version with the same: an identity view, cars placed ahead, behind and aside; all put
//! back (skipped with a null block).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::car_draw::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_car_entry(r) }?;
    Ok(())
}
