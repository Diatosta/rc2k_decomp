//! `hud::draw_marker` (0x4B55AB): a marker sprite (rectangle 0x5C of `dword_75B940`) along
//! the top at the share +0x6C over +0x68 of the HUD's record, times `flt_75C0A8` plus
//! `flt_75C0AC` (`fistp` through `dword_75C060`), coloured `dword_75BF50`.
//!
//! In:  ebp the HUD's record
//! Out: eax 0x5C0, ebx the x, ecx 0, esi the rectangle (as `add_sprite` keeps them)
//!
//! Caller: layout 0's drawer (0x4B1F0C).

use crate::errors::Refused;
use crate::hud::add_quad::QUAD_RVA;
use crate::hud::store::Slot;
use crate::hud::store::{game_addr, game_read, game_write};
use rc2k_core::hud::quads::marker_x;

const COLOUR_RVA: Slot = Slot::at(0x75BF50); // dword_75BF50
const K_RVA: [Slot; 2] = [Slot::at(0x75C0A8), Slot::at(0x75C0AC)];
const SCRATCH_RVA: Slot = Slot::at(0x75C060); // dword_75C060
pub const RECTS_RVA: Slot = Slot::at(0x75B940); // dword_75B940: 0x10 a rectangle
const RECT: u32 = 0x5C;

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::quads::marker_x`
pub unsafe fn draw_marker(record: u32) -> Result<(), Refused> {
    unsafe {
        game_write(QUAD_RVA + 0x14, game_read::<u32>(COLOUR_RVA));
        let get = |o: u32| ((record + o) as *const u32).read_unaligned();
        let f = f32::from_bits;
        let x = marker_x(
            f(get(0x6C)),
            f(get(0x68)),
            f(game_read(K_RVA[0])),
            f(game_read(K_RVA[1])),
        );
        game_write(SCRATCH_RVA, x);
        crate::hud::add_sprite::add_sprite(x, 0, game_addr(RECTS_RVA) as u32 + RECT * 0x10)
    }
}
