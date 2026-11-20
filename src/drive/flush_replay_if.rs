//! `drive::flush_replay` while recording (`replay_mode` 1) at
//! the last tenth (`tenth` 0x63).
//!
//! Refuses what `flush_replay` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;
use rc2k_core::drive::replay_frames::flush_due;

pub(super) fn flush_replay_if(
    race: &Race,
    system: &System,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    if flush_due(
        race.course.numbers.get().replay_mode.get(),
        race.replay.replay_frames().tenth.get(),
    ) {
        return crate::drive::flush_replay::flush_replay(race, system, net);
    }
    Ok(())
}
