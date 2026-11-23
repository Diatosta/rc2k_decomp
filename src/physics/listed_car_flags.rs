//! When the current car's entry in `listed_car_sounds` is not -1, the given stage sound flag is set
//! to 1 (the callers pass `StageSounds.once[0]`, `[1]` or `[2]`).
//!
//! Refuses (before anything) a current car past 7 (`listed_car_sounds` holds eight).

use crate::errors::Refused;
use crate::race::frame_store::FrameWorkStore;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::listed_cars::listed_entry;

/// Sets `flag` to 1 when the current car is listed (`FrameWork.listed_car_sounds`).
pub fn set_flag(
    frame_work: &FrameWorkStore,
    work: &RaceWork,
    flag: &mut u32,
) -> Result<(), Refused> {
    let current = work.current_car;
    let (_, listed) = listed_entry(current, |car| {
        frame_work.borrow().listed_car_sounds[car as usize]
    })?;
    if listed {
        *flag = 1;
    }
    Ok(())
}
