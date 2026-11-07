//! `gfx::pixel_to_components` (0x405CFA): takes a pixel apart into four 8-bit components,
//! one per byte of the result.
//!
//! Takes: pixel, pixel format
//! Returns: the components
//! A pixel of 0 gives 0 without looking at the format. With a format that has rotation
//! counts, each component is the pixel's masked bits rotated right by its count. With the
//! other kind, each component's bits are picked out of the pixel one at a time from the top
//! bit down, at most eight of them, and left-aligned in the byte if there are fewer.
//! Six callers. Refuses a null format.

use crate::gfx::pixel_format::PixelFormat;

// ---- Rust version ----

pub fn pixel_to_components(format: &PixelFormat, pixel: u32) -> u32 {
    // Converted: `rc2k_core::gfx::pixels::unpack`.
    rc2k_core::gfx::pixels::unpack(format, pixel)
}

// Written out without allocating: this is called per pixel.
