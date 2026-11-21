//! Puts four 8-bit components, one per byte of a dword, together into a pixel. The reverse of
//! `pixel_to_components`.
//! Components of 0 give 0 without looking at the format. With a format that has rotation counts,
//! the pixel is the four components rotated left by their counts and masked. With the other kind
//! the pixel is built one bit at a time from the format's top bit down: each bit takes the next
//! bit, from the top, of the first component whose mask has it; the components are shifted out of
//! four scratch bytes (`scratch`), which are left as they end up. Called per pixel by the loops
//! that scale and convert pictures. Refuses a null format.

use crate::gfx::owner::Gfx;
use crate::gfx::pixel_format::PixelFormat;

/// The pixel, and what is left in the scratch bytes if they are used
/// (`rc2k_core::gfx::pixels::pack`).
pub use rc2k_core::gfx::pixels::pack as packed;

pub fn components_to_pixel(format: &PixelFormat, components: u32, gfx: &Gfx) -> u32 {
    let (pixel, scratch) = packed(format, components);
    if let Some(scratch) = scratch {
        gfx.pixel.formats.scratch.set(scratch);
    }
    pixel
}

// Written out without allocating: pictures are converted one pixel at a time through here (19
// million calls while the game loads).
