//! `sound::play_spot_voices` (0x40EB9E): each voice heard this turn sent: started when
//! told to, else kept going while it plays, and stopped once its time is up.
//!
//! In:  esi the request
//! Out: every register preserved (`pusha` / `popa`)
//! For each voice heard (+4 1, cleared): the request's sound = +0x14. Told to start
//! (+0x24 1, cleared): its volume and rate (`sound::sample_settings`, the sample's own)
//! as floats into +0x3C / +0x38, the end (+0x34) = `sound::sample_duration` plus the
//! clock plus its delay (+0x30), playing (+0x28) 1. Else only a playing one goes on, its sound
//! with bit 31 set unless the sample loops (record +0x14 is 1). Then the request: +8
//! the volume, +4 the rate, +0x20 / +0x24 the factors (+0x48 / +0x4C), the place, and
//! `sound::play_placed_sound`; once the clock is at or past the end (`js`), the sound
//! stopped (`sound::stop_sound`) and playing 0. One caller, `sound::spot_sounds`.
//! Refuses a null request (before anything); a sound past the sample records and the
//! two leaves' refusals put back what was written (sounds already sent stay sent).
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls go to stand-ins (the two leaves' left), against the
//! Rust version, on the game's voices (put back), real samples.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::math::x87::from_int;
use crate::physics::flow::guard;
use crate::sound::sample_duration::sample_duration;
use crate::sound::sample_settings::{OWN, RECORDS, RECORDS_RVA, sample_settings};
use crate::sound::spot_voices::{VOICE_COUNT, set, voice, voice_regions, vw};
use crate::sound::turn_parts::{CLOCK, PLACED, Part, STOP, common_regions, turn_entry};
use rc2k_core::sound::spot_voices::{Played, REQUEST_FROM, end_time, going_on, played, time_up};

// ---- Rust version ----

pub fn play_spot_voices_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if r.esi == 0 {
        return Err(BadInput("no request").into());
    }
    let q = r.esi as *mut u32;
    let put = |n: usize, v: u32| unsafe { q.add(n).write_unaligned(v) };
    let mut all = voice_regions();
    all.extend(common_regions());
    guard(&all, || {
        let mut c = *r;
        let clock: u32 = unsafe { game_read(rva(CLOCK)) };
        for n in 0..VOICE_COUNT {
            let v = voice(n);
            (c.ebx, c.ebp) = (v, VOICE_COUNT - n);
            if vw(v, 4) != 1 {
                continue;
            }
            set(v, 4, 0);
            c.eax = vw(v, 0x14);
            put(0, c.eax);
            // Converted: `rc2k_core::sound::spot_voices` (`played`, `end_time`,
            // `going_on`, `REQUEST_FROM`, `time_up`).
            match played(vw(v, 0x24), || vw(v, 0x28)) {
                Played::Start => {
                    set(v, 0x24, 0);
                    (c.ecx, c.edx) = sample_settings(c.eax, OWN, OWN)?;
                    set(v, 0x3C, from_int(c.ecx as i32));
                    set(v, 0x38, from_int(c.edx as i32));
                    c.eax = end_time(sample_duration(c.eax, c.edx)?, clock, vw(v, 0x30));
                    set(v, 0x34, c.eax);
                    set(v, 0x28, 1);
                }
                Played::Silent => continue,
                Played::GoOn => {
                    if c.eax >= RECORDS {
                        return Err(BadInput("a sound past the sample records").into());
                    }
                    c.eax = c.eax.wrapping_mul(0x40);
                    let looping: u32 = unsafe { game_read(RECORDS_RVA + c.eax as usize + 0x14) };
                    put(0, going_on(unsafe { q.read_unaligned() }, looping));
                }
            }
            for (to, from) in REQUEST_FROM {
                c.eax = vw(v, from);
                put(to, c.eax);
            }
            part(PLACED, &mut c)?;
            c.eax = clock;
            if time_up(clock, vw(v, 0x34)) {
                c.eax = vw(v, 0x14);
                put(0, c.eax);
                part(STOP, &mut c)?;
                set(v, 0x28, 0);
            }
        }
        Ok(())
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, play_spot_voices_in)
}
