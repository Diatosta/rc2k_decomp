//! As `hud::add_sprite` with the width times the first scale and the height times the second
//! (rounded to whole numbers through `HudBlock.fistp_scratch`) before they are made positive; the
//! scales are read, not popped.

use crate::errors::Refused;
use crate::hud::add_sprite::{put_corners, put_places};
use crate::hud::block_store::RectAt;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::race::owner_frame::Frame;
use rc2k_core::hud::quads;
use rc2k_core::math::extended::to_f64;

/// The scales as `f64`s (what the x87 holds at the game's double precision): the sprite at `rect`
/// with its corner at `x`, `y`.
pub(super) fn add_scaled_sprite_plain(
    frame: &Frame,
    hud: &mut Hud,
    pen: Pen,
    x: u32,
    y: u32,
    rect: RectAt,
    scales: [f64; 2],
) -> Result<(), Refused> {
    let [_, _, w, h] = put_places(hud, rect)?;
    let w = quads::scaled_size(w, scales[0]);
    let h = quads::scaled_size(h, scales[1]);
    hud.block.fistp_scratch = h;
    put_corners(hud, x, y, w, h);
    crate::hud::add_quad::add_quad(frame, hud, pen)
}

/// `add_scaled_sprite_plain` with the scales as the x87 holds them.
pub(super) fn add_scaled_sprite(
    frame: &Frame,
    hud: &mut Hud,
    pen: Pen,
    x: u32,
    y: u32,
    rect: RectAt,
    scales: [[u8; 10]; 2],
) -> Result<(), Refused> {
    add_scaled_sprite_plain(frame, hud, pen, x, y, rect, scales.map(to_f64))
}
