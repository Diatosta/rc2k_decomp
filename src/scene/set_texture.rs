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

use crate::framework::{game_read, rva};

const SWITCH_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C

/// `d3d::set_texture` with the texture, unless the switch is 0. What it leaves in the
/// registers is read by none of its callers (register liveness, 2026-10-23).
pub fn set_texture(texture: u32) {
    if unsafe { game_read::<u32>(SWITCH_RVA) } != 0 {
        crate::d3d::set_texture::set_texture(texture);
    }
}
