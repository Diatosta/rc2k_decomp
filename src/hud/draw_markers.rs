//! The two players' markers along the top of a split layout: for each player the rectangle
//! 0x5C of the sprite sheet (`hud.block`'s rectangles) at the player's distance shown
//! (`Panel::distance`) over the stage's miles times the first marker constant plus the second,
//! coloured by the player's marker colour, at the player's row, scaled 0x100 by 0x78 over
//! the horizontal texture scale (`hud::add_scaled_sprite`, the scales taken off after).
//!
//! Layout 1 draws them at rows 1 and 0xE, layout 2 (the lower half) at rows 0x11F and
//! 0x12C: `MARKER_ROWS_1`, `MARKER_ROWS_2`.

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::hud::store::RectAt;
use crate::race::owner_frame::Frame;
use rc2k_core::hud::quads;
use rc2k_core::math::extended::{from_f64, to_f64};

/// The rows of layout 1's markers (first player, second player).
pub(super) const MARKER_ROWS_1: [u32; 2] = [1, 0xE];
/// The rows of layout 2's markers.
pub(super) const MARKER_ROWS_2: [u32; 2] = [0x11F, 0x12C];

const RECT: u32 = 0x5C;

/// An integer over a float constant, kept extended (`quads::int_over`: exact at the game's double
/// precision).
pub fn int_over_80(i: u32, k: f32) -> [u8; 10] {
    from_f64(quads::int_over(i, k))
}

/// An 80-bit value taken off into a float (stored as a float).
pub(super) fn taken_off(v: [u8; 10]) -> u32 {
    (to_f64(v) as f32).to_bits()
}

/// The two players' markers at `rows` (see the header).
pub(super) fn draw_markers(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    rows: [u32; 2],
) -> Result<(), Refused> {
    {
        let k = hud.block.uv_scale_x.get();
        for (n, distance) in frame
            .panel_and_overlay
            .get()
            .panel
            .distance
            .iter()
            .enumerate()
        {
            hud.block.quad.colour.set(hud.block.marker_colours[n].get());
            let f = f32::from_bits;
            let x = quads::marker_x(
                f(distance.get()),
                f(frame.panel_and_overlay.get().panel.overlay_miles.get()),
                hud.block.marker_k[0].get(),
                hud.block.marker_k[1].get(),
            );
            hud.block.fistp_scratch.set(x);
            let rect = RectAt::number(RECT);
            hud.block.fistp_scratch.set(0x78u32);
            let st1 = int_over_80(0x78, k);
            hud.block.fistp_scratch.set(0x100u32);
            let st0 = int_over_80(0x100, k);
            crate::hud::add_scaled_sprite::add_scaled_sprite(
                frame,
                hud,
                pen,
                x,
                rows[n],
                rect,
                [st0, st1],
            )?;
            // the second scale is what stays in the scratch cell
            hud.block.fistp_scratch.set(taken_off(st1));
        }
    }
    Ok(())
}
