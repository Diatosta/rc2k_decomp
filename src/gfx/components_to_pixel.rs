//! `gfx::components_to_pixel` (0x405C8A): puts four 8-bit components, one per byte of eax,
//! together into a pixel. The reverse of `pixel_to_components`.
//!
//! In:  eax = components, ebx = pixel format
//! Out: eax = the pixel; everything else preserved
//! Components of 0 give 0 without looking at the format. With a format that has rotation
//! counts, the pixel is the four components rotated left by their counts and masked. With
//! the other kind the pixel is built one bit at a time from the format's top bit down: each
//! bit takes the next bit, from the top, of the first component whose mask has it; the
//! components are shifted out of four scratch bytes (`dword_4F3FA8`), which are left as they
//! end up.
//! 30 call sites, among them the per-pixel loops that scale and convert pictures. Refuses a
//! null format.

use crate::gfx::formats::FORMATS;
use crate::gfx::pixel_format::PixelFormat;

// ---- Rust version ----

/// The pixel, and what is left in the scratch bytes if they are used (converted:
/// `rc2k_core::gfx::pixels::pack`).
pub use rc2k_core::gfx::pixels::pack as packed;

pub unsafe fn components_to_pixel(format: &PixelFormat, components: u32) -> u32 {
    let (pixel, scratch) = packed(format, components);
    if let Some(scratch) = scratch {
        FORMATS.scratch.write(&scratch);
    }
    pixel
}

// Written out without allocating: the game converts pictures one pixel at a time through
// here (19 million calls while it loads).
