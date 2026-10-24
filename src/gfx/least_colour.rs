//! `gfx::least_colour` (0x4053BC): the least colour a texture's format has.
//!
//! In:  eax = the format (`PixelFormat`)
//! Out: eax = the components of a pixel with the lowest bit of each run of each mask
//!      set; everything else preserved
//!
//! Through `gfx::pixel_to_components` (`gfx/texture_pixels.rs`). Two call sites, as
//! `gfx::alpha_from_kept`'s: the key black is replaced with.
//! Refuses a null format.
//! Self-test: the nine test formats.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::gfx::pixel_format::PixelFormat;
use crate::gfx::texture_pixels;

/// The Rust version: the least colour of the format at `format`.
pub fn least_colour(format: u32) -> Result<u32, Refused> {
    if format == 0 {
        return Err(BadInput("a null format").into());
    }
    let format = unsafe { (format as *const PixelFormat).read_unaligned() };
    Ok(texture_pixels::least_colour(&format))
}
