//! `physics::cue_car_sound_last` (0x40ECBB): `physics::cue_car_sound`'s twin for the pace
//! notes: for eax 0 or 1 (unsigned), the sound object of car `g_playerCarA[eax]`
//! (`g_raceSoundObjects`); when its +8 is 1, `physics::queue_sound_cue` with eax 1 (last),
//! edi its +0x298, the cue in ebx. All in `pusha` / `popa`.
//!
//! In:  eax which, ebx the cue
//! Out: every register kept; no flags read
//!
//! Caller: `physics::cue_listed_car_sound_last`. Refuses (before anything) a car past 7 or
//! a null sound object.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use rc2k_core::physics::listed_cars::{QUEUE_OFFSET, cue_object};

fn g(x: impl crate::store::At) -> u32 {
    unsafe { crate::store::at_read(x) }
}

/// `cue` queued last for the car `which` (0 or 1: a player's), if it sounds.
// Converted: `rc2k_core::physics::listed_cars::cue_object`.
pub fn cue_car_sound_last(which: u32, cue: u32) -> Result<(), Refused> {
    let Some(o) = cue_object(
        which,
        |which| g(crate::sound::store::ObjectsSlot::at(0x514F70) + 4 * which as usize),
        |car| g(crate::sound::store::ObjectsSlot::at(0x512A10) + 4 * car as usize),
        |o| unsafe { ((o + 8) as *const u32).read_unaligned() },
    )?
    else {
        return Ok(());
    };
    crate::physics::queue_sound_cue::queue_sound_cue(false, cue, o, o + QUEUE_OFFSET)
}
