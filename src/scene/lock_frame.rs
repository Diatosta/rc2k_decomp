//! `scene::lock_frame` (0x44E837): the back surface locked (`gfx::lock_surface`, the
//! platform's) and its pitch and pixels kept for the drawing (`dword_4F4988` into
//! `FrameScratch.frame_pixels`, `dword_4F4984` into `FrameScratch.frame_pitch`).
//!
//! Callers: `race::loading_bar_frame` and three of the game's (0x442675, 0x4435A0, 0x4435C2).
//! The registers above are the original's; a call by address goes through `routes.rs`, which
//! maps them.

use crate::gfx::owner::Gfx;
use crate::race::owner::Race;

/// The back surface locked; its pixels and a second value kept for the drawing. Returns the
/// locked flag after the lock (1 when locked; otherwise the frame is lost) and that value.
pub fn lock_frame(race: &Race, gfx: &Gfx, system: &crate::system::owner::System) -> (u32, u32) {
    let drawing = &gfx.drawing;
    let locked = crate::platform::lock_surface(gfx, system);
    {
        race.frame
            .scratch
            .frame_pixels
            .set(drawing.draw_surface.present_pixels.get());
        let kept: u32 = drawing.draw_surface.present_pitch.get();
        race.frame.scratch.frame_pitch.set(kept);
        (locked, kept)
    }
}
