//! `drive::load_replay_cars` (0x46010D): `drive::save_replay_cars` the other way: the replay's
//! starting state at `CarsAndReplay.car_blocks.cars[8].frame_head[0]` put back into the four car
//! blocks, the race slots and `replay_extra`'s run, field by field (`drive::replay_state`).
//!
//! Callers: `drive::play_replay_frame`, `race::show_replay_still` (twice).

use crate::cells::load_cells;
use crate::drive::replay_state::scatter;
use crate::race::blocks_store::REPLAY_STATE_OFFSET;
use crate::race::owner::Race;
use rc2k_core::drive::replay_frames::{ReplayFrame, START_DWORDS};

pub fn load_replay_cars(race: &Race) {
    let mut bytes = vec![0u8; 4 * START_DWORDS];
    load_cells(
        race.replay.cars_and_replay.get(),
        REPLAY_STATE_OFFSET,
        &mut bytes,
    )
    .expect("the starting state inside the replay's frame");
    let frame = ReplayFrame::from_start_bytes(&bytes).expect("the starting state's size");
    scatter(&frame, &race.car_blocks().cars, race.shared.work.get());
}
