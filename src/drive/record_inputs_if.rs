//! `drive::record_inputs_if` (0x46008D): `drive::record_inputs` while recording a replay
//! (`dword_5C5CE0` 1) for the cars below `dword_63DB94`.
//!
//! In:  nothing
//! Out: while recording eax `dword_63DB94`, then as `record_inputs` leaves the registers
//!      when called; no flags read
//!
//! Caller: `drive::replay_step` (`sub_460B2F`).

use crate::race::owner::Race;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::records_inputs`.
pub fn record_inputs_if(race: &Race) -> Result<(), crate::errors::BadInput> {
    let records = rc2k_core::drive::replay_frames::records_inputs(
        race.course.numbers.replay_mode.get(),
        crate::race::owner::race().shared.work.current_car.get(),
        || crate::race::owner::race().shared.view.players.get(),
    );
    if records == Some(true) {
        crate::drive::record_inputs::record_inputs(&race.replay)?;
    }
    Ok(())
}
