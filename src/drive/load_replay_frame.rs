//! `drive::load_replay_frame` (0x46020A): the replay's frame `dword_6EF7E0` read
//! (`drive::read_replay_frame_0`), its marks (`drive::replay_frame_marks`), and the frame (0xA04
//! dwords at `dword_9613A4`) copied to the replay's starting state `unk_7172EC`.
//!
//! Caller: `drive::play_replay_frame`. Refuses what `read_replay_frame` refuses.

use crate::errors::Refused;
use crate::pointers::copy_cells;
use crate::race::blocks_store::REPLAY_STATE_OFFSET;
use crate::race::owner::Race;
use crate::race::viewer_buffers_store::VIEWER_BUFFERS;

const DWORDS: usize = 0xA04;

pub fn load_replay_frame(race: &Race) -> Result<(), Refused> {
    let replay = &race.replay;
    let frame = replay.replay_frames.step_block.get();
    { crate::drive::read_replay_frame_0::read_replay_frame_0(race, frame) }?;
    crate::drive::replay_frame_marks::replay_frame_marks(race)?;
    copy_cells(
        (&VIEWER_BUFFERS.block, 0),
        (&replay.cars_and_replay, REPLAY_STATE_OFFSET),
        DWORDS * 4,
    );
    Ok(())
}
