//! `hud::add_sprite` (0x4B70EE): a sprite from the texture's rectangle (left, top, width,
//! height) put at: the quad record's texture places (<< 16), its corners (the width and height made
//! positive), then `hud::add_quad`.
//!
//! Takes: the rectangle, x, y
//!
//! Callers: the HUD's drawers (several places).

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::pointers::GamePtr;
use crate::race::owner_frame::Frame;
use rc2k_core::hud::quads;

/// The quad record's texture places from the rectangle at `s` (shared with
/// `hud::add_scaled_sprite`): the rectangle.
pub fn put_places(hud: &Hud, s: GamePtr) -> Result<[u32; 4], Refused> {
    Ok({
        let rect: [u32; 4] = crate::effects::try_from_fn(|n| {
            Ok(s.offset(4 * n as u32).read::<u32>("a hud record's field")?)
        })?;
        for (n, v) in quads::sprite_places(rect) {
            hud.block.quad.word(n).set(v);
        }
        rect
    })
}

/// The quad record's corners (see `quads::sprite_corners`).
pub fn put_corners(hud: &Hud, x: u32, y: u32, w: u32, h: u32) {
    for (n, v) in quads::sprite_corners(x, y, w, h) {
        hud.block.quad.word(n).set(v);
    }
}

/// The sprite whose rectangle is the place `rect` with its corner at `x`, `y`.
// Converted: `rc2k_core::hud::quads::{sprite_places, sprite_corners}`
pub fn add_sprite(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    x: u32,
    y: u32,
    rect: GamePtr,
) -> Result<(), Refused> {
    let [_, _, w, h] = put_places(hud, rect)?;
    put_corners(hud, x, y, w, h);
    crate::hud::add_quad::add_quad(frame, hud, pen)
}
