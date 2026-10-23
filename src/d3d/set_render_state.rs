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

use crate::framework::{Regs, game_write, rva};

const KEPT_RVA: usize = rva(0x5AB6BC); // dword_5AB6BC
/// The states the game uses are all below this.
const STATES: u32 = 0x40;

/// Render state `state` set to `value` (the standalone keeps the value).
pub fn set_render_state(value: u32, state: u32) {
    if state < STATES {
        unsafe { game_write(KEPT_RVA + state as usize * 4, value) };
    }
}

/// For the typed tables' calls by address: eax the value, ebx the state.
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    set_render_state(r.eax, r.ebx);
    Ok(())
}
