//! `hud::draw_signs_1b` (0x4B5451): the second player's signs in layout 1: as `hud::draw_signs` from the record's +0xB8 ..,
//! the places from +0xC8 plus 0xC8, at row 0x70 (0x20 higher with `dword_6D1502` 0).
//!
//! In:  ebp the HUD's record
//! Out: as `draw_signs`'
//!
//! Caller: layout 1's drawer. The sprites through `part` (1), in the game `add_sprite`'s
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
pub unsafe fn draw_signs_1b(entry: &Regs) -> Result<Regs, Refused> {
    unsafe { signs_with(entry, 0xB8, 0xC8, 0xC8, 0x70, 0x20) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_signs_1b(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
