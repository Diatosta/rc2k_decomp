//! `sound::car_cue_sound` (0x40E1E5): a car's second one-off sound (+0x24C, which the
//! physics' cues set: `physics::sound_cue_e5`, `physics::place_sound_e5`).
//!
//! In:  ebx the car's sound object, esi the request
//! Out: every register preserved (`pusha` / `popa`)
//! Nothing when +0x24C is -2 (none); else it is taken (+0x24C = -2) into the
//! request, with its volume +0x250 and the sample's own rate through
//! `sound::sample_settings`; a volume of 0: `sound::stop_sound`; else the request's +8 and +4 the volume and rate
//! as floats, the place +0x254 .. +0x25C, and `sound::play_placed_sound`.
//! One caller, `sound::car_sounds`.
//! Refuses (before anything) a null object and a sound past the sample
//! records.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::sound::sample_settings::{record, sample_settings};
use crate::sound::sets_store::SoundRequest;

// ---- Rust version ----

/// Converted: `rc2k_core::sound::cars::one_off`.
///
/// The waiting cue sound of object `o`, through `request`.
pub fn car_cue_sound(o: u32, request: &SoundRequest) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{OneOff, from_int, one_off};
    if o == 0 {
        return Err(BadInput("no object").into());
    }
    let so = crate::sound::store::SoundObject::at(o)?;
    let mut waiting = so.cue_sound.get();
    let taken = one_off(
        &mut waiting,
        so.cue_volume.get(),
        LEAST,
        STOP_QUIET,
        |sound| record(sound).map(|_| ()),
        sample_settings,
    )?;
    let (sound, turn) = taken;
    if turn == OneOff::Nothing {
        return Ok(());
    }
    so.cue_sound.set(waiting);
    request.sound.set(sound);
    match turn {
        OneOff::Nothing | OneOff::Quiet { .. } => Ok(()),
        OneOff::Stop { .. } => crate::sound::turn_parts::stop(request),
        OneOff::Play { volume, rate } => {
            request.volume.set(from_int(volume as i32));
            request.pitch.set(from_int(rate as i32));
            for (cell, place) in request.place.iter().zip(&so.cue_place) {
                cell.set(place.bits());
            }
            crate::sound::turn_parts::placed(request)
        }
    }
}

const LEAST: u32 = 1;
const STOP_QUIET: bool = true;
