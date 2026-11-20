//! `race::step_viewed_replay`: moves the viewed replay one frame on while it plays. The frame
//! (`replay_frame`) goes up by one; past 99 the next block starts from frame 0, which past the
//! end of the play (`play_end`) stops it, and is read (`drive::read_replay_frame`). A replay that
//! is not open (`replay_open`), a position at or past the last block, or a failed read sets
//! `replay_at_end` and reports a stop. Otherwise the block and frame are stored and the frame's
//! inputs located (`block` + 0x2000 + frame * 16).
//!
//! Refuses what `read_replay_frame` refuses (a block past 0x708).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;

/// Steps one frame; returns whether the replay stopped.
pub(super) fn step_viewed_replay(race: &Race, system: &System) -> Result<bool, Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    let stop = |at_end: bool| {
        if at_end {
            shared.replay_file().replay_at_end.set(true);
        }
        true
    };
    use rc2k_core::race::viewer::{Step, step};
    let open = shared.replay_file().replay_open.is_set();
    let (block, frame) = (
        replay.replay_view().replay_block.get(),
        replay.replay_view().replay_frame.get(),
    );
    let (block, frame) = match step(
        open,
        block,
        frame,
        shared.replay_file().index.block_count.get(),
        replay.replay_view().play_end.get(),
    ) {
        Step::Closed | Step::AtLast { .. } => return Ok(stop(true)),
        Step::PlayEnd { .. } => return Ok(stop(false)),
        Step::On { block, frame, read } => {
            if read {
                // A block that cannot be read, or a place past what a replay holds, is a
                // damaged file: the play stops there and the player is told.
                let failed = match crate::drive::read_replay_frame::read_replay_frame(
                    race, system, block, 0,
                ) {
                    Ok(failed) => failed,
                    Err(Refused::Recoverable(_)) => true,
                    Err(other) => return Err(other),
                };
                if failed {
                    crate::race::dialogs::notice(rc2k_core::strings::own::id::BAD_REPLAY_PLAY, "");
                    return Ok(stop(true));
                }
            }
            (block, frame)
        }
    };
    replay.replay_view().replay_block.set(block);
    replay.replay_view().replay_frame.set(frame);
    Ok(false)
}
