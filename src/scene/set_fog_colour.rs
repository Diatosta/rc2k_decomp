//! `scene::set_fog_colour` (0x44E76B): the fog's colour: eax kept in `dword_5AB964` and
//! given to Direct3D (`d3d_set_fog_colour`, 0x41AE25: inside `pusha`, render state 0x22 =
//! eax with its top byte set, through `d3d::set_render_state`).
//!
//! In:  eax = the colour
//! Out: every register kept; no flags read
//!
//! Callers: `race::set_up_fog` and ten more. The layer's call is made here (the two lines
//! of `d3d_set_fog_colour`), `d3d::set_render_state` itself through the game.
//! Self-test: the original and the Rust version recorded on the layer, nothing drawn;
//! the colour kept compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_write, rva};

const KEPT_RVA: usize = rva(0x5AB964); // dword_5AB964
const LAYER_RVA: usize = rva(0x41AB5B); // d3d::set_render_state

/// The Rust version (see the header).
// Converted: `rc2k_core::scene::render_states::fog_colour`
pub unsafe fn set_fog_colour(colour: u32) {
    unsafe {
        game_write(KEPT_RVA, colour);
        let s = rc2k_core::scene::render_states::fog_colour(colour);
        let mut r = Regs {
            eax: s.value,
            ebx: s.state,
            ..Regs::default()
        };
        crate::calls::call(LAYER_RVA, &mut r);
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { set_fog_colour(r.eax) };
    Ok(())
}
