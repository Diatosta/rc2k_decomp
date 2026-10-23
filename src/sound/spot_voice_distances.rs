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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::{base, x87_run};
use crate::sound::spot_voices::{VOICE_COUNT, load_voices, set, store_voices, voice, vw};
use crate::sound::turn_parts::turn_entry;

// ---- Rust version ----

/// The converted version (`rc2k_core::sound::spot_voices::hear`, plain floats): the hook's. The
/// exact one below stays for the race sound's turn (`turn_parts`), compared exactly.
pub fn spot_voice_distances_plain(_: &mut Regs) -> Result<(), Refused> {
    let float = |x: usize| f32::from_bits(crate::sound::spot_voices::g(x));
    let hearing = rc2k_core::sound::spot_voices::Hearing {
        turns_a_second: float(0x5124EC),
        offset: float(0x5124F8),
        factor: float(0x5124FC),
        ease: float(0x516360),
    };
    let mut voices = load_voices();
    rc2k_core::sound::spot_voices::hear(&mut voices, &hearing);
    store_voices(&voices);
    Ok(())
}

pub fn spot_voice_distances_in(_: &mut Regs) -> Result<(), Refused> {
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, spot_voice_distances_plain)
}
