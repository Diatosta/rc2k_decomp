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
use crate::gfx::texture_pixels;

/// The Rust version: `pixels` `width` by `height`, black made `key`.
pub fn key_texture_colours(key: u32, pixels: u32, width: u32, height: u32) -> Result<(), Refused> {
    let count = width.wrapping_mul(height);
    if pixels == 0 || count == 0 {
        return Err(BadInput("no pixels, or a null pointer").into());
    }
    unsafe { texture_pixels::key_texture_colours(pixels as *mut u32, count, key) };
    Ok(())
}
