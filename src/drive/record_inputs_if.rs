//! `drive::record_inputs` while recording a replay (`replay_mode` 1) for the cars below
//! `players`; in a network race, for every car on the tape (the other players' cars too: their
//! input words are what their messages put in the car's block).

use crate::net::owner::Net;
use crate::race::owner::Race;

pub fn record_inputs_if(race: &Race, net: &Net) -> Result<(), crate::errors::BadInput> {
    let shared = &race.shared;
    let records = rc2k_core::drive::replay_frames::records_inputs(
        race.course.numbers.get().replay_mode.get(),
        shared.work.get().current_car.get(),
        || {
            if net.race_slot.is_on() {
                shared.view.get().race_cars.get()
            } else {
                shared.view.get().players.get()
            }
        },
    );
    if records == Some(true) {
        crate::drive::record_inputs::record_inputs(race)?;
    }
    Ok(())
}
