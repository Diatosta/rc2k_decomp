//! `menu::load_background` (0x47EFFF): loads a menu screen's background picture and draws
//! the frame sprite onto it.
//!
//! The picture's path may be written to (see `files::swap_missing_extension`); the sprite is
//! drawn with the palette given. The original's callers test the carry flag the blitter leaves
//! (and call a set carry fatal error 0x80712345), but it is no result of the loading.
//! `gfx::load_picture`, whose result is not looked at; `gfx::convert_picture`;
//! `g_pictureFlags` = the screen mode's flags (the screen mode flags) and 0xFFF;
//! `gfx::picture_as_surface`; sprite 0xA0 of `g_spriteTable` at (0, 0). 76 callers, nearly all of
//! them screen loops. Refuses what the functions it calls refuse. Rust callers run its `entry`
//! (`routes.rs`).

use crate::errors::Refused;
use crate::gfx::blit_glyph::BlitCall;

use crate::gfx::convert_picture::convert_picture_now;
use crate::gfx::load_picture::load_picture;
use crate::gfx::owner::gfx;
use crate::gfx::picture_as_surface::picture_as_surface;
use crate::text::draw_text::Canvas;

// ---- game data ----

const FRAME_SPRITE: u32 = 0xA0;

pub fn load_background(
    path: &dyn crate::session::TextPlace,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let gfx = gfx();
    // Whether the file was read is not looked at.
    load_picture(path)?;
    convert_picture_now()?;
    gfx.pictures
        .picture
        .flags
        .set(gfx.display.mode.mode_flags.get() & 0xFFF);
    picture_as_surface(gfx)?;
    { canvas.blit_glyph(BlitCall::sprite(FRAME_SPRITE, 0, 0)) }?;
    Ok(())
}
