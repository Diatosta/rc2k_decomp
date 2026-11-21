//! The first 0x1A1 dwords of the four car blocks, the race slots (0xF0) and `replay_extra` (0x100)
//! copied in that order to the replay's staging (`race::replay_staging_store`): the replay's
//! starting state. A replay of the own format also keeps the state of cars 4 .. 7 and starts
//! the second's inputs and states afresh (`race::replay_v2_store`).

use crate::drive::replay_state::{gather, gather_tail};
use crate::race::owner::Race;

/// The replay's starting state gathered from the four car blocks, the race slots and
/// `replay_extra`'s run, field by field (`drive::replay_state`), and put in the replay's frame.
pub fn save_replay_cars(race: &Race) {
    let frame = gather(&race.car_blocks().cars, race.shared.work.get());
    race.replay.staging_mut().set_start(&frame);
    if race.viewer_buffers().v2.is_set() {
        let mut v2 = race.replay.v2().lock();
        v2.tail.keyframes = gather_tail(&race.car_blocks().cars);
        v2.tail.tape = [[[0; 2]; 4]; 100];
        v2.tail.states.clear();
    }
}
