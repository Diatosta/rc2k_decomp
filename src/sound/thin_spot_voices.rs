//! `sound::thin_spot_voices` (0x40E9F8): of the voices heard with one sound, the nearest
//! two found and one of them, by a random bit, kept heard; unless one is playing.
//!
//! In:  eax the sound
//! Out: ebp the sound; the others as the original leaves them (see the code); esi, edi
//!      preserved
//! The pickers' work cleared. For each voice heard with the sound: one already playing
//! ends it at once (nothing more changes). Else its distance x 1000 (`fistp`) against
//! the nearest's and the next's (unsigned): a new nearest moves the old one to next and
//! stays heard; a new next, and every other, is silenced (+4 0). A nearest displaced
//! stays heard (the code that would silence it, 0x40EA9B, is never reached: notes'
//! bugs). With a next found: `math::next_random` & 1 for the nearest's +4, the other bit
//! for the next's. Twelve callers, `sound::thin_spot_voices_all`.
//! Shadow mode: both (nothing sent; the random state put back between), compared.
//! Self-test: a copy whose call goes to a stand-in (the test's random numbers), against
//! the Rust version, on the game's voices (put back).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::service::harness::{Region, at, capture, restore};
use crate::sound::merge_spot_voices::{BEST, BEST_D, NEXT, NEXT_D, clear_work, distance_1000};
use crate::sound::spot_voices::{
    VOICE_COUNT, VOICE_LEN, fill_voices, g, p, set, voice, voice_regions, vw,
};
use crate::sound::turn_parts::{Part, RANDOM, both, copy_of, dispatch_turn, run_copy};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::thin_spot_voices", rva(0x40E9F8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0xFB;
const OUTS: [usize; 1] = [0xE9];
const ENTRY: [u8; 10] = [0xC7, 0x05, 0x40, 0x7F, 0x51, 0x00, 0x00, 0x00, 0x00, 0x00];

// ---- Rust version ----

pub fn thin_spot_voices_in(c: &mut Regs, part: Part) -> Result<(), Refused> {
    clear_work();
    c.ebp = c.eax;
    for n in 0..VOICE_COUNT {
        let v = voice(n);
        (c.ebx, c.ecx) = (v, VOICE_COUNT - n);
        if vw(v, 4) != 1 || c.ebp != vw(v, 0x14) {
            continue;
        }
        if vw(v, 0x28) == 1 {
            return Ok(());
        }
        c.eax = distance_1000(v);
        if c.eax < g(BEST_D) {
            p(NEXT_D, g(BEST_D));
            c.edx = g(NEXT);
            p(NEXT, g(BEST));
            p(BEST_D, c.eax);
            p(BEST, v);
            continue;
        }
        if c.eax < g(NEXT_D) {
            p(NEXT_D, c.eax);
            p(NEXT, v);
        }
        set(v, 4, 0);
    }
    (c.ebx, c.ecx) = (voice(0) + VOICE_COUNT * VOICE_LEN, 0);
    if g(NEXT) == 0 {
        return Ok(());
    }
    (c.ebx, c.edx) = (g(BEST), g(NEXT));
    part(RANDOM, c)?;
    c.eax &= 1;
    set(c.ebx, 4, c.eax);
    c.eax ^= 1;
    set(c.edx, 4, c.eax);
    Ok(())
}

// ---- hook ----

pub fn thin_regions(_: &Regs) -> Vec<Region> {
    let mut all = voice_regions();
    all.push(at(crate::math::next_random::STATE_RVA));
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(
        &HOOK,
        r,
        call_original,
        thin_spot_voices_in,
        Some(&thin_regions),
    );
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40E9F8, LEN, &ENTRY, &OUTS, &[RANDOM]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let all = thin_regions(&Regs::SENTINEL);
    let saved = capture(&all);
    for seed in 0..6u32 {
        for sound in [0x1B1u32, 0x1B2] {
            let fill = || {
                fill_voices(seed, &|n, v| {
                    let k = n.wrapping_add(seed * 7);
                    set(v, 4, [1, 0, 1, 1][k as usize % 4]);
                    set(v, 0x14, [0x1B1, 0x1B2, 0x1B1][k as usize % 3]);
                    set(v, 0x28, if seed == 4 && n == 40 { 1 } else { 0 });
                    set(
                        v,
                        0x40,
                        0x3F80_0000 + (k.wrapping_mul(0x2_7183) & 0x7F_FFFF),
                    );
                });
            };
            both(
                &mut t,
                &format!("voices {seed}, sound {sound:#x}"),
                &run,
                Regs {
                    eax: sound,
                    ..Regs::SENTINEL
                },
                &fill,
                &all,
                thin_spot_voices_in,
                false,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
