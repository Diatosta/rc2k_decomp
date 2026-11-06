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
use crate::race::owner::Race;

/// The Rust version.
// Converted: `rc2k_core::drive::race_steps::play_frame`.
pub fn play_replay_frame(race: &Race) -> Result<(), Refused> {
    let Some(whole) =
        rc2k_core::drive::race_steps::play_frame(race.course.numbers.replay_mode.get(), || {
            race.replay
                .replay_frames
                .step_block
                .set(race.replay.replay_view.replay_block.get());
            race.replay
                .replay_frames
                .step_frame
                .set(race.replay.replay_view.replay_frame.get());
            race.replay.replay_frames.step_frame.get()
        })
    else {
        return Ok(());
    };
    if whole {
        { crate::drive::load_replay_frame::load_replay_frame(&race.replay) }?;
        crate::drive::replay_check::compare_frame(race)?;
        crate::drive::load_replay_cars::load_replay_cars();
    }
    crate::drive::replay_cameras::replay_cameras()
}
