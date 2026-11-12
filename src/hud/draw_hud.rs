//! `hud::draw_hud` (0x4B1CD0): the race's HUD: `hud::set_batch_depth`; the batch empty and without
//! a texture (`batch_texture` -1); the panel's times in tenths (`hud::tenths`); by `Panel::players`
//! `hud::layout_1` (1), `hud::layout_2` (2) or `hud::layout_0`; then `hud::flush_batch`.
//!
//! Caller: `hud::overlay_pass` (0x45A8F0).

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::race::owner_frame::Frame;

// Converted: `rc2k_core::hud::batch::{Layout, NO_TEXTURE}`
pub fn draw_hud(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    use crate::hud::*;
    use rc2k_core::hud::batch;
    {
        set_batch_depth::set_batch_depth(hud, pen.vertices);
        hud.block.batch_texture.set(batch::NO_TEXTURE);
        hud.block.vertex_count.set(0);
        hud.block.index_count.set(0);
        tenths::tenths(frame, hud);
        match batch::Layout::of(frame.panel_and_overlay.get().panel.players.get()) {
            batch::Layout::Single => layout_0::layout_0(frame, hud, pen)?,
            batch::Layout::One => layout_1::layout_1(frame, hud, pen)?,
            batch::Layout::Two => layout_2::layout_2(frame, hud, pen)?,
        }
        flush_batch::flush_batch(pen.part, hud);
    }
    Ok(())
}
