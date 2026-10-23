//! `drive::read_replay_frame_0` (0x41EBA1): `drive::read_replay_frame` of frame eax at
//! tenth 0 (ebx kept round it).
//!
//! In:  eax the frame
//! Out: CF as `read_replay_frame` leaves it; every register kept
//!
//! Callers: `drive::load_replay_frame`, `sub_41DE81` (six).
//! Self-test: as `read_replay_frame`'s, with its test replay.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;

/// The Rust version: CF.
pub unsafe fn read_replay_frame_0(frame: u32) -> Result<bool, Refused> {
    unsafe { crate::drive::read_replay_frame::read_replay_frame(frame, 0) }
}
