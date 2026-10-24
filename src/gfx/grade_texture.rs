//! `gfx::grade_texture` (0x4055A2): a texture's colours made brighter or duller.
//!
//! In:  eax = the key (components), ebx = the pixels, ecx = width, edx = height
//! Out: every register preserved
//!
//! Only with the reading switch `dword_4F4A24` at 1 and the scale `dword_4F4A28` and
//! the saturation `dword_4F4A2C` not both 0x100 (`scene::reload_system_textures` sets
//! them): the table `byte_4F4A30` made from the scale, then every pixel of an alpha
//! above 0 graded (`gfx/texture_pixels.rs`). Two call sites, as
//! `gfx::alpha_from_kept`'s.
//! Refuses no pixels (the original counts through all 32 bits) and a null pointer.
//! Self-test: 16 pictures by lot under eight settings.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write};
use crate::gfx::texture_pixels::{
    self, GRADES_RVA, READING_RVA, SATURATION_RVA, SCALE_RVA, grading,
};

fn settings() -> [u32; 3] {
    unsafe {
        [
            game_read(READING_RVA),
            game_read(SCALE_RVA),
            game_read(SATURATION_RVA),
        ]
    }
}

/// The Rust version: `pixels` `width` by `height`, `key` the colour keyed.
pub fn grade_texture(key: u32, pixels: u32, width: u32, height: u32) -> Result<(), Refused> {
    let [reading, scale, saturation] = settings();
    if !grading(reading, scale, saturation) {
        return Ok(());
    }
    let count = width.wrapping_mul(height);
    if pixels == 0 || count == 0 {
        return Err(BadInput("no pixels, or a null pointer").into());
    }
    unsafe {
        let mut table = [0u8; 0x100];
        texture_pixels::grade_texture(pixels as *mut u8, count, key, scale, saturation, &mut table);
        game_write(GRADES_RVA, table);
    }
    Ok(())
}
