//! `gfx::alpha_from_kept` (0x405443): a texture's alpha from a second picture.
//!
//! In:  ebx = the pixels, ecx = width, edx = height
//! Out: every register preserved
//!
//! Each pixel's byte 3 = the first byte of the pixel 0x40000 bytes on, inverted
//! (`gfx/texture_pixels.rs`). Two call sites: `sub_41A68B` and `scene::upload_car_icon`,
//! when the texture's record has +0x5C at 1.
//! Refuses no pixels (the original counts through all 32 bits) and a null pointer.
//! Self-test: 12 pictures by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::gfx::texture_pixels;

/// The Rust version: `pixels` `width` by `height`.
pub fn alpha_from_kept(pixels: u32, width: u32, height: u32) -> Result<(), Refused> {
    let count = width.wrapping_mul(height);
    if pixels == 0 || count == 0 || count > 0x10000 {
        return Err(
            BadInput("no pixels, more than the kept picture's room, or a null pointer").into(),
        );
    }
    unsafe { texture_pixels::alpha_from_kept(pixels as *mut u8, count) };
    Ok(())
}
