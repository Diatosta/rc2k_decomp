//! A sprite from the texture's rectangle (left, top, width,
//! height) put at: the quad record's texture places (<< 16), its corners (the width and height made
//! positive), then `hud::add_quad`.

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::hud::store::RectAt;
use crate::race::owner_frame::Frame;
use rc2k_core::hud::quads;

/// The quad record's texture places from the rectangle at `s` (shared with
/// `hud::add_scaled_sprite`): the rectangle.
pub fn put_places(hud: &Hud, s: RectAt) -> Result<[u32; 4], Refused> {
    Ok({
        let rect = s.read(hud)?;
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
pub fn add_sprite(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    x: u32,
    y: u32,
    rect: RectAt,
) -> Result<(), Refused> {
    let [_, _, w, h] = put_places(hud, rect)?;
    put_corners(hud, x, y, w, h);
    crate::hud::add_quad::add_quad(frame, hud, pen)
}
