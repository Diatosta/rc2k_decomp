//! `sound::exclusive_spot_voices` (0x40EAF3): at most one voice of a kind starts: a kind
//! already playing, or started by an earlier voice in the table, silences the others.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! The kinds' bits (+0x1C) of the voices heard and playing gathered; then each voice
//! heard and not playing whose bit is among them is silenced (+4 0), and its bit added
//! either way. One caller, `sound::spot_sounds`.
//! Shadow mode: both, the voices compared.
//! Self-test: the original against the Rust version on the game's voices (put back).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::service::harness::{Region, capture, restore};
use crate::sound::spot_voices::{VOICE_COUNT, fill_voices, set, voice, voice_regions, vw};
use crate::sound::turn_parts::{Part, both, dispatch_turn};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "sound::exclusive_spot_voices",
    rva(0x40EAF3),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub fn exclusive_spot_voices_in(_: &mut Regs, _: Part) -> Result<(), Refused> {
    let voices: Vec<u32> = (0..VOICE_COUNT).map(voice).collect();
    let mut bits = 0u32;
    for &v in &voices {
        if vw(v, 4) == 1 && vw(v, 0x28) == 1 {
            bits |= vw(v, 0x1C);
        }
    }
    for &v in &voices {
        if vw(v, 4) == 1 && vw(v, 0x28) != 1 {
            if vw(v, 0x1C) & bits != 0 {
                set(v, 4, 0);
            }
            bits |= vw(v, 0x1C);
        }
    }
    Ok(())
}

// ---- hook ----

fn regions(_: &Regs) -> Vec<Region> {
    voice_regions()
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(
        &HOOK,
        r,
        call_original,
        exclusive_spot_voices_in,
        Some(&regions),
    );
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = voice_regions();
    let saved = capture(&all);
    for seed in 0..5u32 {
        let fill = || {
            fill_voices(seed, &|n, v| {
                let k = n.wrapping_add(seed * 3);
                set(v, 4, [1, 0, 1][k as usize % 3]);
                set(v, 0x28, [0, 1, 0, 0][k as usize % 4]);
                set(v, 0x1C, 1 << (k % 7));
            });
        };
        both(
            &mut t,
            &format!("voices {seed}"),
            &|r| unsafe { call_original(r) },
            Regs::SENTINEL,
            &fill,
            &all,
            exclusive_spot_voices_in,
            false,
        );
    }
    restore(&all, &saved);
    t.finish()
}
