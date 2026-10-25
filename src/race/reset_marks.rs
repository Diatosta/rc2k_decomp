//! `race::reset_marks` (0x41E403): after an edit, the block shown held to the replay's
//! blocks (`dword_5C2530` at most `dword_5BECD0`, signed), the marks round the whole
//! replay (`dword_96E478` 0, `dword_96E47C` the count) and the block shown read again
//! (`drive::read_replay_frame_0`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::viewer_action` (the cut and the paste), `race::undo_edit`. Refuses what
//! `read_replay_frame_0` refuses (what it wrote put back).

use crate::errors::Refused;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::viewer::{MARK_IN, MARK_OUT, put};
use crate::race::viewer_state_store::REPLAY_VIEW;

/// The Rust version.
pub fn reset_marks() -> Result<(), Refused> {
    {
        // Converted: `rc2k_core::race::viewer::reset_marks`.
        let (block, mark_in, mark_out) = rc2k_core::race::viewer::reset_marks(
            REPLAY_VIEW.replay_block.get(),
            REPLAY_FILE.index_block_count.get(),
        );
        REPLAY_VIEW.replay_block.set(block);
        put(MARK_IN, mark_in);
        put(MARK_OUT, mark_out);
        unsafe {
            crate::drive::read_replay_frame_0::read_replay_frame_0(REPLAY_VIEW.replay_block.get())
        }?;
        Ok(())
    }
}
