//! Sets `StageSounds.once[0]` to 1 when the current car's entry in `listed_car_sounds` is not
//! -1. See `physics/listed_car_flags.rs` (the core, the test).
//!
//! Returns the entry.
//!
//! Refuses (before anything) a current car past 7.

use crate::errors::Refused;
use crate::physics::listed_car_flags::set_flag;
use crate::race::frame_store::FrameWork;
use crate::race::work_store::RaceWork;

pub(super) fn flag_current_car_listed(
    frame_work: &FrameWork,
    work: &RaceWork,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    set_flag(
        frame_work,
        work,
        &mut sound.race.stage.get().borrow_mut().once[0],
    )
}
