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

use crate::math::x87;
use crate::physics::engine_torque::{below, engine_torque, g, p};

/// 0x43EA85 .. 0x43EAE7, as the original.
unsafe fn run() {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{rc} + 0x214740]",
            "fmul dword ptr [{rc} + 0x1CB968]",
            "fdiv dword ptr [{rc} + 0x2140C0]",
            "fdiv dword ptr [{rc} + 0x1C5E34]",
            "fild dword ptr [{rc} + 0x31A610]",
            ".byte 0xDE, 0xC9", // fmulp st(1), st
            "fdiv dword ptr [{rc} + 0x1C5E2C]",
            "fstp dword ptr [{rc} + 0x214750]",
            "fld dword ptr [{rc} + 0x214750]",
            "fmul dword ptr [{rc} + 0x21388C]",
            "fdiv dword ptr [{rc} + 0x2140C0]",
            "fadd dword ptr [{rc} + 0x31A1B0]",
            "fstp dword ptr [{rc} + 0x31A1B0]",
            "fld dword ptr [{rc} + 0x31A1B0]",
            "fmul dword ptr [{rc} + 0x213FD8]",
            "fadd dword ptr [{rc} + 0x213FD4]",
            "fstp dword ptr [{rc} + 0x22400C]",
            rc = in(reg) crate::race::store::asm_base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// The Rust version (the exact one, for `physics::free_rev_in_neutral`,
/// `free_rev_clutch` and their callers).
pub unsafe fn free_rev() {
    unsafe { engine_torque() };
    unsafe { run() };
    let (e, drag) = (g(0x71A1B0), g(0x62400C));
    let x = f32::from_bits(e);
    if x == 0.0 || x.is_nan() {
        return;
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
}
