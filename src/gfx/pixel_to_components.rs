//! `gfx::pixel_to_components` (0x405CFA): takes a pixel apart into four 8-bit components,
//! one per byte of the result.
//!
//! In:  eax = pixel, ebx = pixel format
//! Out: eax = the components; everything else preserved
//! A pixel of 0 gives 0 without looking at the format. With a format that has rotation
//! counts, each component is the pixel's masked bits rotated right by its count. With the
//! other kind, each component's bits are picked out of the pixel one at a time from the top
//! bit down, at most eight of them, and left-aligned in the byte if there are fewer.
//! Six callers. Manual patch with a private copy (0xBA bytes, nothing to relocate): a
//! conditional jump sits in the first bytes.
//! Refuses a null format.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, rva};
use crate::gfx::pixel_format::PixelFormat;

// ---- Rust version ----

pub fn pixel_to_components(format: &PixelFormat, pixel: u32) -> u32 {
    // Converted: `rc2k_core::gfx::pixels::unpack`.
    rc2k_core::gfx::pixels::unpack(format, pixel)
}

// Written out instead of going through `dispatch_state`, which allocates: this is called per
// pixel.
/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if r.ebx == 0 {
        unsafe { crate::framework::call_game(r, crate::framework::game_addr(rva(0x405CFA))) };
        return Ok(());
    }
    let format = unsafe { (r.ebx as *const PixelFormat).read_unaligned() };
    let mine = pixel_to_components(&format, r.eax);
    r.eax = mine;
    Ok(())
}
