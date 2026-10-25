//! `drive::play_replay_frame` (0x460049): while playing a replay (`dword_5C5CE0` 2): the
//! replay's frame and tenth (`dword_5C2530`, `5C2534`) into `dword_6EF7E0`, `6EF7E4`; at
//! a whole frame the frame read and its state loaded (`drive::load_replay_frame`,
//! `drive::load_replay_cars`); then the cameras (`drive::replay_cameras`).
//!
//! In:  nothing
//! Out: as the calls leave the registers; no flags read
//!
//! Caller: `drive::replay_step`. Refuses what its calls refuse.

use crate::errors::Refused;
use crate::physics::flow::{g, p};
use crate::race::numbers_store::NUMBERS;

/// The Rust version.
// Converted: `rc2k_core::drive::race_steps::play_frame`.
pub unsafe fn play_replay_frame() -> Result<(), Refused> {
    let Some(whole) = rc2k_core::drive::race_steps::play_frame(NUMBERS.replay_mode.get(), || {
        p(0x6EF7E0, g(0x5C2530));
        p(0x6EF7E4, g(0x5C2534));
        g(0x6EF7E4)
    }) else {
        return Ok(());
    };
    if whole {
        unsafe { crate::drive::load_replay_frame::load_replay_frame() }?;
        crate::drive::replay_check::compare_frame();
        crate::drive::load_replay_cars::load_replay_cars();
    }
    unsafe { crate::drive::replay_cameras::replay_cameras() }
}
