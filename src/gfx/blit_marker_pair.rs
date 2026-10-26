//! `gfx::blit_marker_pair` (0x46D098): draws glyphs 0x9C and 0x9D of the sprite table at the
//! two points stored in an object.
//!
//! In:  edi = object (first point at +0x44/+0x48, second at +0x4C/+0x50), ebx = palette
//! Out: every register preserved
//! Each point is also stored in the globals `g_markerX`/`g_markerY`, which are left holding
//! the second one. What the object and the two glyphs are is not known yet.

use crate::gfx::blit_glyph::BlitCall;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::loop_store::LOOP_FLAGS;

// ---- game data ----

const FIRST_GLYPH: u32 = 0x9C;
const SECOND_GLYPH: u32 = 0x9D;

pub fn sprite_table() -> u32 {
    LOOP_FLAGS.sprite_table.get()
}

// ---- Rust version ----

/// The two blitter calls the original makes for `object`.
pub fn marker_calls(
    object: u32,
    palette: u32,
    table: u32,
) -> Result<[BlitCall; 2], crate::errors::BadInput> {
    let object = crate::pointers::GamePtr::new(object);
    let coord = |at: u32| object.offset(at).read::<i32>("a marker object\'s position");
    Ok([
        BlitCall::new(FIRST_GLYPH, coord(0x44)?, coord(0x48)?, palette, table),
        BlitCall::new(SECOND_GLYPH, coord(0x4C)?, coord(0x50)?, palette, table),
    ])
}

/// Stores a marker's position in the two globals, as the original does before each blit.
pub fn set_marker_position(call: &BlitCall) {
    DRAW_WORK.marker_x.set(call.x as u32);
    DRAW_WORK.marker_y.set(call.y as u32);
}
