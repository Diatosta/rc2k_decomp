//! The replay's frame `step_block` read
//! (`drive::read_replay_frame_0`), its marks (`drive::replay_frame_marks`), and the frame (0xA04
//! dwords at `block`) copied to the replay's staging (`race::replay_staging_store`).
//!
//! Refuses what `read_replay_frame` refuses.

use crate::cells::copy_cells;
use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;
use rc2k_core::drive::replay_frames::FRAME_BYTES;

pub fn load_replay_frame(race: &Race, system: &System) -> Result<(), Refused> {
    let replay = &race.replay;
    let frame = replay.replay_frames().step_block.get();
    { crate::drive::read_replay_frame_0::read_replay_frame_0(race, system, frame) }?;
    crate::drive::replay_frame_marks::replay_frame_marks(race)?;
    copy_cells(
        (&race.viewer_buffers().block, 0),
        (replay.staging(), 0),
        FRAME_BYTES,
    );
    Ok(())
}
