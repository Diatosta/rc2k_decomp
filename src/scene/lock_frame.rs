//! `scene::lock_frame`: the back surface locked (`gfx::lock_surface`).

use crate::gfx::owner::Gfx;

/// The back surface locked. Returns the locked flag
/// after the lock (1 when locked; otherwise the frame is lost).
pub fn lock_frame(gfx: &Gfx, system: &crate::system::owner::System) -> u32 {
    crate::platform::lock_surface(gfx, system)
}
