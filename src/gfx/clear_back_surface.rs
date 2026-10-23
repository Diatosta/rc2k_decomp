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
use crate::framework::Regs;
use crate::physics::flow::g;

const FRONT: usize = 0x4F32B4; // g_frontSurface
const PIXELS: usize = 0x4F4988; // dword_4F4988
const HEIGHT: usize = 0x4F4950; // g_screenHeight
const ROW_DWORDS: usize = 0x4F4960; // g_screenRowDwords
const GAP: usize = 0x4F498C; // dword_4F498C

/// The Rust version (see the header).
pub fn clear_back_surface() -> Result<(), Refused> {
    if g(FRONT) == 0 {
        return Ok(());
    }
    if g(HEIGHT) == 0 {
        return Err(BadInput("a screen of no rows: the original goes round 2^32 times").into());
    }
    // `cmp g_surfaceLocked, 1`'s carry: not locked.
    if crate::platform::lock_surface() < 1 {
        return Ok(());
    }
    let mut row = g(PIXELS);
    for _ in 0..g(HEIGHT) {
        let dwords = g(ROW_DWORDS);
        unsafe { std::ptr::write_bytes(row as *mut u32, 0, dwords as usize) };
        row = row.wrapping_add(4 * dwords).wrapping_add(g(GAP));
    }
    crate::platform::unlock_surface();
    Ok(())
}

/// The Rust version where the typed tables run it (a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), Refused> {
    clear_back_surface()
}
