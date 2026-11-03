//! `hud::draw_stage_bars_2` (0x4B63C2): `hud::draw_stage_bars_1` 0x11E lower.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_stage_bars_1`'s
//!
//! Caller: layout 2's drawer (0x4B3C0B).

use crate::errors::Refused;
use crate::hud::draw_stage_bars_1::bars_with;
use crate::pointers::GamePtr;

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::stage_bars` (through `draw_stage_bar::bars`)
pub fn draw_stage_bars_2(record: GamePtr) -> Result<(), Refused> {
    bars_with(record, 0x11E)
}
