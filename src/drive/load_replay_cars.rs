//! `drive::save_replay_cars` the other way: the replay's
//! starting state in the replay's staging (`race::replay_staging_store`) put back into the four car
//! blocks, the race slots and `replay_extra`'s run, field by field (`drive::replay_state`); a
//! replay of the own format also puts back cars 4 .. 7, as far as the race has cars.

use crate::drive::replay_state::{scatter, scatter_tail};
use crate::race::owner::Race;

pub fn load_replay_cars(race: &Race) {
    let frame = race.replay.staging().start_frame();
    scatter(&frame, &race.car_blocks().cars, &mut race.shared.work_mut());
    if race.viewer_buffers().v2.is_set() {
        let keys = race.replay.v2().lock().tail.keyframes;
        let count = race.shared.view.get().race_cars.get() as usize;
        scatter_tail(&keys, &race.car_blocks().cars, count);
    }
}
