//! `gfx::clear_back_surface` (0x40579A): with a front surface (`g_frontSurface`), the
//! screen locked (`gfx::lock_surface`; failing, nothing), its rows (`g_screenHeight` of
//! `g_screenRowDwords` dwords from `dword_4F4988`, `dword_4F498C` bytes between them)
//! zeroed and unlocked (`gfx::unlock_surface`).
//!
//! Callers: `race::viewer_frame` (at the replay's end), `gfx::set_screen_mode`,
//! 0x405B10. The lock and unlock (the platform's) are called directly. A
//! height of 0 is refused (the original goes round 2^32 times), before anything.

use crate::errors::{BadInput, Refused};
pub fn clear_back_surface(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let drawing = &gfx.drawing;
    if drawing.surfaces.front.get() == 0 {
        return Ok(());
    }
    if system.session.screen.height.get() == 0 {
        return Err(BadInput("a screen of no rows: the original goes round 2^32 times").into());
    }
    // `cmp g_surfaceLocked, 1`'s carry: not locked.
    if crate::platform::lock_surface() < 1 {
        return Ok(());
    }
    // Each row's dwords, then the gap to the next (the original's step).
    let dwords = system.session.screen.row_dwords.get();
    let rows = crate::gfx::rows::Rows::new(
        drawing.draw_surface.present_pixels.get(),
        (4 * dwords).wrapping_add(drawing.draw_surface.present_gap.get()),
        system.session.screen.height.get(),
    );
    for y in 0..rows.count() {
        rows.fill(y, 0, 0, 4 * dwords as usize, "the back surface")?;
    }
    crate::platform::unlock_surface();
    Ok(())
}
