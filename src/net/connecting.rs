//! `net::connecting` (0x417688): an empty frame (`screens::run_frame`, eax 0), then on the
//! locked surface the background and "connecting" (string 0x14F, `stru_51E9A4` at 0x316,
//! 0x6F), shown, a second's sleep (none when the lock fails); then connection eax made
//! (`net::init_connection`).
//!
//! In:  `connection` (eax: the connection's number)
//! Out: whether `net::init_connection` worked (the original: its flags, carry when it
//!      failed; every register kept)
//! Caller: `net::start_link`.

use crate::errors::Refused;
use crate::net::lobby::{lock_surface, run_frame};

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

/// The Rust version.
pub fn connecting(connection: u32) -> Result<bool, Refused> {
    status(0x14F)?;
    crate::net::init_connection::init_connection(connection)
}
