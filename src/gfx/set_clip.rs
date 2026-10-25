//! `gfx::set_clip` (0x406988): sets the clip rectangle from a rectangle given as a corner
//! and a size.
//!
//! In:  ebx = four dwords: x, y, width, height
//! Out: every register preserved
//! `g_clipLeft` = x, `g_clipTop` = y, `g_clipRight` = x + width - 1, `g_clipBottom` = y +
//! height - 1. Three callers, each with the surface's own rectangle (`g_surface`).
//! Refuses a null rectangle. The name is a reading.

use crate::errors::BadInput;
use crate::gfx::blit::BLIT;

// ---- Rust version ----

/// (left, top, right, bottom) for a rectangle of x, y, width, height. Converted:
/// `rc2k_core::gfx::raster::clip_of`.
pub use rc2k_core::gfx::raster::clip_of;

pub unsafe fn set_clip(rectangle: *const [u32; 4]) -> Result<(), BadInput> {
    if rectangle.is_null() {
        return Err(BadInput("no rectangle to clip to"));
    }
    unsafe {
        let [left, top, right, bottom] = clip_of(rectangle.read_unaligned());
        BLIT.clip.left.set(left);
        BLIT.clip.top.set(top);
        BLIT.clip.right.set(right);
        BLIT.clip.bottom.set(bottom);
    }
    Ok(())
}
