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
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::sound::sample_settings::{record, sample_settings};
use crate::sound::turn_parts::{PLACED, Part, turn_entry};

// ---- Rust version ----

/// Converted: `rc2k_core::sound::cars::one_off`.
pub fn car_hit_sound_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{OneOff, from_int, one_off};
    if r.ebx == 0 || r.esi == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let o = r.ebx;
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    let q = r.esi as *mut u32;
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
    let mut c = Regs { eax: sound, ..*r };
    match turn {
        OneOff::Nothing | OneOff::Quiet { .. } => Ok(()),
        OneOff::Stop { volume, rate } => {
            (c.ecx, c.edx) = (volume, rate);
            part(crate::sound::turn_parts::STOP, &mut c)
        }
        OneOff::Play { volume, rate } => {
            (c.ecx, c.edx) = (volume, rate);
            unsafe {
                q.add(2).write_unaligned(from_int(volume as i32));
                q.add(1).write_unaligned(from_int(rate as i32));
                for k in 0..3 {
                    c.eax = w(WAITING + 8 + 4 * k);
                    q.add(3 + k as usize).write_unaligned(c.eax);
                }
            }
            part(PLACED, &mut c)
        }
    }
}

/// The sound object's waiting sound (its volume after it, its place after that).
const WAITING: u32 = 0x238;
const LEAST: u32 = 4;
const STOP_QUIET: bool = false;

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, car_hit_sound_in)
}
