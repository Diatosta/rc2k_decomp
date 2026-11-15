//! `menu::load_background`: loads a menu screen's background picture and draws the frame sprite
//! onto it.
//!
//! The picture's path may be written to (see `files::swap_missing_extension`); the sprite is
//! drawn at (0, 0) with the palette given; whether the file was read is not looked at. Called
//! by nearly all the screen loops. Refuses what the functions it calls refuse.

use crate::errors::Refused;
use crate::gfx::blit_glyph::BlitCall;

use crate::gfx::convert_picture::convert_picture_now_in;
use crate::gfx::load_picture::load_picture_with;
use crate::gfx::owner::Gfx;
use crate::gfx::picture_as_surface::picture_as_surface;
use crate::system::owner::System;
use crate::text::draw_text::Canvas;

const FRAME_SPRITE: u32 = 0xA0;

pub fn load_background_in(
    path: &dyn crate::memory::session::TextPlace,
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
