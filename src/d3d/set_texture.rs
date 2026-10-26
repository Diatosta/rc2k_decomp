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

use crate::d3d::recorder::{Call, note};
use crate::framework::{RawHook, Regs, game_write, hook_asm, rva, standalone};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("d3d::set_texture", rva(0x41ACF5), thunk, &ORIGINAL);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const CURRENT_RVA: usize = rva(0x5AB93C); // g_currentTexture

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    // Swallowed, the number is still kept, so that the code above sees its own texture
    // as the current one for what it draws next; who swallows puts the variable back.
    if note(|| Call::Texture(r.eax)) || standalone() {
        unsafe { game_write(CURRENT_RVA, r.eax) };
        r.ecx = 0;
        return;
    }
    unsafe { call_original(r) };
}

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
