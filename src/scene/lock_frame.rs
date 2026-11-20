//! `scene::lock_frame`: the back surface locked (`gfx::lock_surface`) and its pitch and pixels kept
//! for the drawing (`FrameScratch.frame_pixels`, `FrameScratch.frame_pitch`).

use crate::gfx::owner::Gfx;
use crate::race::owner::Race;

/// The back surface locked; its pixels and pitch kept for the drawing. Returns the locked flag
/// after the lock (1 when locked; otherwise the frame is lost).
pub fn lock_frame(race: &Race, gfx: &Gfx, system: &crate::system::owner::System) -> u32 {
    let drawing = &gfx.drawing;
    let locked = crate::platform::lock_surface(gfx, system);
    {
        race.frame
            .scratch()
            .frame_pixels
            .set(drawing.draw_surface.present_pixels.get());
        race.frame
            .scratch()
            .frame_pitch
            .set(drawing.draw_surface.present_pitch.get());
        locked
    }
}
