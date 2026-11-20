//! The network screens' background (`chatzone.pcx`) with the screen's title drawn on it,
//! shown.

use crate::errors::Refused;
use crate::net::lobby::{copy_picture, draw_string};

pub const TITLE_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::HEAD;
pub fn show_background(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (gfx, system, net) = (o.gfx, o.system, o.net);
    crate::gfx::load_picture::load_picture_with(&crate::gfx::texts::CHATZONE, gfx, system)?;
    crate::gfx::convert_picture::convert_picture_now_in(gfx)?;
    gfx.pictures
        .picture
        .flags
        .set(gfx.display.mode.mode_flags.get() & 0xFFF);
    crate::gfx::picture_as_surface::picture_as_surface(gfx)?;
    let title = if net.race_slot.is_on() {
        0x15A
    } else {
        match net.lobby().mode.get() {
            0 => 0x162,
            1 => 0x14B,
            _ => 0x14C,
        }
    };
    draw_string(
        crate::strings::append_string_by_id::StringOwners::of(o),
        title,
        TITLE_FONT,
        0x316,
        0x2F,
        gfx,
    )?;
    copy_picture(gfx, system)?;
    crate::shell::frame::show_frame_now(o);
    Ok(())
}
