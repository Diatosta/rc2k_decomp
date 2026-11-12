//! `gfx::clear_front_surface` (0x4059F4): the front (shown) surface made black.
//!
//! Nothing without `g_frontSurface`. The surface locked (DirectDraw's `Lock`
//! into the surface description, tried again while the surface is lost; failing,
//! a log line and the carry): nothing more. Else `g_screenHeight` rows of
//! `g_screenRowDwords` zero dwords from the description's memory, each
//! row the pitch on from the last, then unlocked (`Unlock`;
//! failing, fatal error 0x80208272, raised there). The lock and unlock are the platform's
//! (DirectDraw); in the standalone program the lock fails (its front
//! surface is SDL's window: `standalone.rs` would need a lock of its own for this to clear
//! it), so nothing is written. Refuses (before anything) a height of 0: the original's
//! row countdown would clear 2^32 rows. Six callers (the screen mode's set-up, the films,
//! the menus), by address (`net::lobby`'s `CLEAR_FRONT`, the typed tables).

use crate::errors::{BadInput, Refused};
use crate::gfx::owner::Gfx;
use crate::system::owner::System;

/// In the standalone the lock fails, so only the
/// checks are made.
pub fn clear_front_surface(gfx: &Gfx, system: &System) -> Result<(), Refused> {
    if gfx.drawing.surfaces.front.is_set() && system.session.screen.height.get() == 0 {
        return Err(BadInput("a screen height of 0 (2^32 rows)").into());
    }
    Ok(())
}
