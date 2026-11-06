//! `hud::draw_stage_bars_2` (0x4B63C2): `hud::draw_stage_bars_1` 0x11E lower.
//!
//! In:  (the HUD's record is the panel)
//! Out: as `draw_stage_bars_1`'s
//!
//! Caller: layout 2's drawer (0x4B3C0B).

use crate::errors::Refused;
use crate::hud::draw_stage_bars_1::bars_with;
use crate::hud::owner::Hud;
use crate::race::owner_frame::Frame;

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::stage_bars` (through `draw_stage_bar::bars`)
pub fn draw_stage_bars_2(frame: &Frame, hud: &Hud) -> Result<(), Refused> {
    bars_with(frame, hud, 0x11E)
}
