//! `d3d::set_render_state` (0x41AB5B): sets one of Direct3D's render states, if it is
//! not set to that already.
//!
//! In:  eax = the value, ebx = the state
//! Out: every register preserved
//! The value last set is kept per state in `dword_5AB6BC`; only a new one is stored and
//! passed to the device (`g_d3dDevice`, SetRenderState), if there is a device.
//! 38 callers, and `scene::set_render_state` (0x44EB72, 208 callers) is a call of it.
//! Platform: the original always runs, here it is only noted (`d3d/recorder.rs`). In
//! the standalone program the value is kept and nothing more, until a renderer is
//! there. No self-test: there is nothing of ours to compare.

use crate::d3d::recorder::{Call, note};
use crate::framework::{RawHook, Regs, game_write, hook_asm, rva, standalone};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("d3d::set_render_state", rva(0x41AB5B), thunk, &ORIGINAL);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const KEPT_RVA: usize = rva(0x5AB6BC); // dword_5AB6BC
/// The states the game uses are all below this.
const STATES: u32 = 0x40;

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let swallowed = note(|| Call::State {
        state: r.ebx,
        value: r.eax,
    });
    if swallowed {
        return;
    }
    if standalone() {
        if r.ebx < STATES {
            unsafe { game_write(KEPT_RVA + r.ebx as usize * 4, r.eax) };
        }
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
        unsafe { crate::framework::call_game(r, crate::framework::game_addr(rva(0x41AB5B))) };
        return Ok(());
    }
    if r.ebx < STATES {
        unsafe { game_write(KEPT_RVA + r.ebx as usize * 4, r.eax) };
    }
    Ok(())
}
