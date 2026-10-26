//! `gfx::key_texture_colours` (0x40540C): a texture's black and magenta.
//!
//! In:  eax = the key (components), ebx = the pixels, ecx = width, edx = height
//! Out: every register preserved
//!
//! Each pixel's alpha made 0xFF; one whose colour's top five bits are all 0 becomes the
//! key with alpha 0xFF, one that is 0xF800F8 by those bits becomes 0
//! (`gfx/texture_pixels.rs`). Two call sites, as `gfx::alpha_from_kept`'s, when the
//! texture's record has +0x5C at 0.
//! Refuses no pixels (the original counts through all 32 bits) and a null pointer.
//! Self-test: 12 pictures by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::gfx::harness::pixels_entry;
use crate::gfx::texture_pixels::{self};
use crate::screens::harness::Left;

fn count(r: &Regs) -> u32 {
    r.ecx.wrapping_mul(r.edx)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ebx == 0 || count(r) == 0 {
        return Err(BadInput("no pixels, or a null pointer").into());
    }
    unsafe { texture_pixels::key_texture_colours(r.ebx as *mut u32, count(r), r.eax) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    pixels_entry(r, &rust)
}
