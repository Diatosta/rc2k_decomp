//! `hud::draw_marker` (0x4B55AB): a marker sprite (rectangle 0x5C of `dword_75B940`) along the top
//! at the share +0x6C over +0x68 of the HUD's record, times `flt_75C0A8` plus `flt_75C0AC` (`fistp`
//! through `dword_75C060`), coloured `dword_75BF50`.
//!
//! Takes: the HUD's record
//!
//! Caller: layout 0's drawer (`hud::layout_0`, 0x4B1F0C).

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::hud::store::rect_place;
use crate::race::owner_frame::Frame;
use rc2k_core::hud::quads::marker_x;

const RECT: u32 = 0x5C;

// Converted: `rc2k_core::hud::quads::marker_x`
pub fn draw_marker(frame: &Frame, hud: &Hud) -> Result<(), Refused> {
    hud.block.quad.colour.set(hud.block.marker_colours[0].get());
    {
        let f = f32::from_bits;
        let x = marker_x(
            f(frame.panel_and_overlay.panel.distance[0].get()),
            f(frame.panel_and_overlay.panel.overlay_miles.get()),
            hud.block.marker_k[0].get(),
            hud.block.marker_k[1].get(),
        );
        hud.block.fistp_scratch.set(x);
        crate::hud::add_sprite::add_sprite(frame, hud, x, 0, rect_place(hud, RECT))
    }
}
