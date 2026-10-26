//! `drive::read_replay_frame_0` (0x41EBA1): `drive::read_replay_frame` of frame eax at
//! tenth 0 (ebx kept round it).
//!
//! In:  eax the frame
//! Out: CF as `read_replay_frame` leaves it; every register kept
//!
//! Callers: `drive::load_replay_frame`, `sub_41DE81` (six).

use crate::errors::Refused;

/// The Rust version: CF.
pub fn read_replay_frame_0(frame: u32) -> Result<bool, Refused> {
    crate::drive::read_replay_frame::read_replay_frame(frame, 0)
}
