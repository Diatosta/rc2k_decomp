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
use crate::hud::draw_stage_bars_1::bars_with;

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::stage_bars` (through `draw_stage_bar::bars`)
pub unsafe fn draw_stage_bars_2(record: u32) -> Result<(), Refused> {
    unsafe { bars_with(record, 0x11E) }
}
