//! `drive::flush_replay` (0x46022D): the replay's starting state (0xA04 dwords at `CarsAndReplay.car_blocks.cars[8].frame_head[0]`)
//! copied into the frame `block` and the frames written (`drive::write_replay_frames`).
//!
//! Caller: `drive::flush_replay_if`. Refuses what `write_replay_frames` refuses.

use crate::cells::copy_cells;
use crate::errors::Refused;
use crate::race::blocks_store::REPLAY_STATE_OFFSET;
use crate::race::owner::Race;
use crate::system::owner::System;

const DWORDS: usize = 0xA04;

pub fn flush_replay(race: &Race, system: &System) -> Result<(), Refused> {
    copy_cells(
        (&race.replay.cars_and_replay, REPLAY_STATE_OFFSET),
        (&race.viewer_buffers().block, 0),
        DWORDS * 4,
    );
    crate::drive::write_replay_frames::write_replay_frames(race, system)
}
