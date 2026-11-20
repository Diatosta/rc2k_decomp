//! The race's HUD: `hud::set_batch_depth`; the batch empty and without
//! a texture (`batch_texture` -1); the panel's times in tenths (`hud::tenths`); by `Panel::players`
//! `hud::layout_1` (1), `hud::layout_2` (2) or `hud::layout_0`; then `hud::flush_batch`.

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::race::owner_frame::Frame;

pub fn draw_hud(frame: &Frame, hud: &mut Hud, pen: Pen) -> Result<(), Refused> {
    use crate::hud::*;
    use rc2k_core::hud::batch;
    {
        set_batch_depth::set_batch_depth(hud, pen.vertices);
        hud.block.batch_texture = batch::NO_TEXTURE;
        hud.block.vertex_count = 0;
        hud.block.index_count = 0;
        tenths::tenths(frame, hud);
        use crate::hud::layout_common::{Keep, start_keep};
        match batch::Layout::of(frame.panel_and_overlay.get().panel.players.get()) {
            batch::Layout::Single => {
                start_keep(hud, Keep::Thirds);
                layout_0::layout_0(frame, hud, pen)?;
            }
            batch::Layout::One => {
                start_keep(hud, Keep::Halves);
                layout_1::layout_1(frame, hud, pen)?;
            }
            batch::Layout::Two => {
                start_keep(hud, Keep::Thirds);
                layout_2::layout_2(frame, hud, pen)?;
            }
            batch::Layout::Many(views) => layout_many::layout_many(frame, hud, pen, views)?,
        }
        flush_batch::flush_batch(pen.part, hud);
    }
    Ok(())
}
