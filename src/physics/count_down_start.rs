//! `physics::count_down_start` (0x436AFB): the start's count-down. Unless the car's `finish_flag`
//! is set, and once `run_ticks` less 0x320 is not below 0 (by its sign): the count `time_left`,
//! when not 0, goes down by one; at whole hundreds of 8 or less `physics::set_flag_5165a0`
//! sets the stage sound flag (`sound::stage_sounds` plays sound 0x1AC once and clears it: a beep
//! each whole second of the last eight). `time_left` is in hundredths of a second (the arcade
//! race's clock: `race::panel_neighbours` shows it times 10).
//!
//! Caller: `physics::race_timers`. Refuses what `set_flag_5165a0` refuses.

use crate::errors::Refused;
use crate::physics::listed_car_flags::set_flag;
use crate::race::owner::Race;
use rc2k_core::physics::start::{CarStart, StartCount};

pub fn count_down_start(race: &Race, sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    let mut c = CarStart {
        finished: race.shared.car.finish_flag.get(),
        timer: race.shared.car.run_ticks.get(),
        countdown: race.shared.car.time_left.get(),
        ..CarStart::default()
    };
    match c.count_down_start() {
        StartCount::Stopped | StartCount::Waiting(_) => Ok(()),
        StartCount::Counted { beep, .. } => {
            race.shared.car.time_left.set(c.countdown);
            if beep {
                set_flag(race, &sound.race.stage.once[1])?;
            }
            Ok(())
        }
    }
}
