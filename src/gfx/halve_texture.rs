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
use crate::framework::{ECX, EDX, Regs};
use crate::gfx::harness::pixels_entry;
use crate::gfx::texture_pixels::{self, halvable};
use crate::screens::harness::Left;

fn usable(r: &Regs) -> bool {
    r.eax == 0 || (r.ebx != 0 && halvable(r.ecx, r.edx, r.eax))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    if !usable(r) {
        return Err(BadInput("a size that halves to nothing, or a null pointer").into());
    }
    if r.eax == 0 {
        return Ok(Left::nothing());
    }
    let (width, height) =
        unsafe { texture_pixels::halve_texture(r.ebx as *mut u32, r.ecx, r.edx, r.eax) };
    Ok(Left::nothing().with(ECX, width).with(EDX, height))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    pixels_entry(r, &rust)
}
