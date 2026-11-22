//! The work car's pedals from the input words. With the keys' bit 0x4000 of `input_keys`: the
//! throttle `throttle` stepped by 0x10 toward 0x100 (`input_throttle` set) or 0, held there (by
//! the sign of the steps); else it is `input_throttle`. The same for the brake `brake` by bit
//! 0x2000 and `input_brake`. The handbrake `handbrake` = `input_handbrake`, 0 when that is 0x100
//! or less and `handbrake_failed` is set.

use crate::race::owner::Race;
use rc2k_core::drive::controls::{BRAKE_KEY, THROTTLE_KEY, handbrake, pedal};

/// One pedal: by the keys' `bit`, stepped from `input`, or set to it.
fn step_pedal(keys: u32, bit: u32, input: u32, at: &mut u32) {
    *at = pedal(keys & bit != 0, input, *at);
}

pub fn pedal_inputs(race: &Race) {
    let inputs = race.shared.run().gear_inputs;
    let mut car = race.shared.car_mut();
    let keys = car.input_keys;
    step_pedal(keys, THROTTLE_KEY, inputs.first, &mut car.throttle);
    step_pedal(keys, BRAKE_KEY, inputs.second, &mut car.brake);
    drop(car);
    let hand = inputs.hold;
    let value = handbrake(hand, || race.shared.car().handbrake_failed);
    race.shared.car_mut().handbrake = value;
}
