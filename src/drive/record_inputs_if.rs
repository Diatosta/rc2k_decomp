//! `drive::record_inputs` while recording a replay
//! (`replay_mode` 1) for the cars below `players`.

use crate::race::owner::Race;

pub fn record_inputs_if(race: &Race) -> Result<(), crate::errors::BadInput> {
    let shared = &race.shared;
    let records = rc2k_core::drive::replay_frames::records_inputs(
        race.course.numbers.get().replay_mode.get(),
        shared.work.get().current_car.get(),
        || shared.view.get().players.get(),
    );
    if records == Some(true) {
        crate::drive::record_inputs::record_inputs(race)?;
    }
    Ok(())
}
