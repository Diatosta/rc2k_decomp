//! `race::reset_marks` (0x41E403): after an edit, the block shown held to the replay's blocks
//! (`replay_block` at most `index_block_count`, signed), the marks round the whole replay
//! (`mark_in` 0, `mark_out` the count) and the block shown read again
//! (`drive::read_replay_frame_0`).
//!
//! Callers: `race::viewer_action` (the cut and the paste), `race::undo_edit`. Refuses what
//! `read_replay_frame_0` refuses (what it wrote put back).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;

pub fn reset_marks(race: &Race, system: &System) -> Result<(), Refused> {
    let replay = &race.replay;
    {
        // Converted: `rc2k_core::race::viewer::reset_marks`.
        let (block, mark_in, mark_out) = rc2k_core::race::viewer::reset_marks(
            replay.replay_view().replay_block.get(),
            race.shared.replay_file().index_block_count.get(),
        );
        replay.replay_view().replay_block.set(block);
        race.viewer_buffers().mark_in.set(mark_in);
        race.viewer_buffers().mark_out.set(mark_out);
        {
            crate::drive::read_replay_frame_0::read_replay_frame_0(
                race,
                system,
                replay.replay_view().replay_block.get(),
            )
        }?;
        Ok(())
    }
}
