//! `sound::car_cue_sound`: a car's second one-off sound (`cue_sound`, which the physics' cues set:
//! `physics::sound_cue_e5`, `physics::place_sound_e5`).
//!
//! Takes the car's sound object and the request. Nothing when the sound is -2 (none); else it is
//! taken (set to -2) into the request, with its volume and the sample's own rate through
//! `sound::sample_settings`. A volume of 0 stops the sound (`sound::stop_sound`); else the volume
//! and rate go into the request as floats, with the place, and `sound::play_placed_sound` plays it.
//! Refuses (before anything) a null object and a sound past the sample records.

use crate::errors::Refused;
use crate::sound::sample_settings::{record, sample_settings};
use crate::sound::sets_store::SoundRequest;

/// The logic is `rc2k_core::sound::cars::one_off`.
///
/// The waiting cue sound of object `so`, through `request`.
pub(super) fn car_cue_sound(
    so: &crate::sound::store::SoundObjectCell,
    request: &mut SoundRequest,
    owner: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{OneOff, from_int, one_off};
    let mut waiting = so.borrow().cue_sound;
    let taken = one_off(
        &mut waiting,
        so.borrow().cue_volume,
        LEAST,
        STOP_QUIET,
        |sample| record(audio, sample).map(|_| ()),
        |sample, volume, rate| sample_settings(audio, sample, volume, rate),
    )?;
    let (sound, turn) = taken;
    if turn == OneOff::Nothing {
        return Ok(());
    }
    so.borrow_mut().cue_sound = waiting;
    request.sound = sound;
    match turn {
        OneOff::Nothing | OneOff::Quiet { .. } => Ok(()),
        OneOff::Stop { .. } => crate::sound::turn_parts::stop(request, audio),
        OneOff::Play { volume, rate } => {
            request.volume = from_int(volume as i32);
            request.pitch = from_int(rate as i32);
            request.place = so.borrow().cue_place;
            crate::sound::turn_parts::placed(request, owner, audio)
        }
    }
}

const LEAST: u32 = 1;
const STOP_QUIET: bool = true;
