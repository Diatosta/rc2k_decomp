//! Shows "connecting" on the background for a second, then makes the connection with
//! `net::init_connection`. Returns whether that worked.

use crate::errors::Refused;
use crate::net::lobby::{lock_surface, run_frame};
use crate::race::viewer_screen_store::NetTexts;

pub const STATUS_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::STAGE_TITLE;

/// An empty frame, then string `id` shown on the background and a second's pause, which a key ends (not
/// when the lock fails). Shared with `net::hosting`, `net::joining`, `net::stopping`.
pub fn status(o: crate::owners::Owners<'static>, id: u32) -> Result<(), Refused> {
    run_frame(o, || Ok::<(), Refused>(()))?;
    let Some(locked) = lock_surface(o.gfx, o.system) else {
        return Ok(());
    };
    crate::net::message_frame::string_frame_locked(o, locked, id, STATUS_FONT, 0x316, 0x6F)?;
    crate::system::pause::pause(o, 1000);
    Ok(())
}
pub fn connecting(
    o: crate::owners::Owners<'static>,
    lines: &NetTexts,
    connection: u32,
) -> Result<bool, Refused> {
    status(o, 0x14F)?;
    crate::net::init_connection::init_connection(o.net, lines, connection)
}
