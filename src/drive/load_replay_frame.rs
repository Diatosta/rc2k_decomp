//! `drive::load_replay_frame` (0x46020A): the replay's frame `step_block` read
//! (`drive::read_replay_frame_0`), its marks (`drive::replay_frame_marks`), and the frame (0xA04
//! dwords at `block`) copied to the replay's starting state `CarsAndReplay.car_blocks.cars[8].frame_head[0]`.
//!
//! Caller: `drive::play_replay_frame`. Refuses what `read_replay_frame` refuses.

use crate::cells::copy_cells;
use crate::errors::Refused;
use crate::race::blocks_store::{REPLAY_STATE_DWORDS, REPLAY_STATE_OFFSET};
use crate::race::owner::Race;
use crate::system::owner::System;

pub fn load_replay_frame(race: &Race, system: &System) -> Result<(), Refused> {
    let replay = &race.replay;
    let frame = replay.replay_frames().step_block.get();
    { crate::drive::read_replay_frame_0::read_replay_frame_0(race, system, frame) }?;
    crate::drive::replay_frame_marks::replay_frame_marks(race)?;
    copy_cells(
        (&race.viewer_buffers().block, 0),
        (replay.cars_and_replay.get(), REPLAY_STATE_OFFSET),
        REPLAY_STATE_DWORDS * 4,
    );
    Ok(())
}
