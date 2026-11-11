//! `net::connecting` (0x417688): an empty frame (`screens::run_frame`), then on the
//! locked surface the background and "connecting" (string 0x14F, `typefaces::STAGE_TITLE` at 0x316,
//! 0x6F), shown, a second's sleep (none when the lock fails); then the connection made
//! (`net::init_connection`).
//!
//! Takes `connection`. Returns whether `net::init_connection` worked.
//! Caller: `net::start_link`.

use crate::errors::Refused;
use crate::net::lobby::{lock_surface, run_frame};
use crate::net::owner::Net;
use crate::race::viewer_screen_store::ViewerPanelBlock;

pub const STATUS_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::STAGE_TITLE;

/// An empty frame, then string `id` shown on the background and a second's sleep (not
/// when the lock fails). Shared with `net::hosting`, `net::joining`, `net::stopping`.
pub fn status(id: u32) -> Result<(), Refused> {
    run_frame(|| Ok::<(), Refused>(()))?;
    if !lock_surface() {
        return Ok(());
    }
    crate::net::message_frame::string_frame_locked(id, STATUS_FONT, 0x316, 0x6F)?;
    crate::system::sleep::sleep(1000);
    Ok(())
}
pub fn connecting(net: &Net, panel: &ViewerPanelBlock, connection: u32) -> Result<bool, Refused> {
    status(0x14F)?;
    crate::net::init_connection::init_connection(net, panel, connection)
}
