//! `race::car_number_textures` (0x46203D): car esi's marks' textures (lamps' sides, the
//! model's letter, its number's digits, the stage's plate, the 17 decal bytes); see
//! `race/car_draw.rs`.
//!
//! In:  esi = the car's block
//! Out: eax, ebx, ecx, edx, ebp as it leaves them (its caller saves eax, ecx, edi, esi)
//!
//! Caller: `sub_4616AA`. Shadow mode: by state. Self-test: a test block (a copy of car 0's
//! when there is one), +0x1B8 / +0x1BC, +0x718, +0xAA8, +0x310 and the stage varied.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::car_draw::car_number_textures;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { car_number_textures(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
