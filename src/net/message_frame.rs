//! The frame of `net::show_message`: the background with the message drawn.

use crate::errors::Refused;
use crate::net::lobby::{copy_picture, draw_string, lock_surface};

pub const FONT_B: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::CAPTION_B;

/// A frame with one string: locked, the background, the string, unlocked, shown; nothing if
/// the lock fails.
pub fn string_frame(
    o: crate::owners::Owners<'static>,
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    if !lock_surface(o.gfx, o.system) {
        return Ok(());
    }
    string_frame_locked(o, id, font, x, y)
}

/// `string_frame` after the lock.
pub fn string_frame_locked(
    o: crate::owners::Owners<'static>,
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    let (gfx, system) = (o.gfx, o.system);
    copy_picture(gfx, system)?;
    draw_string(
        crate::strings::append_string_by_id::StringOwners::of(o),
        id,
        font,
        x,
        y,
        gfx,
    )?;
    crate::platform::unlock_surface(gfx);
    crate::platform::show_frame_now(o);
    Ok(())
}
/// A frame with a text built by the caller where the messages are, centred as they are.
pub fn text_frame(o: crate::owners::Owners<'static>, text: &[u8]) -> Result<(), Refused> {
    if !lock_surface(o.gfx, o.system) {
        return Ok(());
    }
    copy_picture(o.gfx, o.system)?;
    let shown = crate::net::lobby::store_text(o.strings, text.to_vec())?;
    crate::text::draw_text::draw_text_at(
        FONT_B,
        &shown,
        0x190,
        0x12C,
        &mut crate::text::draw_text::GameCanvas::new(o.gfx),
    )?;
    crate::platform::unlock_surface(o.gfx);
    crate::platform::show_frame_now(o);
    Ok(())
}

pub fn message_frame(o: crate::owners::Owners<'static>, message: u32) -> Result<(), Refused> {
    string_frame(o, message, FONT_B, 0x190, 0x12C)
}
