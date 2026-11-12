//! `net::message_frame` (0x41748D): the frame of `net::show_message`: the surface locked
//! (`gfx::lock_surface`; failing, nothing), the background copied, the message (string)
//! drawn with `font_b` at 0x190, 0x12C, unlocked and shown.
//!
//! Takes `message`.
//! Called by `screens::run_frame` for `net::show_message`.

use crate::errors::Refused;
use crate::net::lobby::{copy_picture, draw_string, lock_surface};

const FONT_B: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::CAPTION_B;

/// A frame with one string: locked, the background, the string, unlocked, shown; nothing if
/// the lock fails.
pub fn string_frame(
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: u32,
    y: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    if !lock_surface(gfx, system) {
        return Ok(());
    }
    string_frame_locked(id, font, x, y, gfx, system)
}

/// `string_frame` after the lock.
pub fn string_frame_locked(
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: u32,
    y: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    copy_picture(gfx, system)?;
    draw_string(id, font, x, y, gfx)?;
    crate::platform::unlock_surface(gfx);
    crate::platform::show_frame_now();
    Ok(())
}
pub fn message_frame(
    message: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    string_frame(message, FONT_B, 0x190, 0x12C, gfx, system)
}
