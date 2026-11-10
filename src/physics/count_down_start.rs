//! `physics::count_down_start` (0x436AFB): the start's count-down. Unless `g_car` +0xA8
//! (`unk_71A080`) is set, and once the timer +0x14 less 0x320 is not below 0 (by its
//! sign): the count +0x8C (`unk_71A064`), when not 0, goes down by one; at whole
//! hundreds of 8 or less (`div` by 100, ebx 100) `physics::set_flag_5165a0` (the
//! stage sound `once[1]` = `dword_5165A0`: `sound::stage_sounds` plays sound 0x1AC once and clears
//! it, a beep each whole second of the last eight). +0x8C is the car's `time_left`, in hundredths
//! of a second (the arcade race's clock: `race::panel_neighbours` shows it x 10).
//!
//! Caller: `physics::race_timers` (0x4367D4). Refuses what `set_flag_5165a0` refuses.

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
