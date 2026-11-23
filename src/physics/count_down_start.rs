//! The start's count-down. Unless the car's `finish_flag` is set, and once `run_ticks` less 0x320
//! is not below 0: the count `time_left`, when not 0, goes down by one; at whole hundreds of 8 or
//! less `physics::set_flag` sets the stage sound flag (`sound::stage_sounds` plays sound
//! 0x1AC once and clears it: a beep each whole second of the last eight). `time_left` is in
//! hundredths of a second (the arcade race's clock: `race::panel_neighbours` shows it times 10).
//!
//! Refuses what `set_flag` refuses.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::listed_car_flags::set_flag;
use crate::race::frame_store::FrameWorkStore;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::start::{CarStart, StartCount};

pub fn count_down_start(
    frame_work: &FrameWorkStore,
    work: &RaceWork,
    car: &mut Car,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let mut c = CarStart {
        finished: car.finish_flag,
        timer: car.run_ticks,
        countdown: car.time_left,
        ..CarStart::default()
    };
    match c.count_down_start() {
        StartCount::Stopped | StartCount::Waiting(_) => Ok(()),
        StartCount::Counted { beep, .. } => {
            car.time_left = c.countdown;
            if beep {
                set_flag(
                    frame_work,
                    work,
                    &mut sound.race.stage.get().borrow_mut().once[1],
                )?;
            }
            Ok(())
        }
    }
}
