//! `hud::draw_stage_bars_2` (0x4B63C2): `hud::draw_stage_bars_1` 0x11E lower.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_stage_bars_1`'s
//!
//! Caller: layout 2's drawer (0x4B3C0B). The quads through `part` as
//! `draw_stage_bars_1`'s. Shadow mode runs the original alone (it draws). Self-test:
//! `draw_stage_bars_1::test_bars`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::hud::draw_stage_bar::{Part, game_part};
use crate::hud::draw_stage_bars_1::bars_with;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::stage_bars` (through `draw_stage_bar::bars`)
pub unsafe fn draw_stage_bars_2(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { bars_with(entry, 0x11E, part) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_stage_bars_2(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
