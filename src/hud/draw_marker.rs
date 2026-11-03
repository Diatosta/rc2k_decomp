//! `hud::draw_marker` (0x4B55AB): a marker sprite (rectangle 0x5C of `dword_75B940`) along
//! the top at the share +0x6C over +0x68 of the HUD's record, times `flt_75C0A8` plus
//! `flt_75C0AC` (`fistp` through `dword_75C060`), coloured `dword_75BF50`.
//!
//! In:  ebp the HUD's record
//! Out: eax 0x5C0, ebx the x, ecx 0, esi the rectangle (as `add_sprite` keeps them)
//!
//! Caller: layout 0's drawer (0x4B1F0C).

use crate::errors::Refused;
use crate::hud::store::{HUD, rect_addr};
use crate::pointers::GamePtr;
use rc2k_core::hud::quads::marker_x;

const RECT: u32 = 0x5C;

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::quads::marker_x`
pub fn draw_marker(record: GamePtr) -> Result<(), Refused> {
    HUD.quad[5].set(HUD.marker_colours[0].get());
    {
        let get = |o: u32| record.offset(o).read::<u32>("a hud record's field");
        let f = f32::from_bits;
        let x = marker_x(
            f(get(0x6C)?),
            f(get(0x68)?),
            HUD.marker_k[0].get(),
            HUD.marker_k[1].get(),
        );
        HUD.scratch.set(x);
        crate::hud::add_sprite::add_sprite(x, 0, rect_addr(RECT))
    }
}
