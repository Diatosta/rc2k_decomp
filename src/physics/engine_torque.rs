//! `physics::engine_torque` (0x43CBBB): the engine's torque from its speed. s = engine
//! speed / its top (+0x1DC) into `dword_62400C`; for s in 0 ..= below 1 (`flt_5C5D90`,
//! `flt_5C5D38`; unordered counts as out) the curve's entry i = s x 8 (`fistp` with the
//! rounding set to chop) into `dword_623FB8` (ebx), and `flt_614748` the line between
//! (`flt_613FF0[i]`, `dword_614B7C[i]`) and (`flt_613FF0[i + 1]`, `dword_614B7C[i + 1]`)
//! at s; else `dword_614B7C[0]`. Then `flt_614744` = engine / top x `flt_613F94` (0.25),
//! and `flt_614748` times +0x3F4 (`unk_71A3CC`, as a float).
//!
//! In:  nothing
//! Out: ebx i on the curve's path; the rest kept; no flags read
//!
//! Callers: `physics::engine_torque_or_cut`, `sub_421848`, `sub_43CAAB`, `sub_43EA80`.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back), against the converted version: engine speeds across the curve and past it,
//! exact at double precision (`test_engine_floats`).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EBX, game_read, game_write, rva};
use crate::math::x87;
use crate::math::x87_run::{div_then_mul, lerp_table, trunc_mul_to_int};
use crate::screens::harness::Left;
use std::cmp::Ordering;

pub fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

pub fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// C0 of comparing `a` with `b`: below, or unordered.
pub fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The Rust version (the exact one, for `physics::engine_torque_or_cut`, `drive_force`,
/// `free_rev*`, `test_drive_run`): ebx when the curve was read.
pub unsafe fn engine_torque() -> Left {
    let mut left = Left::nothing();
    let s = x87::div(g(0x71A1B0), g(0x71A1B4));
    p(0x62400C, s);
    if below(s, g(0x5C5D38)) && !below(s, g(0x5C5D90)) {
        let i = trunc_mul_to_int(s, g(0x5C5D50)) as u32;
        p(0x623FB8, i);
        left = left.with(EBX, i);
        let k = 4 * i as usize;
        p(
            0x614748,
            lerp_table(
                s,
                g(0x613FF0 + k),
                g(0x613FF4 + k),
                g(0x614B7C + k),
                g(0x614B80 + k),
            ),
        );
    } else {
        p(0x614748, x87::loaded(g(0x614B7C)));
    }
    p(
        0x614744,
        div_then_mul(g(0x71A1B0), g(0x71A1B4), g(0x613F94)),
    );
    p(0x614748, x87::mul(g(0x614748), g(0x71A3CC)));
    left
}
