//! `sound::spot_voice_distances` (0x40E733): each voice heard this turn: its distance,
//! the distance's change and the pitch and volume factors, as `sound::car_engine_state`
//! does for a car.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! For each voice heard (+4 1): +0x40 the root of +8 .. +0x10 squared, +0x44 its change
//! x the turns a second (`dword_5124EC`); +0x4C 0 and +0x48 1.0, then unless the voice
//! is new (+0x20 1) +0x48 eased from 1.0 toward (+0x44 + 337.52) x 0.00296; +0x4C 1.0,
//! or 1 / the distance when that is over 1 (compared as integers); +0x20 cleared. See
//! `sound/spot_voices.rs`. One caller, `sound::spot_sounds`.
//! Shadow mode: both, the voices compared.
//! Self-test: the original against the Rust version on the game's voices (put back).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{base, x87_run};
use crate::service::harness::{Region, capture, restore};
use crate::sound::spot_voices::{VOICE_COUNT, fill_voices, set, voice, voice_regions, vw};
use crate::sound::turn_parts::{Part, both, dispatch_turn};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "sound::spot_voice_distances",
    rva(0x40E733),
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

pub fn spot_voice_distances_in(_: &mut Regs, _: Part) -> Result<(), Refused> {
    let b = base();
    for n in 0..VOICE_COUNT {
        let bx = voice(n);
        if vw(bx, 4) != 1 {
            continue;
        }
        x87_run!(
            "fld dword ptr [{bx} + 0x40]",
            "fld dword ptr [{bx} + 0x8]",
            "fmul dword ptr [{bx} + 0x8]",
            "fld dword ptr [{bx} + 0xC]",
            "fmul dword ptr [{bx} + 0xC]",
            "fld dword ptr [{bx} + 0x10]",
            "fmul dword ptr [{bx} + 0x10]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fst dword ptr [{bx} + 0x40]",
            ".byte 0xDE, 0xE9",
            "fmul dword ptr [{b} + 0x1124EC]",
            "fstp dword ptr [{bx} + 0x44]";
            bx = bx, b = b
        );
        set(bx, 0x4C, 0);
        set(bx, 0x48, 0x3F80_0000);
        if vw(bx, 0x20) != 1 {
            x87_run!(
                "fld dword ptr [{bx} + 0x44]",
                "fadd dword ptr [{b} + 0x1124F8]",
                "fmul dword ptr [{b} + 0x1124FC]",
                "fsub dword ptr [{bx} + 0x48]",
                "fmul dword ptr [{b} + 0x116360]",
                "fadd dword ptr [{bx} + 0x48]",
                "fstp dword ptr [{bx} + 0x48]";
                bx = bx, b = b
            );
        }
        set(bx, 0x4C, 0x3F80_0000);
        if vw(bx, 0x40) as i32 > 0x3F80_0000 {
            x87_run!(
                ".byte 0xD9, 0xE8",
                "fdiv dword ptr [{bx} + 0x40]",
                "fstp dword ptr [{bx} + 0x4C]";
                bx = bx
            );
        }
        set(bx, 0x20, 0);
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
        spot_voice_distances_in,
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
    for seed in 0..4u32 {
        let fill = || {
            fill_voices(seed, &|n, v| {
                let k = n.wrapping_add(seed);
                set(v, 4, [1, 0, 1][k as usize % 3]);
                set(v, 0x20, [0, 1][k as usize % 2]);
                set(
                    v,
                    8,
                    [0x3F00_0000, 0x4120_0000, 0xC2C8_0000, 0][k as usize % 4],
                );
            });
        };
        both(
            &mut t,
            &format!("voices {seed}"),
            &|r| unsafe { call_original(r) },
            Regs::SENTINEL,
            &fill,
            &all,
            spot_voice_distances_in,
            false,
        );
    }
    restore(&all, &saved);
    t.finish()
}
