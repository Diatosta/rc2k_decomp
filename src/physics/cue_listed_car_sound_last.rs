//! `physics::cue_listed_car_sound_last` (0x45D1D3): `physics::cue_listed_car_sound`'s twin
//! for the pace notes: eax the current car's entry in `dword_641680`; unless -1,
//! `physics::cue_car_sound_last` with it.
//!
//! In:  ebx the cue
//! Out: eax the entry; the rest kept; no flags read
//!
//! Caller: `physics::queue_pace_note` (inside `pusha` / `popa`). Refuses (before anything)
//! a current car past 7.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, rva};
use rc2k_core::physics::listed_cars::listed_entry;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

/// `cue` for the listed car, if it is listed: its entry returned (the original's eax).
// Converted: `rc2k_core::physics::listed_cars::listed_entry`.
pub fn cue_listed_car_sound_last(cue: u32) -> Result<u32, Refused> {
    let (entry, listed) = listed_entry(g(0x71B3F8), |car| g(0x641680 + 4 * car as usize))?;
    if listed {
        crate::physics::cue_car_sound_last::cue_car_sound_last(entry, cue)?;
    }
    Ok(entry)
}
