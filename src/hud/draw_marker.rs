//! `hud::draw_marker` (0x4B55AB): a marker sprite (rectangle 0x5C of `dword_75B940`) along
//! the top at the share +0x6C over +0x68 of the HUD's record, times `flt_75C0A8` plus
//! `flt_75C0AC` (`fistp` through `dword_75C060`), coloured `dword_75BF50`.
//!
//! In:  ebp the HUD's record
//! Out: eax 0x5C0, ebx the x, ecx 0, esi the rectangle (as `add_sprite` keeps them)
//!
//! Caller: layout 0's drawer (0x4B1F0C).

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::hud::store::rect_place;
use crate::race::owner::race;
use rc2k_core::hud::quads::marker_x;

const RECT: u32 = 0x5C;

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::quads::marker_x`
pub fn draw_marker(hud: &Hud) -> Result<(), Refused> {
    hud.block.quad[5].set(hud.block.marker_colours[0].get());
    {
        let f = f32::from_bits;
        let x = marker_x(
            f(race().frame.panel_and_overlay.panel.distance[0].get()),
            f(race().frame.panel_and_overlay.panel.overlay_miles.get()),
            hud.block.marker_k[0].get(),
            hud.block.marker_k[1].get(),
        );
        hud.block.scratch.set(x);
        crate::hud::add_sprite::add_sprite(hud, x, 0, rect_place(hud, RECT))
    }
}
