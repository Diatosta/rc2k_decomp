//! `drive::play_inputs_if` (0x4600A9): `drive::play_inputs` while playing a replay
//! (`dword_5C5CE0` 2).
//!
//! In:  nothing
//! Out: as `play_inputs` leaves the registers when called; no flags read
//!
//! Caller: `drive::replay_step`.

/// The Rust version.
use crate::race::owner::Race;
// Converted: `rc2k_core::drive::replay_frames::plays_inputs`.
pub fn play_inputs_if(race: &Race) -> Result<(), crate::errors::BadInput> {
    if rc2k_core::drive::replay_frames::plays_inputs(race.course.numbers.replay_mode.get()) {
        crate::drive::play_inputs::play_inputs()?;
    }
    Ok(())
}
