//! `gfx::rgb_pixels_to_dwords` (0x41CC96): turns the picture buffer's decoded pixels of three
//! bytes into dwords, in place.
//!
//! In:  nothing
//! Out: every register preserved
//! `g_pictureWidth` * `g_pictureHeight` pixels, from the last down: the first byte shifted
//! up 16, the second up 8, the third; the top byte 0. One caller, `gfx::read_png`, for a
//! file without a palette: every PNG file of the game.
//! Refuses what `pixels_to_dwords::pixel_count` refuses. The name is a reading.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::Regs;
use crate::gfx::picture::picture;
use crate::gfx::pixels_to_dwords::{pixel_count, spread_entry};

// ---- Rust version ----

pub unsafe fn rgb_pixels_to_dwords() -> Result<(), BadInput> {
    // Converted: `rc2k_core::gfx::pixels::rgb_to_dwords`.
    let count = unsafe { pixel_count() }?;
    let buffer = unsafe { std::slice::from_raw_parts_mut(picture(), count * 4) };
    rc2k_core::gfx::pixels::rgb_to_dwords(buffer, count).expect("the slice holds the dwords");
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    spread_entry(r, rgb_pixels_to_dwords)
}
