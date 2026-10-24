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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::flow::x87_run;
use crate::sound::spot_voices::{VOICE_COUNT, set, voice, vw};

// ---- Rust version ----

pub fn spot_voice_distances() {
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
            "fmul dword ptr [{ck} + 8]",
            "fstp dword ptr [{bx} + 0x44]";
            bx = bx, ck = crate::sound::store::clock::base()
        );
        set(bx, 0x4C, 0);
        set(bx, 0x48, 0x3F80_0000);
        if vw(bx, 0x20) != 1 {
            x87_run!(
                            "fld dword ptr [{bx} + 0x44]",
                            "fadd dword ptr [{ck} + 0x14]",
                            "fmul dword ptr [{ck} + 0x18]",
                            "fsub dword ptr [{bx} + 0x48]",
                            "fmul dword ptr [{sf} + 0x8]",
                            "fadd dword ptr [{bx} + 0x48]",
                            "fstp dword ptr [{bx} + 0x48]";
                bx = bx, ck = crate::sound::store::clock::base(), sf = crate::sound::store::surface::base()
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
}
