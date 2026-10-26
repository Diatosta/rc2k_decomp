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
//! Refuses (before anything) a null object or request and a sound past the sample
//! records.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::sound::sample_settings::{record, sample_settings};

// ---- Rust version ----

/// Converted: `rc2k_core::sound::cars::one_off`.
///
/// The waiting cue sound of object `o`, through `request`.
pub fn car_cue_sound(o: u32, request: u32) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{OneOff, from_int, one_off};
    if o == 0 || request == 0 {
        return Err(BadInput("no object or no request").into());
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
    crate::sound::turn_parts::put(request, 0, sound)?;
    match turn {
        OneOff::Nothing | OneOff::Quiet { .. } => Ok(()),
        OneOff::Stop { .. } => crate::sound::turn_parts::stop(request),
        OneOff::Play { volume, rate } => {
            crate::sound::turn_parts::put(request, 2, from_int(volume as i32))?;
            crate::sound::turn_parts::put(request, 1, from_int(rate as i32))?;
            for k in 0..3usize {
                crate::sound::turn_parts::put(request, 3 + k, so.cue_place[k].bits())?;
            }
            crate::sound::turn_parts::placed(request)
        }
    }
}

const LEAST: u32 = 1;
const STOP_QUIET: bool = true;
