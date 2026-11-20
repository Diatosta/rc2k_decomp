//! `sound::car_hit_sound`: a car's sound of a hit, sent once (`hit_sound`, which the physics'
//! contacts set: `physics::place_sound_object` and the like).
//!
//! Takes the car's sound object and the request. Nothing when the sound is -2 (none); else it is
//! taken (set to -2) into the request, with its volume and the sample's own rate through
//! `sound::sample_settings`. A volume under 4 plays nothing; else the volume and rate go into the
//! request as floats, with the place, and `sound::play_placed_sound` plays it. Refuses (before
//! anything) a null object and a sound past the sample records.

use crate::errors::Refused;
use crate::sound::sample_settings::{record, sample_settings};
use crate::sound::sets_store::SoundRequest;

/// The logic is `rc2k_core::sound::cars::one_off`.
///
/// The waiting hit sound of object `so`, through `request`.
pub(super) fn car_hit_sound(
    so: &crate::sound::store::SoundObject,
    request: &mut SoundRequest,
    owner: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{OneOff, from_int, one_off};
    let mut waiting = so.hit_sound.get();
    let taken = one_off(
        &mut waiting,
        so.hit_volume.get(),
        LEAST,
        STOP_QUIET,
        |sample| record(audio, sample).map(|_| ()),
        |sample, volume, rate| sample_settings(audio, sample, volume, rate),
    )?;
    let (sound, turn) = taken;
    if turn == OneOff::Nothing {
        return Ok(());
    }
    so.hit_sound.set(waiting);
    request.sound = sound;
    match turn {
        OneOff::Nothing | OneOff::Quiet { .. } => Ok(()),
        OneOff::Stop { .. } => crate::sound::turn_parts::stop(request, audio),
        OneOff::Play { volume, rate } => {
            request.volume = from_int(volume as i32);
            request.pitch = from_int(rate as i32);
            for (to, from) in request.place.iter_mut().zip(&so.hit_place) {
                *to = from.get();
            }
            crate::sound::turn_parts::placed(request, owner, audio)
        }
    }
}

const LEAST: u32 = 4;
const STOP_QUIET: bool = false;
