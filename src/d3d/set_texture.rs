//! `d3d::set_texture` (0x41ACF5): makes a texture the one polygons are drawn with.
//!
//! In:  eax = the texture's number, or 0xFFFFFC43 for none
//! Out: ecx = 0; eax, edx as Direct3D leaves them
//! Nothing if it is the current one (`g_currentTexture`, 0x5AB93C); else that is set
//! and the device told: render state 1 with the texture's handle from the texture
//! table (0x78 bytes a texture at `dword_5ABC84`) and a light state, or render state 1
//! with 0 for none. `sub_41ACF0`, five bytes before it, loads eax with "none" and runs
//! into it. Four callers.
//! Platform: the original always runs, here it is only noted. In the standalone
//! program, and for a recording that swallows, the number is kept and nothing more.
//! No self-test.

use crate::framework::{Regs, game_write, rva, standalone};

pub const CURRENT_RVA: usize = rva(0x5AB93C); // g_currentTexture

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's). In the
/// hooked game it goes to the game's function, whose hook notes the call for the tests'
/// recorder (`d3d::recorder`) and runs the original: noted here as well, a call would be
/// noted twice.
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if !standalone() {
        unsafe { crate::framework::call_game(r, crate::framework::game_addr(rva(0x41ACF5))) };
        return Ok(());
    }
    unsafe { game_write(CURRENT_RVA, r.eax) };
    r.ecx = 0;
    Ok(())
}
