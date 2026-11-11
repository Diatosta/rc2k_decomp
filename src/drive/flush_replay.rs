//! `drive::flush_replay` (0x46022D): the replay's starting state (0xA04 dwords at `unk_7172EC`)
//! copied into the frame `block` and the frames written (`drive::write_replay_frames`).
//!
//! Caller: `drive::flush_replay_if`. Refuses what `write_replay_frames` refuses.

use crate::errors::Refused;
use crate::pointers::copy_cells;
use crate::race::blocks_store::REPLAY_STATE_OFFSET;
use crate::race::owner::Race;
use crate::race::viewer_buffers_store::VIEWER_BUFFERS;

const DWORDS: usize = 0xA04;

pub fn flush_replay(race: &Race) -> Result<(), Refused> {
    copy_cells(
        (&race.replay.cars_and_replay, REPLAY_STATE_OFFSET),
        (&VIEWER_BUFFERS.block, 0),
        DWORDS * 4,
    );
    crate::drive::write_replay_frames::write_replay_frames(race)
}
