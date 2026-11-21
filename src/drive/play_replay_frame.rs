//! While playing a replay (`replay_mode` 2): the replay's frame and tenth (`replay_block`,
//! `replay_frame`) into `step_block`, `step_frame`; at a whole frame the frame read and its state
//! loaded (`drive::load_replay_frame`, `drive::load_replay_cars`); then the cameras
//! (`drive::replay_cameras`).
//!
//! Refuses what its calls refuse.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;

pub(super) fn play_replay_frame(race: &Race, system: &System) -> Result<(), Refused> {
    let Some(whole) = rc2k_core::drive::race_steps::play_frame(race.course.replay_mode(), || {
        race.replay.replay_frames_mut().step_block = race.replay.replay_view().replay_block.get();
        race.replay.replay_frames_mut().step_frame = race.replay.replay_view().replay_frame.get();
        race.replay.replay_frames().step_frame
    }) else {
        return Ok(());
    };
    if whole {
        { crate::drive::load_replay_frame::load_replay_frame(race, system) }?;
        crate::drive::replay_check::compare_frame(race)?;
        crate::drive::load_replay_cars::load_replay_cars(race);
    }
    crate::drive::replay_cameras::replay_cameras(race)
}
