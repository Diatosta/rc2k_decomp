//! `drive::play_inputs_if` (0x4600A9): `drive::play_inputs` while playing a replay
//! (`dword_5C5CE0` 2).
//!
//! In:  nothing
//! Out: as `play_inputs` leaves the registers when called; no flags read
//!
//! Caller: `drive::replay_step`.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): the mode and cars.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::physics::flow::g;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::plays_inputs`.
pub unsafe fn play_inputs_if() {
    if rc2k_core::drive::replay_frames::plays_inputs(g(0x5C5CE0)) {
        unsafe { crate::drive::play_inputs::play_inputs() };
    }
}
