//! `race::add_car_depth_entry` (0x4574A0): a car's place (block esi +0x134 ..) through
//! `g_viewMatrix` into a depth entry (the key from its depth, the second word ebx +
//! 0x2000000), for the car's lights (see `race/frame_draw.rs`).
//!
//! In:  esi = the car's block, ebx = the car
//! Out: eax the place's z, ebx + 0x2000000, ecx the key, edx the entry, esi
//!      `g_viewMatrix` (its caller calls it inside `pusha`)
//!
//! Caller: `sub_455CFF` (0x455D22). Refuses (before anything) a depth entry past the
//! 0x1000. Shadow mode: by state. Self-test: a test block's places, the view matrix and
//! counts set; all put back; 0x1000 refused.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { add_car_depth_entry(r.esi, r.ebx) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
