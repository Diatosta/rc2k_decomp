//! `drive::flush_replay_if` (0x460031): `drive::flush_replay` while recording (`dword_5C5CE0` 1) at
//! the last tenth (`dword_6EF7DC` 0x63).
//!
//! Caller: `drive::replay_step`. Refuses what `flush_replay` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::drive::replay_frames::flush_due;

// Converted: `rc2k_core::drive::replay_frames::flush_due`.
pub fn flush_replay_if(race: &Race) -> Result<(), Refused> {
    if flush_due(
        race.course.numbers.replay_mode.get(),
        race.replay.replay_frames.tenth.get(),
    ) {
        return crate::drive::flush_replay::flush_replay(race);
    }
    Ok(())
}
