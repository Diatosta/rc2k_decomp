//! `d3d::set_render_state` (0x41AB5B): sets one of Direct3D's render states, if it is
//! not set to that already.
//!
//! Takes: the value, the state
//! The value last set is kept per state in `dword_5AB6BC`; only a new one is stored and
//! passed to the device (`g_d3dDevice`, SetRenderState), if there is a device.
//! 38 callers, and `scene::set_render_state` (0x44EB72, 208 callers) is a call of it.
//! Platform: the original always runs, here it is only noted (`d3d/recorder.rs`). In
//! the standalone program the value is kept and nothing more, until a renderer is
//! there.

/// The states the game uses are all below this.
const STATES: u32 = 0x40;

/// Render state `state` set to `value` in drawing part `part`'s cache (the standalone keeps the
/// value).
pub fn set_render_state(part: &crate::workers::PartScratch, value: u32, state: u32) {
    if state < STATES {
        // Noted for the drawing threads' runs whether or not it changes (`run_states`).
        crate::d3d::run_states::note_set(part, state as usize);
        if crate::d3d::render_cache::kept(part, state as usize) != value {
            crate::d3d::render_cache::set_kept(part, state as usize, value);
            crate::platform_3d::forget_states(part);
        }
    }
}
