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
//! Self-test: the original and the converted version on the game's variables (saved and
//! put back): parts within, past and at the limits, lengths past the top; exact at double
//! precision, within `FLOAT` at extended (no infinities: the original logs).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{ECX, ESI, Regs, game_addr, game_read, game_write, rva};
use crate::logfile::log_float::log_float;
use crate::logfile::log_write::log_write;
use crate::physics::scale_vector::scale_vector;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::rounded;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;
use crate::service::harness::Region;
use rc2k_core::physics::limits::Moving;
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

fn floats3(x: usize) -> [f32; 3] {
    [0, 1, 2].map(|k| f32::from_bits(g(x + 4 * k)))
}

fn put3(x: usize, v: [f32; 3]) {
    for (k, f) in v.iter().enumerate() {
        p(x + 4 * k, f.to_bits());
    }
}

/// The converted version (`rc2k_core::physics::limits`, plain floats): the hook's.
// Converted: `rc2k_core::physics::limits::limit_motion`.
pub fn limit_motion_plain() -> Left {
    let a = |x: usize| game_addr(rva(x)) as u32;
    let f = |x: usize| f32::from_bits(g(x));
    let mut m = Moving {
        spin: floats3(0x71A15C),
        motion: floats3(0x71A170),
        local: floats3(0x71A17C),
        spin_limit: f(0x613E70),
        motion_limit: f(0x613E74),
    };
    let log = |motion: [f32; 3]| unsafe {
        log_write(game_addr(rva(0x4DBB2C)) as *const u8, false, 1);
        log_write(game_addr(rva(0x4DBB39)) as *const u8, false, 1);
        for (k, v) in motion.iter().enumerate() {
            log_float(*v, 5, k == 2, 1);
        }
    };
    let out = rc2k_core::physics::limits::limit_motion(&mut m, f(0x614100), log);
    put3(0x71A15C, m.spin);
    put3(0x71A170, m.motion);
    put3(0x71A17C, m.local);
    p(0x613E70, m.spin_limit.to_bits());
    p(0x613E74, m.motion_limit.to_bits());
    p(0x623FBC, out.length.to_bits());
    let mut esi = a(0x71A170);
    if let Some(factor) = out.factor {
        p(0x62400C, factor.to_bits());
        esi = a(0x71A17C);
    }
    Left::nothing().with(ECX, out.exponent).with(ESI, esi)
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::find_clear_place`).
pub unsafe fn limit_motion() -> Left {
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut ecx = 0;
    let mut bad = false;
    for k in 0..3usize {
        ecx = g(0x71A170 + 4 * k) & 0x7F80_0000;
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
        let mut esi = a(0x71A170);
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
            esi = a(0x71A17C);
        }
        Left::nothing().with(ECX, ecx).with(ESI, esi)
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(limit_motion_plain())
}

pub fn regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (a(0x71A15C), 0x10),
        (a(0x71A170), 0x18),
        (a(0x613E70), 8),
        (a(0x623FBC), 4),
        (a(0x62400C), 4),
    ]
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
