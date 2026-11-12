//! `drive::load_replay_cars` (0x46010D): `drive::save_replay_cars` the other way: the replay's
//! starting state in the replay's staging (`race::replay_staging_store`) put back into the four car
//! blocks, the race slots and `replay_extra`'s run, field by field (`drive::replay_state`).
//!
//! Callers: `drive::play_replay_frame`, `race::show_replay_still` (twice).

use crate::drive::replay_state::scatter;
use crate::race::owner::Race;

pub fn load_replay_cars(race: &Race) {
    let frame = race.replay.staging().start_frame();
    scatter(&frame, &race.car_blocks().cars, race.shared.work.get());
}
