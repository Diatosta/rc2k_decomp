//! `drive::read_replay_frame_0` (0x41EBA1): `drive::read_replay_frame` of frame eax at
//! tenth 0 (ebx kept round it).
//!
//! Takes: the frame
//! Out: CF as `read_replay_frame` leaves it; every register kept
//!
//! Callers: `drive::load_replay_frame`, `sub_41DE81` (six).

use crate::errors::Refused;
use crate::race::owner::Race;

/// The Rust version: CF.
pub fn read_replay_frame_0(race: &Race, frame: u32) -> Result<bool, Refused> {
    crate::drive::read_replay_frame::read_replay_frame(race, frame, 0)
}
