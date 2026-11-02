//! `sound::car_hit_sound` (0x40E182): a car's sound of a hit, sent once (+0x238, which
//! the physics' contacts set: `physics::place_sound_object` and the like).
//!
//! In:  ebx the car's sound object, esi the request
//! Out: every register preserved (`pusha` / `popa`)
//! Nothing when +0x238 is -2 (none); else it is taken (+0x238 = -2) into the
//! request, with its volume +0x23C and the sample's own rate through
//! `sound::sample_settings`; a volume under 4: nothing; else the request's +8 and +4 the volume and rate
//! as floats, the place +0x240 .. +0x248, and `sound::play_placed_sound`.
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
/// The waiting hit sound of object `o`, through `request`.
pub fn car_hit_sound(o: u32, request: &SoundRequest) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{OneOff, from_int, one_off};
    if o == 0 {
        return Err(BadInput("no object").into());
    }
    let so = crate::sound::store::SoundObject::at(o)?;
    let mut waiting = so.hit_sound.get();
    let taken = one_off(
        &mut waiting,
        so.hit_volume.get(),
        LEAST,
        STOP_QUIET,
        |sound| record(sound).map(|_| ()),
        sample_settings,
    )?;
    let (sound, turn) = taken;
    if turn == OneOff::Nothing {
        return Ok(());
    }
    so.hit_sound.set(waiting);
    request.sound.set(sound);
    match turn {
        OneOff::Nothing | OneOff::Quiet { .. } => Ok(()),
        OneOff::Stop { .. } => crate::sound::turn_parts::stop(request),
        OneOff::Play { volume, rate } => {
            request.volume.set(from_int(volume as i32));
            request.pitch.set(from_int(rate as i32));
            for (cell, place) in request.place.iter().zip(&so.hit_place) {
                cell.set(place.bits());
            }
            crate::sound::turn_parts::placed(request)
        }
    }
}

const LEAST: u32 = 4;
const STOP_QUIET: bool = false;
