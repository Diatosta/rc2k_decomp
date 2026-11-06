//! `gfx::blit_marker_pair` (0x46D098): draws glyphs 0x9C and 0x9D of the sprite table at the
//! two points stored in a stage record (`Record::markers`: the stage's two ends on the map).
//!
//! In:  edi = the record (first point at +0x44/+0x48, second at +0x4C/+0x50), ebx = palette
//! Out: every register preserved
//! Each point is also stored in the globals `g_markerX`/`g_markerY`, which are left holding
//! the second one.

use crate::gfx::blit_glyph::BlitCall;
use crate::records::stages_store::Record;

// ---- game data ----

const FIRST_GLYPH: u32 = 0x9C;
const SECOND_GLYPH: u32 = 0x9D;

pub fn sprite_table() -> u32 {
    crate::menu::owner::menus()
        .screens
        .loop_flags
        .sprite_table
        .get()
}

// ---- Rust version ----

/// The two blitter calls the original makes for `record`.
pub fn marker_calls(record: &Record) -> [BlitCall; 2] {
    let [(x0, y0), (x1, y1)] = record.markers();
    [
        BlitCall::sprite(FIRST_GLYPH, x0, y0),
        BlitCall::sprite(SECOND_GLYPH, x1, y1),
    ]
}

/// Stores a marker's position in the two globals, as the original does before each blit.
pub fn set_marker_position(call: &BlitCall) {
    crate::menu::owner::menus()
        .screens
        .draw_work
        .marker_x
        .set(call.x as u32);
    crate::menu::owner::menus()
        .screens
        .draw_work
        .marker_y
        .set(call.y as u32);
}
