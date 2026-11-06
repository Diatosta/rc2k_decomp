//! `gfx::clear_back_surface` (0x40579A): with a front surface (`g_frontSurface`), the
//! screen locked (`gfx::lock_surface`; failing, nothing), its rows (`g_screenHeight` of
//! `g_screenRowDwords` dwords from `dword_4F4988`, `dword_4F498C` bytes between them)
//! zeroed and unlocked (`gfx::unlock_surface`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::viewer_frame` (at the replay's end), `gfx::set_screen_mode`,
//! `sub_405B10`. The lock and unlock (the platform's) are called directly (2026-10-23). A
//! height of 0 is refused (the original goes round 2^32 times), before anything.

use crate::errors::{BadInput, Refused};
use crate::session::SCREEN;

/// The Rust version (see the header).
pub fn clear_back_surface() -> Result<(), Refused> {
    if crate::gfx::owner::gfx().drawing.surfaces.front.get() == 0 {
        return Ok(());
    }
    if SCREEN.height.get() == 0 {
        return Err(BadInput("a screen of no rows: the original goes round 2^32 times").into());
    }
    // `cmp g_surfaceLocked, 1`'s carry: not locked.
    if crate::platform::lock_surface() < 1 {
        return Ok(());
    }
    // Each row's dwords, then the gap to the next (the original's step).
    let dwords = SCREEN.row_dwords.get();
    let rows = crate::gfx::rows::Rows::new(
        crate::gfx::owner::gfx()
            .drawing
            .draw_surface
            .present_pixels
            .get(),
        (4 * dwords).wrapping_add(
            crate::gfx::owner::gfx()
                .drawing
                .draw_surface
                .present_gap
                .get(),
        ),
        SCREEN.height.get(),
    );
    for y in 0..rows.count() {
        rows.fill(y, 0, 0, 4 * dwords as usize, "the back surface")?;
    }
    crate::platform::unlock_surface();
    Ok(())
}
