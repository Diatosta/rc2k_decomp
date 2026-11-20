//! `d3d::set_render_state`: sets one of Direct3D's render states, if it is not set to that
//! already. Takes the value and the state. The value last set is kept per state; only a new one
//! is stored. 38 callers, and `scene::set_render_state` (208 callers) is a call of it. The
//! state is only noted (`d3d/recorder.rs`); the value is kept and nothing more.

/// The states the game uses are all below this.
const STATES: u32 = 0x40;

/// Render state `state` set to `value` in drawing part `part`'s cache.
pub fn set_render_state(part: &crate::scene::workers::PartScratch, value: u32, state: u32) {
    if state < STATES {
        // Noted for the drawing threads' runs whether or not it changes (`run_states`).
        crate::d3d::run_states::note_set(part, state as usize);
        if crate::d3d::render_cache::kept(part, state as usize) != value {
            crate::d3d::render_cache::set_kept(part, state as usize, value);
            crate::d3d::device::forget_states(part);
        }
    }
}
