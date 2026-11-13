//! Sets the clip rectangle from a rectangle given as a corner and a size: four dwords x, y, width,
//! height become left = x, top = y, right = x + width - 1, bottom = y + height - 1. Refuses a null
//! rectangle.

/// (left, top, right, bottom) for a rectangle of x, y, width, height.
pub use rc2k_core::gfx::raster::clip_of;

pub fn set_clip(rectangle: [u32; 4], gfx: &crate::gfx::owner::Gfx) {
    let drawing = &gfx.drawing;
    let [left, top, right, bottom] = clip_of(rectangle);
    drawing.blit.clip.left.set(left);
    drawing.blit.clip.top.set(top);
    drawing.blit.clip.right.set(right);
    drawing.blit.clip.bottom.set(bottom);
}
