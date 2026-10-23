//! `scene::clear_texture` (0x44E958): polygons drawn with no texture. Unless `dword_5CDA8C`
//! is 0, `sub_41ACF0`: eax "none" (0xFFFFFC43; `mov eax, 0FFFFFC43h`, five bytes) and on
//! into `d3d::set_texture` (0x41ACF5).
//!
//! In:  nothing
//! Out: with the switch on, eax 0xFFFFFC43 and what `d3d::set_texture` leaves (ecx 0;
//!      edx as Direct3D leaves it); else nothing
//!
//! Ported 2026-10-23: `marks::run_rain` called the game's for it (section 15 had it as
//! platform), and the standalone ended there on loading a stage with rain ("not ported:
//! 0x44e958"). No hook: the game's callers run the original, Rust callers this.

use crate::framework::{Regs, game_read, rva};

const SWITCH_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C
const LAYER_RVA: usize = rva(0x41ACF5); // d3d::set_texture
/// The texture number that means none.
pub const NO_TEXTURE: u32 = 0xFFFF_FC43;

/// The Rust version (see the header); `part` makes the layer's call.
pub unsafe fn clear_texture(r: &mut Regs, part: &mut dyn FnMut(&mut Regs)) {
    if unsafe { game_read::<u32>(SWITCH_RVA) } != 0 {
        r.eax = NO_TEXTURE;
        part(r);
    }
}

/// `clear_texture` with the game's layer (`d3d::set_texture`'s route), for Rust callers.
pub unsafe fn clear_texture_in_game(r: &mut Regs) {
    unsafe { clear_texture(r, &mut |r| crate::calls::call(LAYER_RVA, r)) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { clear_texture_in_game(r) };
    Ok(())
}
