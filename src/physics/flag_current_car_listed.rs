//! `physics::flag_current_car_listed` (0x45D151): `dword_51659C` 1 when the current car's entry in
//! `listed_car_sounds` is not -1. See `physics/listed_car_flags.rs` (the core, the test).
//!
//! Returns: the entry
//!
//! Caller: `physics::take_split_counted`. Refuses (before anything) a current car past 7.

use crate::errors::Refused;
use crate::physics::listed_car_flags::set_flag;
use crate::race::frame_store::FrameWork;
use crate::race::work_store::RaceWork;

pub fn flag_current_car_listed(
    frame_work: &FrameWork,
    work: &RaceWork,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    set_flag(frame_work, work, &sound.race.stage.once[0])
}
