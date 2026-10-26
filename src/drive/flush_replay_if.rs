//! `drive::flush_replay_if` (0x460031): `drive::flush_replay` while recording
//! (`dword_5C5CE0` 1) at the last tenth (`dword_6EF7DC` 0x63).
//!
//! In:  nothing
//! Out: as `flush_replay` leaves the registers when called; no flags read
//!
//! Caller: `drive::replay_step`. Refuses what `flush_replay` refuses.

use crate::drive::replay_frames_store::REPLAY_FRAMES;
use crate::errors::Refused;
use crate::race::numbers_store::NUMBERS;
use rc2k_core::drive::replay_frames::flush_due;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::flush_due`.
pub unsafe fn flush_replay_if() -> Result<(), Refused> {
    if flush_due(NUMBERS.replay_mode.get(), REPLAY_FRAMES.tenth.get()) {
        return unsafe { crate::drive::flush_replay::flush_replay() };
    }
    Ok(())
}
