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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87::from_int;
use crate::physics::flow::guard;
use crate::service::harness::{at, capture, restore};
use crate::sound::sample_duration::sample_duration;
use crate::sound::sample_settings::{OWN, RECORDS, RECORDS_RVA, sample_settings};
use crate::sound::spot_voices::{VOICE_COUNT, fill_voices, set, voice, voice_regions, vw};
use crate::sound::turn_parts::{
    CLOCK, PLACED, Part, REQUEST, STOP, both, common_regions, copy_of, dispatch_turn, run_copy,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::play_spot_voices", rva(0x40EB9E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0xDA;
/// `sample_settings`, `sample_duration` (both left), `play_placed_sound`, `stop_sound`.
const OUTS: [usize; 4] = [0x39, 0x52, 0xAE, 0xC2];
const ENTRY: [u8; 7] = [0x60, 0x8D, 0x1D, 0xC0, 0x68, 0x51, 0x00];

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
            if vw(v, 0x24) == 1 {
                set(v, 0x24, 0);
                (c.ecx, c.edx) = sample_settings(c.eax, OWN, OWN)?;
                set(v, 0x3C, from_int(c.ecx as i32));
                set(v, 0x38, from_int(c.edx as i32));
                c.eax = sample_duration(c.eax, c.edx)?
                    .wrapping_add(clock)
                    .wrapping_add(vw(v, 0x30));
                set(v, 0x34, c.eax);
                set(v, 0x28, 1);
            } else {
                if vw(v, 0x28) != 1 {
                    continue;
                }
                if c.eax >= RECORDS {
                    return Err(BadInput("a sound past the sample records").into());
                }
                c.eax = c.eax.wrapping_mul(0x40);
                let looping: u32 = unsafe { game_read(RECORDS_RVA + c.eax as usize + 0x14) };
                if looping != 1 {
                    put(0, unsafe { q.read_unaligned() } | 0x8000_0000);
                }
            }
            for (to, from) in [
                (2, 0x3C),
                (1, 0x38),
                (8, 0x48),
                (9, 0x4C),
                (3, 8),
                (4, 0xC),
                (5, 0x10),
            ] {
                c.eax = vw(v, from);
                put(to, c.eax);
            }
            part(PLACED, &mut c)?;
            c.eax = clock;
            if (clock.wrapping_sub(vw(v, 0x34)) as i32) >= 0 {
                c.eax = vw(v, 0x14);
                put(0, c.eax);
                part(STOP, &mut c)?;
                set(v, 0x28, 0);
            }
        }
        Ok(())
    })
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, play_spot_voices_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40EB9E, LEN, &ENTRY, &OUTS, &[PLACED, STOP]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let mut all = voice_regions();
    all.extend(common_regions());
    all.push(at(rva(CLOCK)));
    let saved = capture(&all);
    for seed in 0..5u32 {
        let fill = || {
            fill_voices(seed, &|n, v| {
                let k = n.wrapping_add(seed * 13);
                set(v, 4, [1, 0, 1, 1][k as usize % 4]);
                set(v, 0x24, [1, 0, 0][k as usize % 3]);
                set(v, 0x28, [1, 0, 1, 1, 0][k as usize % 5]);
                set(v, 0x14, [0x1B1, 0x40, 0x13F, 0x1C5, 3][k as usize % 5]);
                set(v, 0x30, k * 7);
                set(v, 0x34, [0, 900, 1100, 0x8000_0000][k as usize % 4]);
            });
            unsafe { game_write(rva(CLOCK), 1000u32) };
        };
        both(
            &mut t,
            &format!("voices {seed}"),
            &run,
            Regs {
                esi: game_addr(rva(REQUEST)) as u32,
                ..Regs::SENTINEL
            },
            &fill,
            &all,
            play_spot_voices_in,
            false,
        );
    }
    restore(&all, &saved);
    t.finish()
}
