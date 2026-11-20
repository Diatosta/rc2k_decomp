//! `scene::unlock_frame`: the back surface unlocked (`gfx::unlock_surface`).

/// The back surface unlocked. Returns the locked flag before.
pub fn unlock_frame(gfx: &crate::gfx::owner::Gfx) -> u32 {
    crate::platform::unlock_surface(gfx)
}
