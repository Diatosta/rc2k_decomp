//! The replay's starting state (the replay's staging, `race::replay_staging_store`) copied into
//! the frame `block` and the frames written (`drive::write_replay_frames`).
//!
//! Refuses what `write_replay_frames` refuses.

use crate::errors::Refused;
use crate::memory::cells::copy_cells;
use crate::race::owner::Race;
use crate::system::owner::System;
use rc2k_core::drive::replay_frames::FRAME_BYTES;

pub fn flush_replay(
    race: &Race,
    system: &System,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    copy_cells(
        (race.replay.staging(), 0),
        (&race.viewer_buffers().block, 0),
        FRAME_BYTES,
    );
    crate::drive::write_replay_frames::write_replay_frames(race, system, net)
}
