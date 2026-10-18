//! `sound::merge_spot_voices` (0x40E809): of the voices heard with one sound, the
//! nearest kept and (re)started, moved to the point between it and the next nearest
//! that is nearest the listener; all the others silenced.
//!
//! In:  eax the sound
//! Out: ebp the sound, ecx 0; ebx the nearest (or the end of the voices), eax and edx
//!      as the original leaves them (see the code); esi, edi preserved
//! The pickers' work (`dword_517F40` .. `dword_517F50`) cleared (distances to -1). For
//! each voice heard with the sound: a playing one noted in `dword_517F40`; its distance
//! x 1000 as a whole number (`fistp` through `g_soundScratch`) compared (unsigned) with
//! the nearest's and the next's, which move down; the voice silenced (+4 and +0x28 0).
//! The nearest, if any, gets +0x24 and +4 1; with a next as well, `math::nearest_on_segment`
//! of their places and the listener (0, 0, 0) gives the nearest's new place.
//! Six callers, `sound::merge_spot_voices_all`.
//! Shadow mode: both (nothing sent), the voices and variables compared.
//! Self-test: a copy whose call goes to a stand-in (which runs the Rust
//! `nearest_on_segment`), against the Rust version, on the game's voices (put back).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::physics::flow::{base, x87_run};
use crate::service::harness::{Region, capture, restore};
use crate::sound::spot_voices::{
    VOICE_COUNT, VOICE_LEN, WORK, fill_voices, g, p, set, voice, voice_regions, vw,
};
use crate::sound::turn_parts::{Part, SCRATCH, both, copy_of, dispatch_turn, run_copy};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::merge_spot_voices", rva(0x40E809), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const NEAREST_ON_SEGMENT: u32 = 0x4633DE;
const ENDS: usize = 0x71C384;
const RESULT: usize = 0x71C3A8;
pub const PLAYING: usize = WORK;
pub const BEST_D: usize = WORK + 4;
pub const BEST: usize = WORK + 8;
pub const NEXT_D: usize = WORK + 12;
pub const NEXT: usize = WORK + 16;
const LEN: usize = 0x174;
const OUTS: [usize; 1] = [0x150];
const ENTRY: [u8; 10] = [0xC7, 0x05, 0x40, 0x7F, 0x51, 0x00, 0x00, 0x00, 0x00, 0x00];

/// `mov scratch, 1000.0; fld [v+0x40]; fmul scratch; fistp scratch`.
pub fn distance_1000(v: u32) -> u32 {
    p(SCRATCH, 0x447A_0000);
    x87_run!(
        "fld dword ptr [{bx} + 0x40]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fistp dword ptr [{b} + 0x1183CC]";
        bx = v, b = base()
    );
    g(SCRATCH)
}

/// The pickers' work cleared.
pub fn clear_work() {
    for (at, v) in [
        (PLAYING, 0),
        (BEST_D, u32::MAX),
        (BEST, 0),
        (NEXT_D, u32::MAX),
        (NEXT, 0),
    ] {
        p(at, v);
    }
}

// ---- Rust version ----

pub fn merge_spot_voices_in(c: &mut Regs, part: Part) -> Result<(), Refused> {
    clear_work();
    c.ebp = c.eax;
    for n in 0..VOICE_COUNT {
        let v = voice(n);
        (c.ebx, c.ecx) = (v, VOICE_COUNT - n);
        if vw(v, 4) != 1 || c.ebp != vw(v, 0x14) {
            continue;
        }
        if vw(v, 0x28) == 1 {
            p(PLAYING, v);
        }
        c.eax = distance_1000(v);
        if c.eax < g(BEST_D) {
            p(NEXT_D, g(BEST_D));
            c.edx = g(NEXT);
            p(NEXT, g(BEST));
            p(BEST_D, c.eax);
            p(BEST, v);
        } else if c.eax < g(NEXT_D) {
            p(NEXT_D, c.eax);
            p(NEXT, v);
        }
        set(v, 4, 0);
        set(v, 0x28, 0);
    }
    (c.ebx, c.ecx) = (voice(0) + VOICE_COUNT * VOICE_LEN, 0);
    if g(BEST) == 0 {
        return Ok(());
    }
    c.ebx = g(BEST);
    set(c.ebx, 0x24, 1);
    set(c.ebx, 4, 1);
    if g(NEXT) == 0 {
        return Ok(());
    }
    for (k, from) in [(0usize, g(BEST)), (12, g(NEXT))] {
        for m in 0..3 {
            c.eax = vw(from, 8 + 4 * m as u32);
            p(ENDS + k + 4 * m, c.eax);
        }
    }
    c.ebx = g(NEXT);
    for m in 0..3 {
        p(ENDS + 24 + 4 * m, 0);
    }
    part(NEAREST_ON_SEGMENT, c)?;
    c.ebx = g(BEST);
    for m in 0..3 {
        c.eax = g(RESULT + 4 * m);
        set(c.ebx, 8 + 4 * m as u32, c.eax);
    }
    Ok(())
}

// ---- hook ----

pub fn merge_regions(_: &Regs) -> Vec<Region> {
    let mut all = voice_regions();
    use crate::math::nearest_on_segment::{VARS, VARS_LEN};
    all.push((game_addr(rva(VARS)) as u32, VARS_LEN));
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(
        &HOOK,
        r,
        call_original,
        merge_spot_voices_in,
        Some(&merge_regions),
    );
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40E809, LEN, &ENTRY, &OUTS, &[NEAREST_ON_SEGMENT]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let all = merge_regions(&Regs::SENTINEL);
    let saved = capture(&all);
    for seed in 0..6u32 {
        for sound in [0x1B7u32, 0x1B8] {
            let fill = || {
                fill_voices(seed, &|n, v| {
                    let k = n.wrapping_add(seed * 5);
                    set(v, 4, [1, 0, 1, 1][k as usize % 4]);
                    set(v, 0x14, [0x1B7, 0x1B8, 0x1B7][k as usize % 3]);
                    set(v, 0x28, [0, 1, 0, 0, 0][k as usize % 5]);
                    set(
                        v,
                        0x40,
                        0x3F80_0000 + (k.wrapping_mul(0x3_1415) & 0x7F_FFFF),
                    );
                    if seed == 5 {
                        set(v, 4, 0);
                    }
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
                merge_spot_voices_in,
                false,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
