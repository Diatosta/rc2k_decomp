//! `physics::free_rev` (0x43EA80): the engine revving free. `physics::engine_torque`;
//! `flt_614750` = `flt_614740` x `flt_5CB968` / gear 0's ratio / 10000 x +0x638 (`fild`) /
//! 256; the engine speed + it x the step / the ratio; the drag `dword_62400C` = engine
//! speed x `flt_613FD8` + `flt_613FD4`; then the engine speed brought toward 0 by the drag
//! (not past it; at 0 or unordered left alone).
//!
//! The x87 run is the original's instructions over the game's variables (from IDA).
//!
//! In:  nothing
//! Out: as `engine_torque` leaves them; no flags read (the callers return or call on)
//!
//! Callers: `physics::free_rev_in_neutral`, `physics::free_rev_clutch`.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back), against the converted version: `physics::engine_torque`'s cases
//! (`test_engine_floats`), exact at double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr};
use crate::math::x87;
use crate::physics::engine_torque::{
    below, engine_entry, engine_torque, engine_torque_plain, g, p,
};
use crate::screens::harness::Left;
use rc2k_core::physics::engine::{FreeRev, free_rev as free_rev_core};

/// 0x43EA85 .. 0x43EAE7, as the original.
unsafe fn run() {
    let b = game_addr(0) as u32;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x214740]",
            "fmul dword ptr [{b} + 0x1CB968]",
            "fdiv dword ptr [{b} + 0x2140C0]",
            "fdiv dword ptr [{b} + 0x1C5E34]",
            "fild dword ptr [{b} + 0x31A610]",
            ".byte 0xDE, 0xC9", // fmulp st(1), st
            "fdiv dword ptr [{b} + 0x1C5E2C]",
            "fstp dword ptr [{b} + 0x214750]",
            "fld dword ptr [{b} + 0x214750]",
            "fmul dword ptr [{b} + 0x21388C]",
            "fdiv dword ptr [{b} + 0x2140C0]",
            "fadd dword ptr [{b} + 0x31A1B0]",
            "fstp dword ptr [{b} + 0x31A1B0]",
            "fld dword ptr [{b} + 0x31A1B0]",
            "fmul dword ptr [{b} + 0x213FD8]",
            "fadd dword ptr [{b} + 0x213FD4]",
            "fstp dword ptr [{b} + 0x22400C]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// The converted version (`rc2k_core::physics::engine`, plain floats): the hook's.
// Converted: `rc2k_core::physics::engine::free_rev`.
pub unsafe fn free_rev_plain() -> Left {
    let left = unsafe { engine_torque_plain() };
    let f = |x: usize| f32::from_bits(g(x));
    let (push, engine, drag) = free_rev_core(&FreeRev {
        drive: f(0x614740),
        drive_factor: f(0x5CB968),
        ratio: f(0x6140C0),
        k10000: f(0x5C5E34),
        gearing: g(0x71A610) as i32,
        k256: f(0x5C5E2C),
        step: f(0x61388C),
        engine: f(0x71A1B0),
        drag_factor: f(0x613FD8),
        drag_base: f(0x613FD4),
    });
    p(0x614750, push.to_bits());
    p(0x71A1B0, engine.to_bits());
    p(0x62400C, drag.to_bits());
    left
}

/// The float places `free_rev` writes (IDA addresses, lengths).
pub const FREE_REV_FLOATS: [(usize, usize); 4] =
    [(0x62400C, 4), (0x614744, 8), (0x614750, 4), (0x71A1B0, 4)];

/// The Rust version (the exact one, for `physics::free_rev_in_neutral`,
/// `free_rev_clutch` and their callers).
pub unsafe fn free_rev() -> Left {
    let left = unsafe { engine_torque() };
    unsafe { run() };
    let (e, drag) = (g(0x71A1B0), g(0x62400C));
    let x = f32::from_bits(e);
    if x == 0.0 || x.is_nan() {
        return left;
    }
    if x < 0.0 {
        let y = x87::add(e, drag);
        p(0x71A1B0, y);
        if !below(y, 0) {
            p(0x71A1B0, 0);
        }
    } else {
        let y = x87::sub(e, drag);
        p(0x71A1B0, y);
        if below(y, 0) {
            p(0x71A1B0, 0);
        }
    }
    left
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { free_rev_plain() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    engine_entry(r, &rust)
}
