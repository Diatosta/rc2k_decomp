//! `drive::read_replay_frame_0` (0x41EBA1): `drive::read_replay_frame` of the given frame at tenth 0.
//!
//! Takes: the frame
//!
//! Callers: `drive::load_replay_frame`, the viewer's actions (six).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;

/// CF.
pub fn read_replay_frame_0(race: &Race, system: &System, frame: u32) -> Result<bool, Refused> {
    crate::drive::read_replay_frame::read_replay_frame(race, system, frame, 0)
}
