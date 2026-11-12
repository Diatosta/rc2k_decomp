//! `gfx::set_clip` (0x406988): sets the clip rectangle from a rectangle given as a corner
//! and a size.
//!
//! Takes: four dwords: x, y, width, height
//! `g_clipLeft` = x, `g_clipTop` = y, `g_clipRight` = x + width - 1, `g_clipBottom` = y +
//! height - 1. Three callers, each with the surface's own rectangle (`g_surface`).
//! Refuses a null rectangle. The name is a reading.

/// (left, top, right, bottom) for a rectangle of x, y, width, height. Converted:
/// `rc2k_core::gfx::raster::clip_of`.
pub use rc2k_core::gfx::raster::clip_of;

pub fn set_clip(rectangle: [u32; 4], gfx: &crate::gfx::owner::Gfx) {
    let drawing = &gfx.drawing;
    let [left, top, right, bottom] = clip_of(rectangle);
    drawing.blit.clip.left.set(left);
    drawing.blit.clip.top.set(top);
    drawing.blit.clip.right.set(right);
    drawing.blit.clip.bottom.set(bottom);
}
