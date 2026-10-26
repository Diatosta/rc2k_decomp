//! `hud::draw_markers_2` (0x4B55EE): `hud::draw_markers_1`'s twin at rows 0x11F and
//! 0x12C (the lower half's).
//!
//! In:  ebp the HUD's record
//! Out: as `draw_markers_1`'s
//!
//! Caller: layout 2's drawer (0x4B403B). The sprites through `part` as
//! `draw_markers_1`'s. Shadow mode runs the original alone (it draws). Self-test:
//! `draw_markers_1::test_markers`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::hud::draw_markers_1::{Part, game_part, markers_with};

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::quads::{marker_x, int_over}` (through `markers_with`)
pub unsafe fn draw_markers_2(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { markers_with(entry, [0x11F, 0x12C], part) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_markers_2(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
