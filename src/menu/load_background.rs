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

use crate::gfx::convert_picture::convert_picture_now_in;
use crate::gfx::load_picture::load_picture_with;
use crate::gfx::owner::Gfx;
use crate::gfx::picture_as_surface::picture_as_surface;
use crate::system::owner::System;
use crate::text::draw_text::Canvas;

// ---- game data ----

const FRAME_SPRITE: u32 = 0xA0;

pub fn load_background_in(
    path: &dyn crate::session::TextPlace,
    canvas: &mut dyn Canvas,
    menus: &crate::menu::owner::Menus,
    gfx: &Gfx,
    system: &System,
) -> Result<(), Refused> {
    // Whether the file was read is not looked at.
    load_picture_with(path, gfx, system)?;
    convert_picture_now_in(gfx, system)?;
    gfx.pictures
        .picture
        .flags
        .set(gfx.display.mode.mode_flags.get() & 0xFFF);
    picture_as_surface(gfx)?;
    let call = BlitCall::sprite(menus, FRAME_SPRITE, 0, 0, canvas.clip_right());
    { canvas.blit_glyph(call) }?;
    Ok(())
}
