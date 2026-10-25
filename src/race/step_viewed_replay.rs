//! `race::step_viewed_replay` (0x41EC47): the viewed replay one frame on while it plays:
//! the frame (`dword_5C2534`) one more; past 99 the next block from frame 0, which past
//! the end of the play (`dword_5C2538`) stops it, and is read
//! (`drive::read_replay_frame`). Not open (`dword_5BEC88`), at or past the last block, or
//! the read failing: `dword_5BEC90` 1 and carry. Else the block and frame stored and ebx
//! the frame's inputs (`dword_9613A4 + 0x2000 + frame * 16`).
//!
//! In:  nothing
//! Out: carry set when it did not go on; ebx the frame's inputs (on the ways that stop,
//!      the frame counted so far); ZF set at a block's first frame; eax kept (pushed)
//!
//! Caller: `race::replay_viewer` (each frame played; it reads carry). Refuses what
//! `read_replay_frame` refuses (the fatal errors of a block past 0x708).

use crate::errors::Refused;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::viewer_state_store::REPLAY_VIEW;

/// The Rust version: whether it stopped (the carry the original leaves; it leaves ebx and
/// the zero flag too, and no caller reads them).
pub fn step_viewed_replay() -> Result<bool, Refused> {
    let stop = |at_end: bool| {
        if at_end {
            REPLAY_FILE.replay_at_end.set(1);
        }
        true
    };
    // Converted: `rc2k_core::race::viewer::step`.
    use rc2k_core::race::viewer::{Step, step};
    let open = REPLAY_FILE.replay_open.get() == 1;
    let (block, frame) = (
        REPLAY_VIEW.replay_block.get(),
        REPLAY_VIEW.replay_frame.get(),
    );
    let (block, frame) = match step(
        open,
        block,
        frame,
        REPLAY_FILE.index_block_count.get(),
        REPLAY_VIEW.play_end.get(),
    ) {
        Step::Closed | Step::AtLast { .. } => return Ok(stop(true)),
        Step::PlayEnd { .. } => return Ok(stop(false)),
        Step::On { block, frame, read } => {
            if read {
                let failed =
                    { unsafe { crate::drive::read_replay_frame::read_replay_frame(block, 0) } }?;
                if failed {
                    return Ok(stop(true));
                }
            }
            (block, frame)
        }
    };
    REPLAY_VIEW.replay_block.set(block);
    REPLAY_VIEW.replay_frame.set(frame);
    Ok(false)
}
