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
use crate::framework::{Regs, game_read, game_write};
use crate::gfx::harness::pixels_entry;
use crate::gfx::texture_pixels::{
    self, GRADES_RVA, READING_RVA, SATURATION_RVA, SCALE_RVA, grading,
};
use crate::screens::harness::Left;

fn count(r: &Regs) -> u32 {
    r.ecx.wrapping_mul(r.edx)
}

fn settings() -> [u32; 3] {
    unsafe {
        [
            game_read(READING_RVA),
            game_read(SCALE_RVA),
            game_read(SATURATION_RVA),
        ]
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let [reading, scale, saturation] = settings();
    if !grading(reading, scale, saturation) {
        return Ok(Left::nothing());
    }
    if r.ebx == 0 || count(r) == 0 {
        return Err(BadInput("no pixels, or a null pointer").into());
    }
    unsafe {
        let mut table = [0u8; 0x100];
        texture_pixels::grade_texture(
            r.ebx as *mut u8,
            count(r),
            r.eax,
            scale,
            saturation,
            &mut table,
        );
        game_write(GRADES_RVA, table);
    }
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    pixels_entry(r, &rust)
}
