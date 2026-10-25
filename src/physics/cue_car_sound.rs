//! `physics::cue_car_sound` (0x40ECE7): for eax 0 or 1 (unsigned), the sound object of
//! car `dword_514F70[eax]` (`g_raceSoundObjects`); when its +8 is 1,
//! `physics::queue_sound_cue` with eax 0 (first), edi its +0x298, the cue in ebx. All in
//! `pusha` / `popa`.
//!
//! In:  eax which, ebx the cue
//! Out: every register kept; no flags read
//!
//! Caller: `physics::cue_listed_car_sound`. Refuses (before anything) a car past 7 or a
//! null sound object.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use rc2k_core::physics::listed_cars::{QUEUE_OFFSET, cue_object};

/// `cue` queued first for the car `which` (0 or 1: a player's), if it sounds.
// Converted: `rc2k_core::physics::listed_cars::cue_object`.
pub fn cue_car_sound(which: u32, cue: u32) -> Result<(), Refused> {
    let Some(o) = cue_object(
        which,
        |which| crate::sound::store::RACE_SOUNDS.player_cars[which as usize].get(),
        |car| crate::sound::store::RACE_SOUNDS.table[car as usize].get(),
        |o| unsafe { ((o + 8) as *const u32).read_unaligned() },
    )?
    else {
        return Ok(());
    };
    crate::physics::queue_sound_cue::queue_sound_cue(true, cue, o, o + QUEUE_OFFSET)
}
