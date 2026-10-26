//! `gfx::set_clip` (0x406988): sets the clip rectangle from a rectangle given as a corner
//! and a size.
//!
//! In:  ebx = four dwords: x, y, width, height
//! Out: every register preserved
//! `g_clipLeft` = x, `g_clipTop` = y, `g_clipRight` = x + width - 1, `g_clipBottom` = y +
//! height - 1. Three callers, each with the surface's own rectangle (`g_surface`).
//! Shadow mode runs the original and checks the four variables against the Rust
//! arithmetic. Refuses a null rectangle. The name is a reading.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::{Regs, game_write};
use crate::gfx::surface::{CLIP_BOTTOM_RVA, CLIP_LEFT_RVA, CLIP_RIGHT_RVA, CLIP_TOP_RVA};

// ---- Rust version ----

/// (left, top, right, bottom) for a rectangle of x, y, width, height. Converted:
/// `rc2k_core::gfx::raster::clip_of`.
pub use rc2k_core::gfx::raster::clip_of;

pub unsafe fn set_clip(rectangle: *const [u32; 4]) -> Result<(), BadInput> {
    if rectangle.is_null() {
        return Err(BadInput("no rectangle to clip to"));
    }
    unsafe {
        let [left, top, right, bottom] = clip_of(rectangle.read_unaligned());
        game_write(CLIP_LEFT_RVA, left);
        game_write(CLIP_TOP_RVA, top);
        game_write(CLIP_RIGHT_RVA, right);
        game_write(CLIP_BOTTOM_RVA, bottom);
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let rectangle = r.ebx as *const [u32; 4];
    if let Err(e) = unsafe { set_clip(rectangle) } {
        return Err(e.into());
    }
    Ok(())
}

// ---- self-test: a copy of the original that writes four variables of the test's instead
// of the clip rectangle, which the game's drawing reads all the time ----
