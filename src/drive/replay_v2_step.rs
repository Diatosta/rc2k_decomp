//! After a car's step in a replay of the own format (`rc2k_core::race::replay_v2`): while
//! recording, a car this machine did not drive has its motion kept for the step; while playing,
//! the motion kept is put into it, so that the car is where the recording showed it whatever
//! the physics made of the step.

use crate::drive::replay_state::{gather_car, scatter_state};
use crate::errors::BadInput;
use crate::race::owner::Race;
use rc2k_core::drive::replay_frames::{PLAYING, RECORDING};
use rc2k_core::race::replay_v2::{CarState, MAX_CARS, STEPS};

pub fn after_step(race: &Race) -> Result<(), BadInput> {
    if !race.viewer_buffers().v2.is_set() {
        return Ok(());
    }
    let mode = race.course.replay_mode();
    if mode != RECORDING && mode != PLAYING {
        return Ok(());
    }
    let car = race.shared.work().current_car as usize;
    if car >= MAX_CARS {
        return Ok(());
    }
    let mut v2 = race.replay.v2().lock();
    if v2.state_mask() >> car & 1 == 0 {
        return Ok(());
    }
    let Some(block) = race.car_blocks().cars.get(car) else {
        return Ok(());
    };
    if mode == RECORDING {
        let step = race.replay.replay_frames().tenth as usize;
        if step >= STEPS {
            return Err(BadInput("a replay step past the second's hundred"));
        }
        let state = CarState::of(&gather_car(block));
        if v2.tail.states.len() <= step {
            v2.tail.states.resize(step + 1, [None; MAX_CARS]);
        }
        v2.tail.states[step][car] = Some(state);
    } else {
        let step = race.replay.replay_frames().step_frame as usize;
        if let Some(state) = v2.tail.states.get(step).and_then(|tick| tick[car]) {
            scatter_state(&state, block);
        }
    }
    Ok(())
}
