//! `sound::car_cue_sound`: a car's second one-off sound (`cue_sound`, which the physics' cues set:
//! `physics::sound_cue_e5`, `physics::place_sound_e5`).
//!
//! Takes the car's sound object and the request. Nothing when the sound is -2 (none); else it is
//! taken (set to -2) into the request, with its volume and the sample's own rate through
//! `sound::sample_settings`. A volume of 0 stops the sound (`sound::stop_sound`); else the volume
//! and rate go into the request as floats, with the place, and `sound::play_placed_sound` plays it.
//! One caller, `sound::car_sounds`. Refuses (before anything) a null object and a sound past the
//! sample records.

use crate::errors::Refused;
use crate::sound::sample_settings::{record, sample_settings};
use crate::sound::sets_store::SoundRequest;

/// The logic is `rc2k_core::sound::cars::one_off`.
///
/// The waiting cue sound of object `so`, through `request`.
pub fn car_cue_sound(
    so: &crate::sound::store::SoundObject,
    request: &SoundRequest,
    owner: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{OneOff, from_int, one_off};
    let mut waiting = so.cue_sound.get();
    let taken = one_off(
        &mut waiting,
        so.cue_volume.get(),
        LEAST,
        STOP_QUIET,
        |sample| record(audio, sample).map(|_| ()),
        |sample, volume, rate| sample_settings(audio, sample, volume, rate),
    )?;
    let (sound, turn) = taken;
    if turn == OneOff::Nothing {
        return Ok(());
    }
    so.cue_sound.set(waiting);
    request.sound.set(sound);
    match turn {
        OneOff::Nothing | OneOff::Quiet { .. } => Ok(()),
        OneOff::Stop { .. } => crate::sound::turn_parts::stop(request, audio),
        OneOff::Play { volume, rate } => {
            request.volume.set(from_int(volume as i32));
            request.pitch.set(from_int(rate as i32));
            for (cell, place) in request.place.iter().zip(&so.cue_place) {
                cell.set(place.get());
            }
            crate::sound::turn_parts::placed(request, owner, audio)
        }
    }
}

const LEAST: u32 = 1;
const STOP_QUIET: bool = true;
