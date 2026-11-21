//! A sprite from the texture's rectangle (left, top, width,
//! height) put at: the quad record's texture places (<< 16), its corners (the width and height made
//! positive), then `hud::add_quad`.

use crate::errors::Refused;
use crate::hud::block_store::RectAt;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::race::owner_frame::Frame;
use rc2k_core::hud::quads;

/// The quad record's texture places from the rectangle at `s` (shared with
/// `hud::add_scaled_sprite`): the rectangle.
pub(super) fn put_places(hud: &mut Hud, s: RectAt) -> Result<[u32; 4], Refused> {
    Ok({
        let rect = s.read(hud)?;
        hud.block.quad.set_places(quads::sprite_places(rect));
        rect
    })
}

/// The quad record's corners (see `quads::sprite_corners`).
pub(super) fn put_corners(hud: &mut Hud, x: u32, y: u32, w: u32, h: u32) {
    hud.block.quad.set_rect(quads::sprite_corners(x, y, w, h));
}

/// The sprite whose rectangle is the place `rect` with its corner at `x`, `y`.
pub(super) fn add_sprite(
    frame: &Frame,
    hud: &mut Hud,
    pen: Pen,
    x: u32,
    y: u32,
    rect: RectAt,
) -> Result<(), Refused> {
    let [_, _, w, h] = put_places(hud, rect)?;
    put_corners(hud, x, y, w, h);
    crate::hud::add_quad::add_quad(frame, hud, pen)
}
