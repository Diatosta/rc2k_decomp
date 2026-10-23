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
/// The waiting hit sound of object `o`, through `request`.
pub fn car_hit_sound(o: u32, request: u32) -> Result<(), Refused> {
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
const WAITING: u32 = 0x238;
const LEAST: u32 = 4;
const STOP_QUIET: bool = false;
