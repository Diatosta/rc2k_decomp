//! `scene::lock_frame` (0x44E837): the back surface locked (`gfx::lock_surface`, the
//! platform's) and its pitch and pixels kept for the drawing (`dword_4F4988` into
//! `dword_5C5FB4`, `dword_4F4984` into `dword_5C5FB0`).
//!
//! In:  nothing
//! Out: CF as the lock leaves it; eax the pixels; the rest as the lock leaves them
//!
//! Callers: `race::loading_bar_frame` and three of the game's (0x442675, 0x4435A0, 0x4435C2).
//! The registers above are the original's; a call by address goes through `routes.rs`, which
//! maps them.

use crate::gfx::screen_state::DRAW_SURFACE;
use crate::race::scratch_store::FRAME_SCRATCH;

/// The back surface locked; its pixels and `dword_4F4984` kept for the drawing. Returns
/// `g_surfaceLocked` after the lock (the original leaves the flags of comparing it with
/// 1: carry when not locked, the frame lost) and `dword_4F4984` (left in eax).
pub fn lock_frame() -> (u32, u32) {
    let locked = crate::platform::lock_surface();
    {
        FRAME_SCRATCH
            .frame_pixels
            .set(DRAW_SURFACE.present_pixels.get());
        let kept: u32 = DRAW_SURFACE.present_pitch.get();
        FRAME_SCRATCH.frame_pitch.set(kept);
        (locked, kept)
    }
}
