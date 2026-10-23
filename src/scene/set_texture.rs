//! `scene::set_texture` (0x44E77E): `d3d::set_texture` with eax, unless `dword_5CDA8C` is 0.
//!
//! In:  eax the texture's number
//! Out: as `d3d::set_texture` leaves the registers (ecx 0 ..), or all kept
//!
//! 21 callers, none reading flags. `d3d::set_texture` is the platform's: called through
//! its hook (`call_game`, section 15). Shadow mode runs the original alone (it tells the
//! device). Self-test: a copy whose call goes to a stand-in noting the registers, against
//! the Rust version on the same stand-in, the switch 0 and 1.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_read, rva};

const SWITCH_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C
const LAYER_RVA: usize = rva(0x41ACF5); // d3d::set_texture

/// The Rust version (see the header); `part` makes the layer's call.
pub unsafe fn set_texture(r: &mut Regs, part: &mut dyn FnMut(&mut Regs)) {
    if unsafe { game_read::<u32>(SWITCH_RVA) } != 0 {
        part(r);
    }
}

/// In the game: the layer through its hook.
pub fn game_part(r: &mut Regs) {
    crate::calls::call(LAYER_RVA, r);
}

/// `set_texture` with the game's layer, for Rust callers.
pub unsafe fn set_texture_in_game(r: &mut Regs) {
    unsafe { set_texture(r, &mut game_part) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { set_texture_in_game(r) };
    Ok(())
}
