//! `physics::cue_listed_car_sound` (0x45D1BC): eax the current car's entry in
//! `dword_641680`; unless -1, `physics::cue_car_sound` with it.
//!
//! In:  ebx the cue
//! Out: eax the entry; the rest kept; no flags read
//!
//! Caller: `physics::off_line_warnings` (twice). Refuses (before anything) a current car
//! past 7.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::race::owner::race;
use crate::race::owner_frame::Frame;
use rc2k_core::physics::listed_cars::listed_entry;

/// `cue` for the listed car, if it is listed: its entry returned (the original's eax).
// Converted: `rc2k_core::physics::listed_cars::listed_entry`.
pub fn cue_listed_car_sound(frame: &Frame, cue: u32) -> Result<u32, Refused> {
    let (entry, listed) = listed_entry(race().shared.work.current_car.get(), |car| {
        frame.frame_work.listed_car_sounds[car as usize].get()
    })?;
    if listed {
        crate::physics::cue_car_sound::cue_car_sound(entry, cue)?;
    }
    Ok(entry)
}
