//! The stage's progress bars of the split layouts, one row a player: as
//! `hud::draw_stage_bar` with a frame of seven quads and each part twice (rows 2 .. 0xE and
//! 0x10 .. 0x1C), the second player's colours from the second bar-colour cell, the
//! first's from the first. Layout 2 draws them `STAGE_BARS_DROP_2` lower.
//!
//! Refuses more than 0x40 parts and a colour offset past the four colours, before anything.

use crate::errors::Refused;
use crate::hud::draw_stage_bar::*;
use crate::hud::owner::{Hud, Pen};
use crate::race::owner_frame::Frame;

/// How far layout 2's bars lie below layout 1's.
pub const STAGE_BARS_DROP_2: u32 = 0x11E;

/// The two players' bars `dy` lower (see the header).
pub fn draw_stage_bars(frame: &Frame, hud: &Hud, pen: Pen, dy: u32) -> Result<(), Refused> {
    bars(
        frame,
        hud,
        pen,
        &[
            &frame.panel_and_overlay.get().overlay.overlay_part_kinds_a,
            &frame.panel_and_overlay.get().overlay.overlay_part_kinds_b,
        ],
        &[&hud.block.bar_colours[0], &hud.block.bar_colours[1]],
        dy,
    )
}
