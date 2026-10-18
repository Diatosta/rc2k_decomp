//! `sound::repeat_spot_voices` (0x40EB44): each voice heard and not playing told when to
//! start again: a random delay of its kind and, while its repeats last, a restart.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! For each voice heard (+4 1) and not playing: its kind's record (`unk_517FAC`, 8 bytes
//! from kind 1: words +2 the most repeats, +4 and +6 the least and most delay): +0x30 =
//! the least + `math::random_below` (most - least) (as words; no call when they are
//! equal); then with no limit (+2 0), or the repeats (+0x2C) below it (counted up), the
//! restart flag +0x24 1. One caller, `sound::spot_sounds`.
//! Refuses (before anything) a voice of kind 0 or past 0x15.
//! Shadow mode: both (nothing sent; the random state put back between), compared.
//! Self-test: a copy whose call goes to a stand-in (the test's random numbers), against
//! the Rust version, on the game's voices (put back).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::service::harness::{Region, capture, restore};
use crate::sound::spot_voices::{MOST_KIND, REPEATS, VOICE_COUNT, fill_voices, set, voice, vw};
use crate::sound::thin_spot_voices::thin_regions;
use crate::sound::turn_parts::{BELOW, Part, both, copy_of, dispatch_turn, run_copy};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::repeat_spot_voices", rva(0x40EB44), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x5A;
const OUTS: [usize; 1] = [0x2F];
const ENTRY: [u8; 7] = [0x60, 0x8D, 0x1D, 0xC0, 0x68, 0x51, 0x00];

fn waiting(v: u32) -> bool {
    vw(v, 4) == 1 && vw(v, 0x28) != 1
}

fn half(at: u32) -> u32 {
    unsafe { (at as *const u16).read_unaligned() as u32 }
}

// ---- Rust version ----

pub fn repeat_spot_voices_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let voices: Vec<u32> = (0..VOICE_COUNT).map(voice).collect();
    if voices
        .iter()
        .any(|&v| waiting(v) && !(1..=MOST_KIND).contains(&vw(v, 0x18)))
    {
        return Err(BadInput("a waiting spot voice of a kind past the table").into());
    }
    let table = game_addr(rva(REPEATS)) as u32;
    let mut c = *r;
    for (n, &v) in voices.iter().enumerate() {
        (c.ebx, c.ecx) = (v, VOICE_COUNT - n as u32);
        if !waiting(v) {
            continue;
        }
        c.edx = table + (vw(v, 0x18) - 1) * 8;
        let least = half(c.edx + 4);
        c.eax = half(c.edx + 6).wrapping_sub(least) & 0xFFFF;
        if c.eax != 0 {
            part(BELOW, &mut c)?;
        }
        c.eax = (c.eax & 0xFFFF_0000) | ((c.eax as u16).wrapping_add(least as u16) as u32);
        set(v, 0x30, c.eax);
        c.eax = half(c.edx + 2);
        if c.eax != 0 {
            if c.eax == vw(v, 0x2C) {
                continue;
            }
            set(v, 0x2C, vw(v, 0x2C).wrapping_add(1));
        }
        set(v, 0x24, 1);
    }
    Ok(())
}

// ---- hook ----

fn regions(r: &Regs) -> Vec<Region> {
    thin_regions(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(
        &HOOK,
        r,
        call_original,
        repeat_spot_voices_in,
        Some(&regions),
    );
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40EB44, LEN, &ENTRY, &OUTS, &[BELOW]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let all = thin_regions(&Regs::SENTINEL);
    let saved = capture(&all);
    for seed in 0..6u32 {
        let fill = || {
            fill_voices(seed, &|n, v| {
                let k = n.wrapping_add(seed * 11);
                set(v, 4, [1, 0, 1][k as usize % 3]);
                set(v, 0x28, [0, 1, 0, 0][k as usize % 4]);
                set(
                    v,
                    0x18,
                    if seed == 5 && n == 9 {
                        0x16
                    } else {
                        1 + k % 0x15
                    },
                );
                set(v, 0x2C, k % 4);
                if seed == 5 && n == 9 {
                    set(v, 4, 1);
                    set(v, 0x28, 0);
                }
            });
        };
        both(
            &mut t,
            &format!("voices {seed}"),
            &run,
            Regs::SENTINEL,
            &fill,
            &all,
            repeat_spot_voices_in,
            seed == 5,
        );
    }
    restore(&all, &saved);
    t.finish()
}
