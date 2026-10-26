//! `hud::add_sprite` (0x4B70EE): a sprite from the texture's rectangle at esi (left, top,
//! width, height) put at (ebx, ecx): the quad record's texture places (<< 16), its corners
//! (the width and height made positive), then `hud::add_quad`.
//!
//! In:  esi the rectangle, ebx x, ecx y
//! Out: all kept (`pusha` .. `popa`)
//!
//! Callers: the HUD's drawers (code at 0x4B1E42, 0x4B1E62, ...).

use crate::errors::Refused;
use crate::hud::store::HUD;
use crate::pointers::GamePtr;
use rc2k_core::hud::quads;

/// The quad record's texture places from the rectangle at `s` (shared with
/// `hud::add_scaled_sprite`): the rectangle.
pub fn put_places(s: u32) -> Result<[u32; 4], Refused> {
    Ok({
        let rect: [u32; 4] = crate::marks::try_from_fn(|n| {
            Ok(GamePtr::new(s + 4 * n as u32).read::<u32>("a hud record's field")?)
        })?;
        for (n, v) in quads::sprite_places(rect) {
            HUD.quad[n].set(v);
        }
        rect
    })
}

/// The quad record's corners (see `quads::sprite_corners`).
pub fn put_corners(x: u32, y: u32, w: u32, h: u32) {
    for (n, v) in quads::sprite_corners(x, y, w, h) {
        HUD.quad[n].set(v);
    }
}

/// The Rust version (see the header): the sprite whose rectangle is at `rect` with its
/// corner at `x`, `y` (ebx, ecx, esi; every register kept).
// Converted: `rc2k_core::hud::quads::{sprite_places, sprite_corners}`
pub fn add_sprite(x: u32, y: u32, rect: u32) -> Result<(), Refused> {
    let [_, _, w, h] = put_places(rect)?;
    put_corners(x, y, w, h);
    crate::hud::add_quad::add_quad()
}
