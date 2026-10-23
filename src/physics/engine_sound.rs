//! `physics::engine_sound` (0x43DC72): the work car's sound figures. +0xF08
//! (`unk_71AEE0`) = speed x `flt_5C5D00`, and that (x `flt_5C5E08` with bit 8 of +0x220)
//! `fistp` into +0xAB0; +0x294 = the motion's length (`fsqrt` of z² + x² + y², as the
//! original adds them) x `flt_5C5D00`; the revs +0x1E4 eased toward the engine speed (at
//! least `flt_6140F8`, `fistp`, through `dword_62400C`) by 50/256 of the gap (`imul`,
//! `sar`), held to 2000 ..= 12000 (by the sign); then `physics::car_sound_state` and
//! `physics::gear_change_sound`.
//!
//! The x87 runs are the original's instructions over the game's variables (from IDA).
//!
//! In:  nothing
//! Out: eax the gap's step, edi the sound object (`gear_change_sound`); no flags read
//!
//! Caller: `sub_43B635` (0x43B905), every physics step.
//! Refuses (before anything) what the two calls refuse.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with a test object, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::math::x87_run::ext_to_int;
use crate::physics::car_sound_state::car_sound_state;
use crate::physics::gear_change_sound::gear_change_sound;
use crate::scene::matrix3::extended;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::engine::ease_revs;
use rc2k_core::physics::x87::hold_low;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// 0x43DC72 .. 0x43DCC6, as the original (its `test` / `jz` in the block).
unsafe fn run() {
    let b = game_addr(0) as u32;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x31A184]",
            "fmul dword ptr [{b} + 0x1C5D00]",
            "fstp dword ptr [{b} + 0x31AEE0]",
            "fld dword ptr [{b} + 0x31AEE0]",
            "test dword ptr [{b} + 0x31A1F8], 0x100",
            "jz 2f",
            "fmul dword ptr [{b} + 0x1C5E08]",
            "2:",
            "fistp dword ptr [{b} + 0x31AA88]",
            "fld dword ptr [{b} + 0x31A178]",
            ".byte 0xD8, 0xC8", // fmul st, st
            "fld dword ptr [{b} + 0x31A170]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x31A174]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1", // faddp st(1), st
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA", // fsqrt
            "fmul dword ptr [{b} + 0x1C5D00]",
            "fstp dword ptr [{b} + 0x31A26C]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// The Rust version.
// Converted: `rc2k_core::physics::engine::ease_revs` and `x87::hold_low` (the speed figures
// and the rounding stay x87).
pub unsafe fn engine_sound() -> Result<Left, Refused> {
    let o = g(0x512A10 + 4 * (g(0x71B3F8) & 7) as usize);
    if o == 0 {
        return Err(crate::errors::BadInput("a null sound object").into());
    }
    if g(0x63DB94) == 1 && g(0x63DBB0) == 0 {
        return Err(crate::errors::BadInput("a null view block").into());
    }
    unsafe { run() };
    p(0x62400C, x87::loaded(g(0x71A1B0)));
    let floor = g(0x6140F8);
    let held = hold_low(f32::from_bits(g(0x62400C)), f32::from_bits(floor));
    p(0x62400C, held.to_bits());
    p(0x62400C, ext_to_int(extended(g(0x62400C))) as u32);
    let (revs, step) = ease_revs(g(0x71A1BC), g(0x62400C));
    p(0x71A1BC, revs);
    unsafe { car_sound_state() }?;
    let left = unsafe { gear_change_sound() }?;
    Ok(left.with(EAX, step))
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { engine_sound() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
