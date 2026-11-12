//! `drive::flush_replay_if` (0x460031): `drive::flush_replay` while recording (`replay_mode` 1) at
//! the last tenth (`tenth` 0x63).
//!
//! Caller: `drive::replay_step`. Refuses what `flush_replay` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;
use rc2k_core::drive::replay_frames::flush_due;

// Converted: `rc2k_core::drive::replay_frames::flush_due`.
pub fn flush_replay_if(race: &Race, system: &System) -> Result<(), Refused> {
    if flush_due(
        race.course.numbers.replay_mode.get(),
        race.replay.replay_frames().tenth.get(),
    ) {
        return crate::drive::flush_replay::flush_replay(race, system);
    }
    Ok(())
}
