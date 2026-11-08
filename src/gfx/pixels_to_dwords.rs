//! What `gfx::palette_pixels_to_dwords` and `gfx::rgb_pixels_to_dwords` share: both spread
//! the picture buffer's freshly decoded pixels, one or three bytes each and one after the
//! other, out to a dword a pixel, in place, from the last pixel down.

use crate::errors::BadInput;
use crate::gfx::picture::has_picture;

/// The number of pixels, `g_pictureWidth` * `g_pictureHeight`.
/// Refuses none (the original then writes before the buffer) and more than a signed number
/// holds (it then does one pixel and stops), and a null buffer.
pub fn pixel_count(gfx: &crate::gfx::owner::Gfx) -> Result<usize, BadInput> {
    let pictures = &gfx.pictures;
    let count = pictures
        .picture
        .width
        .get()
        .wrapping_mul(pictures.picture.height.get());
    if !has_picture() {
        return Err(BadInput("no picture buffer"));
    }
    if count == 0 || count > i32::MAX as u32 {
        return Err(BadInput("a picture of no pixels, or of too many"));
    }
    Ok(count as usize)
}
