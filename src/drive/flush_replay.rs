//! The replay's starting state (the replay's staging, `race::replay_staging_store`) copied into
//! the frame `block` and the frames written (`drive::write_replay_frames`).
//!
//! Refuses what `write_replay_frames` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;

pub fn flush_replay(
    race: &Race,
    system: &System,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    crate::race::replay_staging_store::store_in_block(race);
    crate::drive::write_replay_frames::write_replay_frames(race, system, net)
}
