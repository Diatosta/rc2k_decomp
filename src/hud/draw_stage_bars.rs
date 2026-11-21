//! The stage's progress bars of the split layouts, one row a player: as
//! `hud::draw_stage_bar` with a frame of seven quads and each part twice (rows 2 .. 0xE and
//! 0x10 .. 0x1C), Layout 2 draws them `STAGE_BARS_DROP_2` lower.
//!
//! Refuses more than 0x40 parts and a colour offset past the four colours, before anything.

use crate::errors::Refused;
use crate::hud::draw_stage_bar::*;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::race::owner_frame::Frame;

/// How far layout 2's bars lie below layout 1's.
pub(super) const STAGE_BARS_DROP_2: u32 = 0x11E;

/// The two players' bars `dy` lower (see the header).
pub fn draw_stage_bars(frame: &Frame, hud: &mut Hud, pen: Pen, dy: u32) -> Result<(), Refused> {
    let (a, b) = {
        let overlay = frame.overlay();
        (overlay.overlay_part_kinds_a, overlay.overlay_part_kinds_b)
    };
    bars(frame, hud, pen, &[&a[..], &b[..]], dy)
}
