//! Makes the front (shown) surface black.
//! Nothing without a front surface. The surface is locked (tried again while it is lost; failing, a
//! log line and failure: nothing more), the screen's height in rows of the screen's row dwords are
//! zeroed, each row the pitch on from the last, then it is unlocked (failing is a fatal error). The
//! lock and unlock are the platform's; the lock fails (the front surface is SDL's window), so
//! nothing is written. A height of 0 is refused before anything, as the row
//! countdown would clear 2^32 rows.

use crate::errors::{BadInput, Refused};
use crate::gfx::owner::Gfx;
use crate::system::owner::System;

/// The lock fails, so only the checks are made.
pub fn clear_front_surface(gfx: &Gfx, system: &System) -> Result<(), Refused> {
    if gfx.drawing.surfaces.front.get() != 0 && system.session.screen.get().height.get() == 0 {
        return Err(BadInput("a screen height of 0 (2^32 rows)").into());
    }
    Ok(())
}
