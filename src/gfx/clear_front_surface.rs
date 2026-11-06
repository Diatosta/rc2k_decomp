//! `gfx::clear_front_surface` (0x4059F4): the front (shown) surface made black.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! Nothing without `g_frontSurface`. The surface locked (`sub_405996`: DirectDraw's `Lock`
//! into the description at `dword_4F3400`, tried again while the surface is lost; failing,
//! a log line and the carry): nothing more. Else `g_screenHeight` rows of
//! `g_screenRowDwords` zero dwords from the description's memory (`dword_4F3424`), each
//! row `dword_4F3410` (the pitch) on from the last, then unlocked (`sub_4059D5`: `Unlock`;
//! failing, fatal error 0x80208272, raised there). The lock and unlock are the platform's
//! (DirectDraw); in the standalone program the lock fails (its front
//! surface is SDL's window: `standalone.rs` would need a lock of its own for this to clear
//! it), so nothing is written. Refuses (before anything) a height of 0: the original's
//! `dec edx; jnz` would clear 2^32 rows. Six callers (the screen mode's set-up, the films,
//! the menus), by address (`net::lobby`'s `CLEAR_FRONT`, the typed tables).

use crate::errors::{BadInput, Refused};
use crate::session::SCREEN;

/// The Rust version (see the header): in the standalone the lock fails, so only the
/// checks are made.
pub fn clear_front_surface() -> Result<(), Refused> {
    if crate::gfx::owner::gfx().drawing.surfaces.front.get() != 0 && SCREEN.height.get() == 0 {
        return Err(BadInput("a screen height of 0 (2^32 rows)").into());
    }
    Ok(())
}
