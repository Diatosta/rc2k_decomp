//! `hud::draw_marker` (0x4B55AB): a marker sprite (rectangle 0x5C of `HudBlock.rects[0].left`) along the top
//! at the share +0x6C over +0x68 of the HUD's record, times `HudBlock.marker_k[0]` plus `HudBlock.marker_k[1]` (`fistp`
//! through `HudBlock.fistp_scratch`), coloured `HudBlock.marker_colours[0]`.
//!
//! Takes: the HUD's record
//!
//! Caller: layout 0's drawer (`hud::layout_0`, 0x4B1F0C).

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::hud::store::RectAt;
use crate::race::owner_frame::Frame;
use rc2k_core::hud::quads::marker_x;

const RECT: u32 = 0x5C;

// Converted: `rc2k_core::hud::quads::marker_x`
pub fn draw_marker(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    hud.block.quad.colour.set(hud.block.marker_colours[0].get());
    {
        let f = f32::from_bits;
        let x = marker_x(
            f(frame.panel_and_overlay.get().panel.distance[0].get()),
            f(frame.panel_and_overlay.get().panel.overlay_miles.get()),
            hud.block.marker_k[0].get(),
            hud.block.marker_k[1].get(),
        );
        hud.block.fistp_scratch.set(x);
        crate::hud::add_sprite::add_sprite(frame, hud, pen, x, 0, RectAt::number(RECT))
    }
}
