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
//! Shadow mode runs the original alone (it sends a sound).
//! Self-test: a copy whose calls go to stand-ins (the one of `sample_settings` left),
//! against the Rust version, on an object of the test's, real samples.
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
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    let q = request as *mut u32;
    let mut waiting = w(WAITING);
    let taken = one_off(
        &mut waiting,
        w(WAITING + 4),
        LEAST,
        STOP_QUIET,
        |sound| record(sound).map(|_| ()),
        sample_settings,
    )?;
    let (sound, turn) = taken;
    if turn == OneOff::Nothing {
        return Ok(());
    }
    unsafe {
        ((o + WAITING) as *mut u32).write_unaligned(waiting);
        q.write_unaligned(sound);
    }
    match turn {
        OneOff::Nothing | OneOff::Quiet { .. } => Ok(()),
        OneOff::Stop { .. } => crate::sound::turn_parts::stop(request),
        OneOff::Play { volume, rate } => {
            unsafe {
                q.add(2).write_unaligned(from_int(volume as i32));
                q.add(1).write_unaligned(from_int(rate as i32));
                for k in 0..3 {
                    q.add(3 + k as usize)
                        .write_unaligned(w(WAITING + 8 + 4 * k));
                }
            }
            crate::sound::turn_parts::placed(request)
        }
    }
}

/// The sound object's waiting sound (its volume after it, its place after that).
const WAITING: u32 = 0x24C;
const LEAST: u32 = 1;
const STOP_QUIET: bool = true;
