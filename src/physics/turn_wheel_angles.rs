//! `physics::turn_wheel_angles` (0x449EE9): the wheels' turning angles moved on by the
//! speed (a reading: their spin, for the drawing). Only when the car's kind (`g_car`
//! +0x24, `dword_719FFC`) is below 0x22 and its entry in `dword_5C5B48` is not 0 (else CF
//! and nothing). Then, with `dword_71A22C` 0, s = `g_carDriveSpeed` (else 0), held at
//! 0 or more (`flt_5C5D90`), times `flt_623DA8` and `flt_623DB0`, added to the four
//! floats at `dword_623D60`; and s = `g_carDriveSpeed` less `g_carSpeed` (else 0), held
//! the same, times `flt_623DAC` and `flt_623DB4`, added to the four at `dword_623D70`.
//! Each s goes through `dword_62400C`.
//!
//! In:  nothing
//! Out: every register kept; CF when skipped (no caller reads it: the next is an `fld`)
//!
//! Caller: `sub_43B635` (0x43B91E), every physics step.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): kinds with and without a table entry and past the table, both switch settings,
//! speeds by lot and below 0, against the converted version, exact at double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::math::x87;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;
use rc2k_core::physics::wheels::{add_turn as add_turn_core, slip, turns_wheels};
use std::cmp::Ordering;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

const KIND: usize = 0x719FFC; // g_car + 0x24
const KINDS: usize = 0x5C5B48; // dword_5C5B48, 0x22 entries
const STILL: usize = 0x71A22C; // dword_71A22C
const SCRATCH: usize = 0x62400C;
const ZERO: usize = 0x5C5D90;

/// `fld zero; fld s; fcompp`, C0: s below zero or unordered.
fn below(s: u32, zero: u32) -> bool {
    !matches!(
        f32::from_bits(s).partial_cmp(&f32::from_bits(zero)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// One half: `speed` (None with the switch set) held, scaled, added to the four at `to`.
fn add_turn(speed: Option<u32>, f1: usize, f2: usize, to: usize) {
    p(SCRATCH, 0);
    if let Some(s) = speed {
        p(SCRATCH, s);
        if below(g(SCRATCH), g(ZERO)) {
            p(SCRATCH, x87::loaded(g(ZERO)));
        }
    }
    p(SCRATCH, x87::mul2(g(SCRATCH), g(f1), g(f2)));
    for k in 0..4 {
        p(to + 4 * k, x87::add(g(to + 4 * k), g(SCRATCH)));
    }
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn turn_wheel_angles() -> Left {
    let kind = g(KIND);
    if kind >= 0x22 || g(KINDS + 4 * kind as usize) == 0 {
        return Left::carry(true);
    }
    let still = g(STILL) != 0;
    let drive = || x87::loaded(g(0x71A190));
    add_turn((!still).then(drive), 0x623DA8, 0x623DB0, 0x623D60);
    let slip = || x87::sub(g(0x71A190), g(0x71A184));
    add_turn((!still).then(slip), 0x623DAC, 0x623DB4, 0x623D70);
    Left::nothing()
}

/// The converted version (`rc2k_core::physics::wheels`, plain floats): the hook's.
// Converted: `rc2k_core::physics::wheels::{turns_wheels, add_turn, slip}`.
pub unsafe fn turn_wheel_angles_plain() -> Left {
    if !turns_wheels(g(KIND), |k| g(KINDS + 4 * k as usize)) {
        return Left::carry(true);
    }
    let f = |x: usize| f32::from_bits(g(x));
    let still = g(STILL) != 0;
    let half = |speed: Option<f32>, f1: usize, f2: usize, to: usize| {
        let mut angles = [0, 1, 2, 3].map(|k| f(to + 4 * k));
        let step = add_turn_core(speed, f(ZERO), f(f1), f(f2), &mut angles);
        p(SCRATCH, step.to_bits());
        for (k, v) in angles.iter().enumerate() {
            p(to + 4 * k, v.to_bits());
        }
    };
    let drive = (!still).then(|| f(0x71A190));
    half(drive, 0x623DA8, 0x623DB0, 0x623D60);
    let slip = (!still).then(|| slip(f(0x71A190), f(0x71A184)));
    half(slip, 0x623DAC, 0x623DB4, 0x623D70);
    Left::nothing()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    let left = unsafe { turn_wheel_angles_plain() };
    // Flags are only compared on the skipped path (`Left::carry`).
    Ok(left)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
