//! `physics::cue_car_sound_last` (0x40ECBB): `physics::cue_car_sound`'s twin for the pace
//! notes: for eax 0 or 1 (unsigned), the sound object of car `g_playerCarA[eax]`
//! (`g_raceSoundObjects`); when its +8 is 1, `physics::queue_sound_cue` with eax 1 (last),
//! edi its +0x298, the cue in ebx.
//!
//! Takes: which, the cue
//!
//! Caller: `physics::cue_listed_car_sound_last`. Refuses (before anything) a car past 7 or
//! a null sound object.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::sound::store::SoundObject;
use rc2k_core::physics::listed_cars::cue_object;

/// `cue` queued last for the car `which` (0 or 1: a player's), if it sounds.
// Converted: `rc2k_core::physics::listed_cars::cue_object`.
pub fn cue_car_sound_last(
    which: u32,
    cue: u32,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let failed = std::cell::Cell::new(None);
    let found = cue_object(
        which,
        |which| sound.race.sounds.player_cars[which as usize].get(),
        |car| car,
        |car| match SoundObject::number(car) {
            Ok(o) => o.in_use.get(),
            Err(e) => {
                failed.set(Some(e));
                0
            }
        },
    )?;
    if let Some(e) = failed.take() {
        return Err(e.into());
    }
    let Some(car) = found else {
        return Ok(());
    };
    crate::physics::queue_sound_cue::queue_sound_cue(false, cue, SoundObject::number(car)?)
}
