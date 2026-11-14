//! Draws glyphs 0x9C and 0x9D of the sprite table at the two points stored in a stage record
//! (`Record::markers`: the stage's two ends on the map; first point at +0x44/+0x48, second at
//! +0x4C/+0x50).

use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::owner::Gfx;
use crate::gfx::surface::clip;
use crate::menu::owner::Menus;
use crate::records::stages_store::Record;

const FIRST_GLYPH: u32 = 0x9C;
const SECOND_GLYPH: u32 = 0x9D;

pub fn sprite_table(menus: &Menus) -> u32 {
    menus.screens.loop_flags.sprite_table.get()
}

/// The two blitter calls made for `record`.
pub fn marker_calls(gfx: &Gfx, menus: &Menus, record: &Record) -> [BlitCall; 2] {
    let [(x0, y0), (x1, y1)] = record.markers();
    [
        BlitCall::sprite(menus, FIRST_GLYPH, x0, y0, clip(gfx).right),
        BlitCall::sprite(menus, SECOND_GLYPH, x1, y1, clip(gfx).right),
    ]
}
