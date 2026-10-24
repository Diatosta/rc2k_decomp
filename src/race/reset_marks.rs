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
use crate::race::viewer::{BLOCK, BLOCKS, MARK_IN, MARK_OUT, get, put};

/// The Rust version.
pub fn reset_marks() -> Result<(), Refused> {
    {
        // Converted: `rc2k_core::race::viewer::reset_marks`.
        let (block, mark_in, mark_out) =
            rc2k_core::race::viewer::reset_marks(get(BLOCK), get(BLOCKS));
        put(BLOCK, block);
        put(MARK_IN, mark_in);
        put(MARK_OUT, mark_out);
        unsafe { crate::drive::read_replay_frame_0::read_replay_frame_0(get(BLOCK)) }?;
        Ok(())
    }
}
