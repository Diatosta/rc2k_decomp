//! `hud::draw_stage_bars_1` (0x4B5E29): the stage's progress bars of the split layout,
//! one row a player: as `hud::draw_stage_bar` with a frame of seven quads and each part
//! twice (rows 2 .. 0xE and 0x10 .. 0x1C), the second player's colours from
//! `dword_6D170C` (into `dword_75BFA8`; the first's into `dword_75BFA4`).
//! `hud::draw_stage_bars_2` (0x4B63C2) is the same 0x11E lower.
//!
//! In:  ebp the HUD's record
//! Out: ebx the parts drawn (kept with none), ecx 0; the rest kept
//!
//! Caller: layout 1's drawer (0x4B27F4). Refuses more than 0x40 parts and a colour offset past
//! `dword_75BF70`'s four, before anything.

use crate::errors::Refused;
use crate::hud::draw_stage_bar::*;
use crate::hud::owner::Hud;
use crate::race::owner::race;

/// The two players' bars `dy` lower (see the header).
pub fn bars_with(hud: &Hud, dy: u32) -> Result<(), Refused> {
    bars(
        hud,
        &[
            &race().frame.panel_and_overlay.overlay.overlay_part_kinds_a,
            &race().frame.panel_and_overlay.overlay.overlay_part_kinds_b,
        ],
        &[&hud.block.bar_colours[0], &hud.block.bar_colours[1]],
        dy,
    )
}

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::stage_bars` (through `draw_stage_bar::bars`)
pub fn draw_stage_bars_1(hud: &Hud) -> Result<(), Refused> {
    bars_with(hud, 0)
}
