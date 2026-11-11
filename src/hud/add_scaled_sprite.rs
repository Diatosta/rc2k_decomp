//! `hud::add_scaled_sprite` (0x4B716A): as `hud::add_sprite` with the width times st(0) and the
//! height times st(1) (`fistp`ed through `HudBlock.fistp_scratch`) before they are made positive; st(0) and
//! st(1) are read, not popped.
//!
//! Callers: the HUD's drawers (code at 0x4B28B0, ...).

use crate::errors::Refused;
use crate::hud::add_sprite::{put_corners, put_places};
use crate::hud::owner::{Hud, Pen};
use crate::pointers::GamePtr;
use crate::race::owner_frame::Frame;
use rc2k_core::hud::quads;
use rc2k_core::math::extended::to_f64;

/// The scales as `f64`s (what the x87 holds at the game's double precision): the sprite at `rect`
/// with its corner at `x`, `y`.
// Converted: `rc2k_core::hud::quads::{sprite_places, scaled_size, sprite_corners}`
pub fn add_scaled_sprite_plain(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    x: u32,
    y: u32,
    rect: GamePtr,
    scales: [f64; 2],
) -> Result<(), Refused> {
    let [_, _, w, h] = put_places(hud, rect)?;
    let w = quads::scaled_size(w, scales[0]);
    let h = quads::scaled_size(h, scales[1]);
    hud.block.fistp_scratch.set(h);
    put_corners(hud, x, y, w, h);
    crate::hud::add_quad::add_quad(frame, hud, pen)
}

/// `add_scaled_sprite_plain` with the scales as the x87 holds them.
pub fn add_scaled_sprite(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    x: u32,
    y: u32,
    rect: GamePtr,
    scales: [[u8; 10]; 2],
) -> Result<(), Refused> {
    add_scaled_sprite_plain(frame, hud, pen, x, y, rect, scales.map(to_f64))
}
