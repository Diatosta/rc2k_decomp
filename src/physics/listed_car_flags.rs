//! Three twins: when the current car's entry in `listed_car_sounds`
//! (indexed by `g_currentCar`) is not -1, a flag is set to 1:
//! `physics::set_flag_51659C` (0x45D151, `dword_51659C`), `physics::set_flag_5165A0`
//! (0x45D16D, `dword_5165A0`), `physics::set_flag_5165A4` (0x45D189, `dword_5165A4`).
//!
//! Callers: `physics::take_split_counted`, `physics::count_down_start`, `physics::start_go_notice`
//! (under `physics::race_timers`). Refuses (before anything) a current car past 7 (`listed_car_sounds` holds eight).

use crate::errors::Refused;
use crate::race::frame_store::FrameWork;
use crate::race::work_store::RaceWork;
use crate::session::Dword;
use rc2k_core::physics::listed_cars::listed_entry;

/// Sets `flag` to 1 when the current car is listed (`FRAME_WORK.listed_car_sounds`).
pub fn set_flag(frame_work: &FrameWork, work: &RaceWork, flag: &Dword) -> Result<(), Refused> {
    let current = work.current_car.get();
    let (_, listed) = listed_entry(current, |car| {
        frame_work.listed_car_sounds[car as usize].get()
    })?;
    if listed {
        flag.set(1);
    }
    Ok(())
}
