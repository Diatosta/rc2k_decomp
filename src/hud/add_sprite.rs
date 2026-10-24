//! `hud::add_sprite` (0x4B70EE): a sprite from the texture's rectangle at esi (left, top,
//! width, height) put at (ebx, ecx): the quad record's texture places (<< 16), its corners
//! (the width and height made positive), then `hud::add_quad`.
//!
//! In:  esi the rectangle, ebx x, ecx y
//! Out: all kept (`pusha` .. `popa`)
//!
//! Callers: the HUD's drawers (code at 0x4B1E42, 0x4B1E62, ...). The quad through `part`
//! (1), in the game its Rust version. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version, rectangles
//! by lot (negative sizes among them).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::hud::add_quad::QUAD_RVA;
use crate::hud::store::game_write;
use rc2k_core::hud::quads;

/// The quad record's texture places from the rectangle at `s` (shared with
/// `hud::add_scaled_sprite`): the rectangle.
pub unsafe fn put_places(s: u32) -> [u32; 4] {
    unsafe {
        let rect: [u32; 4] =
            std::array::from_fn(|n| ((s + 4 * n as u32) as *const u32).read_unaligned());
        for (n, v) in quads::sprite_places(rect) {
            game_write(QUAD_RVA + 4 * n, v);
        }
        rect
    }
}

/// The quad record's corners (see `quads::sprite_corners`).
pub unsafe fn put_corners(x: u32, y: u32, w: u32, h: u32) {
    for (n, v) in quads::sprite_corners(x, y, w, h) {
        unsafe { game_write(QUAD_RVA + 4 * n, v) };
    }
}

/// The Rust version (see the header): the sprite whose rectangle is at `rect` with its
/// corner at `x`, `y` (ebx, ecx, esi; every register kept).
// Converted: `rc2k_core::hud::quads::{sprite_places, sprite_corners}`
pub unsafe fn add_sprite(x: u32, y: u32, rect: u32) -> Result<(), Refused> {
    unsafe {
        let [_, _, w, h] = put_places(rect);
        put_corners(x, y, w, h);
        crate::hud::add_quad::add_quad()
    }
}
