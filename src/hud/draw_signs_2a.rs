//! `hud::draw_signs_2a` (0x4B507F): the first player's signs in layout 2: as `hud::draw_signs` from the record's +0xA4 ..,
//! the places from +0xB4, at row 0x8 (no lift).
//!
//! In:  ebp the HUD's record
//! Out: as `draw_signs`'
//!
//! Caller: layout 2's drawer. The sprites through `part` (1), in the game `add_sprite`'s
//! Rust version. Shadow mode runs the original alone (it draws). Self-test:
//! `draw_signs::test_signs`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::hud::draw_signs::signs_with;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::signs` (through `signs_with`)
pub unsafe fn draw_signs_2a(entry: &Regs) -> Result<Regs, Refused> {
    unsafe { signs_with(entry, 0xA4, 0xB4, 0, 0x8, 0x0) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_signs_2a(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
