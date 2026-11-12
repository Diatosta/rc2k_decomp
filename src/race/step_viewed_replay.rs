//! `race::step_viewed_replay` (0x41EC47): the viewed replay one frame on while it plays: the frame
//! (`replay_frame`) one more; past 99 the next block from frame 0, which past the end of the play
//! (`play_end`) stops it, and is read (`drive::read_replay_frame`). Not open (`replay_open`),
//! at or past the last block, or the read failing: `replay_at_end` 1 and carry. Else the block and
//! frame stored and the frame's inputs located (`block` + 0x2000 + frame * 16`).
//!
//! Caller: `race::replay_viewer` (each frame played; it reads carry). Refuses what
//! `read_replay_frame` refuses (the fatal errors of a block past 0x708).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;

/// Whether it stopped.
pub fn step_viewed_replay(race: &Race, system: &System) -> Result<bool, Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    let stop = |at_end: bool| {
        if at_end {
            shared.replay_file().replay_at_end.set(true);
        }
        true
    };
    // Converted: `rc2k_core::race::viewer::step`.
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
        shared.replay_file().index_block_count.get(),
        replay.replay_view().play_end.get(),
    ) {
        Step::Closed | Step::AtLast { .. } => return Ok(stop(true)),
        Step::PlayEnd { .. } => return Ok(stop(false)),
        Step::On { block, frame, read } => {
            if read {
                let failed =
                    crate::drive::read_replay_frame::read_replay_frame(race, system, block, 0)?;
                if failed {
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
