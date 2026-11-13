//! The first 0x1A1 dwords of the four car blocks, the race slots (0xF0) and `replay_extra` (0x100)
//! copied in that order to the replay's staging (`race::replay_staging_store`): the replay's
//! starting state.

use crate::drive::replay_state::gather;
use crate::race::owner::Race;

/// The replay's starting state gathered from the four car blocks, the race slots and
/// `replay_extra`'s run, field by field (`drive::replay_state`), and put in the replay's frame.
pub fn save_replay_cars(race: &Race) {
    let frame = gather(&race.car_blocks().cars, race.shared.work.get());
    race.replay.staging().set_start(&frame);
}
