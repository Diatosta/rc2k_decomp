//! `drive::play_inputs_if` (0x4600A9): `drive::play_inputs` while playing a replay
//! (`dword_5C5CE0` 2).
//!
//! In:  nothing
//! Out: as `play_inputs` leaves the registers when called; no flags read
//!
//! Caller: `drive::replay_step`.

use crate::physics::flow::g;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::plays_inputs`.
pub unsafe fn play_inputs_if() {
    if rc2k_core::drive::replay_frames::plays_inputs(g(0x5C5CE0)) {
        unsafe { crate::drive::play_inputs::play_inputs() };
    }
}
