//! `scene::unlock_frame`: the back surface unlocked (`gfx::unlock_surface`) and the kept pitch and
//! pixels (`FrameScratch.frame_pixels`, `FrameScratch.frame_pitch`) 0.
use crate::race::owner_frame::Frame;

/// The back surface unlocked and the kept pitch and pixels 0. Returns the locked flag
/// before.
pub fn unlock_frame(frame: &Frame, gfx: &crate::gfx::owner::Gfx) -> u32 {
    let locked = crate::platform::unlock_surface(gfx);
    {
        frame.scratch().frame_pixels.set(0u32);
        frame.scratch().frame_pitch.set(0u32);
    }
    locked
}
