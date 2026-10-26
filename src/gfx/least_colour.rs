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
use crate::framework::{EAX, Regs};
use crate::gfx::harness::pixels_entry;
use crate::gfx::pixel_format::PixelFormat;
use crate::gfx::texture_pixels;
use crate::screens::harness::Left;

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.eax == 0 {
        return Err(BadInput("a null format").into());
    }
    let format = unsafe { (r.eax as *const PixelFormat).read_unaligned() };
    Ok(Left::nothing().with(EAX, texture_pixels::least_colour(&format)))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    pixels_entry(r, &rust)
}
