//! `gfx::set_clip` (0x406988): sets the clip rectangle from a rectangle given as a corner
//! and a size.
//!
//! In:  ebx = four dwords: x, y, width, height
//! Out: every register preserved
//! `g_clipLeft` = x, `g_clipTop` = y, `g_clipRight` = x + width - 1, `g_clipBottom` = y +
//! height - 1. Three callers, each with the surface's own rectangle (`g_surface`).
//! Refuses a null rectangle. The name is a reading.

// ---- Rust version ----

/// (left, top, right, bottom) for a rectangle of x, y, width, height. Converted:
/// `rc2k_core::gfx::raster::clip_of`.
pub use rc2k_core::gfx::raster::clip_of;

pub fn set_clip(rectangle: [u32; 4]) {
    let [left, top, right, bottom] = clip_of(rectangle);
    crate::gfx::owner::gfx().drawing.blit.clip.left.set(left);
    crate::gfx::owner::gfx().drawing.blit.clip.top.set(top);
    crate::gfx::owner::gfx().drawing.blit.clip.right.set(right);
    crate::gfx::owner::gfx()
        .drawing
        .blit
        .clip
        .bottom
        .set(bottom);
}
