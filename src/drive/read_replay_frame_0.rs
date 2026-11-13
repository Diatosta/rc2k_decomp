//! `drive::read_replay_frame` of the given frame at tenth 0.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;

pub fn read_replay_frame_0(race: &Race, system: &System, frame: u32) -> Result<bool, Refused> {
    crate::drive::read_replay_frame::read_replay_frame(race, system, frame, 0)
}
