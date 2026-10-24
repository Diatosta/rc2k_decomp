//! `gfx::halve_texture` (0x40545F): a texture's picture made smaller for its mip
//! levels.
//!
//! In:  eax = how many times, ebx = the pixels, ecx = width, edx = height
//! Out: ecx, edx = the size after the last time (as they were for none); everything
//!      else preserved
//!
//! In place, so many times: half as wide and high, each pixel the average of four,
//! the colour counting only those of an alpha above 0 (`gfx/texture_pixels.rs`). An
//! odd width leaves the rows a pixel short of where the original's pointer goes, as it
//! does. Two call sites, as `gfx::alpha_from_kept`'s.
//! Refuses a size that halves to 0 (the original counts through all 32 bits) and a
//! null pointer.
//! Self-test: 14 pictures by lot, 0 to 5 times.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::gfx::texture_pixels::{self, halvable};

/// The Rust version: `pixels` `width` by `height` halved `times` times; the size after
/// the last time (the original's ecx, edx).
pub fn halve_texture(
    times: u32,
    pixels: u32,
    width: u32,
    height: u32,
) -> Result<(u32, u32), Refused> {
    if !(times == 0 || (pixels != 0 && halvable(width, height, times))) {
        return Err(BadInput("a size that halves to nothing, or a null pointer").into());
    }
    if times == 0 {
        return Ok((width, height));
    }
    Ok(unsafe { texture_pixels::halve_texture(pixels as *mut u32, width, height, times) })
}
