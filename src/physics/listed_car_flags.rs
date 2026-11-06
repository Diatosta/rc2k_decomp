//! Three twins (IDA 2026-10-17): when the current car's entry in `dword_641680`
//! (indexed by `g_currentCar`) is not -1, a flag is set to 1:
//! `physics::set_flag_51659C` (0x45D151, `dword_51659C`), `physics::set_flag_5165A0`
//! (0x45D16D, `dword_5165A0`), `physics::set_flag_5165A4` (0x45D189, `dword_5165A4`).
//!
//! In:  nothing
//! Out: eax the entry; the rest kept; no flags read (callers go on with other
//!      instructions)
//!
//! Callers: `sub_436833`, `sub_436AFB`, `sub_436A4B` (under `sub_436790`). Refuses (before
//! anything) a current car past 7 (the table's size not worked out).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::session::Dword;
use rc2k_core::physics::listed_cars::listed_entry;

/// Sets `flag` to 1 when the current car is listed (`FRAME_WORK.listed_car_sounds`).
pub fn set_flag(race: &Race, flag: &Dword) -> Result<(), Refused> {
    let current = race.shared.work.current_car.get();
    let (_, listed) = listed_entry(current, |car| {
        race.frame.frame_work.listed_car_sounds[car as usize].get()
    })?;
    if listed {
        flag.set(1);
    }
    Ok(())
}
