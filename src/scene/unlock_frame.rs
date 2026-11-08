//! `scene::unlock_frame` (0x44E851): the back surface unlocked (`gfx::unlock_surface`, the
//! platform's) and the kept pitch and pixels (`dword_5C5FB4`, `dword_5C5FB0`) 0.
//!
//! Callers: `race::loading_bar_frame` and three of the game's. The registers above are the
//! original's; a call by address goes through `routes.rs`, which maps them.
use crate::race::owner_frame::Frame;

/// The back surface unlocked and the kept pitch and pixels 0. Returns `g_surfaceLocked`
/// before (the original leaves the flags of comparing it with 1, and eax 0).
pub fn unlock_frame(frame: &Frame) -> u32 {
    let locked = crate::platform::unlock_surface();
    {
        frame.scratch.frame_pixels.set(0u32);
        frame.scratch.frame_pitch.set(0u32);
    }
    locked
}
