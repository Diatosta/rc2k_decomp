//! `scene::depth_test_off` (0x44E3D1): Direct3D render state 14 (writing to the depth buffer)
//! 0.
//!
//! In:  nothing
//! Out: eax 0, ebx 0xE; the rest as `scene::set_render_state` leaves them (kept)
//!
//! One call of `scene::set_render_state`. 14 callers. Self-test: the original and the Rust
//! version recorded on the layer, nothing drawn.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::scene::set_render_state::set_render_state;

/// The Rust version (see the header).
pub fn depth_test_off() {
    set_render_state(0, 0xE);
}
