//! `physics::limit_motion` (0x43F7AC): the work car's motion held to its limits. A motion
//! part that is no number or infinite (its exponent all ones) logs "*** INFINITY",
//! "[car0.newxspd] =" and the three (5 decimals) and zeroes the motion (+0x198), its car
//! frame copy (+0x1A4) and the spin (+0x184). Then the spin's parts held to
//! ±`flt_613E70` and the motion's (x, z, y) to ±`flt_613E74` (the limit negated in place
//! for the second compare and put back); its length (`physics::vector_length`) into
//! `flt_623FBC`, and past `flt_614100` the motion and its copy scaled down to it
//! (`physics::scale_vector`, the factor then in `dword_62400C`).
//!
//! The x87 runs are the original's instructions over the game's variables (from IDA).
//!
//! In:  nothing
//! Out: ecx the last part's exponent bits, esi +0x198 (+0x1A4 after a scaling); no
//!      flags read
//!
//! Callers: `sub_43B635` (twice), `sub_43BDD1`.

use crate::framework::{game_addr, game_read, game_write, rva};
use crate::logfile::log_float::log_float;
use crate::logfile::log_write::log_write;
use crate::physics::scale_vector::scale_vector;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::rounded;
use std::cmp::Ordering;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The value at `v` held to ±(the limit at `l`), as the original: `fld l; fld v;
/// fcompp`, the limit negated in place for the second compare and put back.
unsafe fn hold(v: u32, l: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{l}]",
            "fld dword ptr [{v}]",
            ".byte 0xDE, 0xD9", // fcompp
            "fnstsw ax",
            "shr ah, 1",
            "jb 2f",
            "fld dword ptr [{l}]",
            "fstp dword ptr [{v}]",
            "2:",
            "fld dword ptr [{l}]",
            ".byte 0xD9, 0xE0", // fchs
            "fstp dword ptr [{l}]",
            "fld dword ptr [{l}]",
            "fld dword ptr [{v}]",
            ".byte 0xDE, 0xD9",
            "fnstsw ax",
            "shr ah, 1",
            "jnb 3f",
            "fld dword ptr [{l}]",
            "fstp dword ptr [{v}]",
            "3:",
            "fld dword ptr [{l}]",
            ".byte 0xD9, 0xE0",
            "fstp dword ptr [{l}]",
            v = in(reg) v,
            l = in(reg) l,
            out("eax") _,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::find_clear_place`).
pub unsafe fn limit_motion() {
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut bad = false;
    for k in 0..3usize {
        let ecx = g(0x71A170 + 4 * k) & 0x7F80_0000;
        if ecx == 0x7F80_0000 {
            bad = true;
            break;
        }
    }
    if bad {
        unsafe {
            log_write(game_addr(rva(0x4DBB2C)) as *const u8, false, 1);
            log_write(game_addr(rva(0x4DBB39)) as *const u8, false, 1);
            for k in 0..3usize {
                log_float(f32::from_bits(g(0x71A170 + 4 * k)), 5, k == 2, 1);
            }
        }
        for k in 0..3usize {
            p(0x71A170 + 4 * k, 0);
            p(0x71A17C + 4 * k, 0);
            p(0x71A15C + 4 * k, 0);
        }
    }
    unsafe {
        for k in 0..3usize {
            hold(a(0x71A15C + 4 * k), a(0x613E70));
        }
        for k in [0usize, 2, 1] {
            hold(a(0x71A170 + 4 * k), a(0x613E74));
        }
        let length = rounded(vector_length(a(0x71A170)));
        p(0x623FBC, length);
        let top = g(0x614100);
        let past = matches!(
            f32::from_bits(top).partial_cmp(&f32::from_bits(length)),
            Some(Ordering::Less)
        ) || f32::from_bits(top).is_nan()
            || f32::from_bits(length).is_nan();
        if past {
            let factor = crate::math::x87_run::div_80(top, length);
            scale_vector(a(0x71A170), factor);
            scale_vector(a(0x71A17C), factor);
            p(0x62400C, rounded(factor));
        }
    }
}
