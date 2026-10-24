//! `physics::load_car_specs` (0x421482): the work car's handling figures from its kind's
//! record (`g_car` +0x24, at most 0x21; records of 0xB0 at 0x5C790C, "00 renault maxi
//! megane" ..): the drive kind; four whole figures as floats into `flt_5CB96C` .. (each at
//! least 8.0) and the engine's top (+0x1DC, at least `flt_5C5D18`); the shift points
//! `flt_5CB958` .. `flt_5CB964`; `flt_5CB968`; `flt_614104`; the torque curve (nine
//! dwords) at `dword_614B7C`; the gear count (3 ..= 7, by the sign) and nine more words;
//! three whole figures x 65536 / `flt_5C5D18` `fistp` into `dword_614BC0` .. (for kinds
//! below 0x17 from `dword_613F04` .. instead); `dword_614BA0` .. from `dword_613EFC`,
//! `613F00`, `613F14` (the record's first written, then these); two floats x
//! `flt_613FC8`; `dword_62400C`, `dword_61476C`; and by the drive kind `flt_6140F8` and
//! it + 100 into `flt_6140FC`.
//!
//! The x87 runs are the original's instructions (from IDA); the compares and the integer
//! steps in Rust.
//!
//! In:  nothing
//! Out: eax the record's gear count, ebx the drive kind, esi the record; no flags read
//!
//! Callers: `sub_43B635` (0x43B63A), `sub_4217E4`. Refuses (before anything) a drive kind
//! past 3 (`flt_613FB8`'s size not worked out).

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;
use rc2k_core::physics::car_specs::{
    OWN_FIGURES_FROM, RECORD_LEN, check_drive, gear_count, record_of,
};
use std::cmp::Ordering;

const RECORDS: usize = 0x5C790C; // the kinds' records

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

macro_rules! x87_block {
    ($($line:literal),+ ; $($name:ident = $value:expr),*) => {
        core::arch::asm!(
            $($line),+,
            $($name = in(reg) $value),*,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    };
}

/// The record of the work car's kind.
fn record() -> u32 {
    game_addr(rva(RECORDS)) as u32 + RECORD_LEN * record_of(g(0x719FFC))
}

/// The whole-number steps both versions share, after the floats: the torque curve, the
/// gear count (held), the words copied.
unsafe fn words(s: u32) {
    unsafe {
        for k in 0..9u32 {
            p(0x614B7C + 4 * k as usize, at(s + 0x7C + 4 * k));
        }
        p(0x614C18, at(s + 0xA0));
        let count = at(s + 0x1C);
        p(0x614C10, gear_count(count));
        for (to, from) in [
            (0x614C14usize, 0x20u32),
            (0x614BB4, 0x2C),
            (0x614BB8, 0x30),
            (0x614B74, 0x24),
            (0x614B78, 0x28),
            (0x614BBC, 0x34),
        ] {
            p(to, at(s + from));
        }
    }
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::test_drive`).
pub unsafe fn load_car_specs() -> Result<(), Refused> {
    let kind = g(0x719FFC);
    let s = record();
    let drive = check_drive(unsafe { at(s + 0x18) })?;
    p(0x71A018, drive);
    // 0x4214A7 .. 0x421543
    unsafe {
        x87_block!(
            "fild dword ptr [{s} + 0x64]",
            "fstp dword ptr [{rc} + 0x1CB96C]",
            "fild dword ptr [{s} + 0x68]",
            "fstp dword ptr [{rc} + 0x1CB970]",
            "fild dword ptr [{s} + 0x6C]",
            "fstp dword ptr [{rc} + 0x1CB974]",
            "fild dword ptr [{s} + 0x70]",
            "fstp dword ptr [{rc} + 0x1CB978]",
            "fild dword ptr [{s} + 0x74]",
            "fstp dword ptr [{rc} + 0x31A1B4]",
            "fld dword ptr [{rc} + 0x213F10]",
            "fstp dword ptr [{rc} + 0x31A1B8]",
            "fld dword ptr [{s} + 0xA4]",
            "fst dword ptr [{rc} + 0x1CB958]",
            "fmul dword ptr [{rc} + 0x213E88]",
            "fstp dword ptr [{rc} + 0x1CB964]",
            "fld dword ptr [{s} + 0xA8]",
            "fadd dword ptr [{rc} + 0x1C5DDC]",
            "fstp dword ptr [{rc} + 0x1CB95C]",
            "fld dword ptr [{s} + 0xAC]",
            "fadd dword ptr [{rc} + 0x1C5DDC]",
            "fstp dword ptr [{rc} + 0x1CB960]",
            "fild dword ptr [{s} + 0x78]",
            "fmul dword ptr [{rc} + 0x213EB8]",
            "fdiv dword ptr [{rc} + 0x31A1B4]",
            "fstp dword ptr [{rc} + 0x1CB968]",
            "fld dword ptr [{rc} + 0x31A1B4]",
            "fmul dword ptr [{rc} + 0x213FE4]",
            "fstp dword ptr [{rc} + 0x214104]";
            s = s, rc = crate::race::store::asm_base()
        )
    };
    if below(g(0x71A1B4), g(0x5C5D18)) {
        p(0x71A1B4, x87::loaded(g(0x5C5D18)));
    }
    for k in 0..4usize {
        let at_k = 0x5CB96C + 4 * k;
        if below(g(at_k), g(0x5C5D50)) {
            p(at_k, x87::loaded(g(0x5C5D50)));
        }
    }
    unsafe {
        words(s);
        // 0x4216C5 .. 0x421704
        x87_block!(
            "fild dword ptr [{s} + 0x40]",
            "fmul dword ptr [{rc} + 0x1C5D98]",
            "fdiv dword ptr [{rc} + 0x1C5D18]",
            "fistp dword ptr [{rc} + 0x214BC0]",
            "fild dword ptr [{s} + 0x44]",
            "fmul dword ptr [{rc} + 0x1C5D98]",
            "fdiv dword ptr [{rc} + 0x1C5D18]",
            "fistp dword ptr [{rc} + 0x214BC4]",
            "fild dword ptr [{s} + 0x48]",
            "fmul dword ptr [{rc} + 0x1C5D98]",
            "fdiv dword ptr [{rc} + 0x1C5D18]",
            "fistp dword ptr [{rc} + 0x214BC8]";
            s = s, rc = crate::race::store::asm_base()
        );
        if kind < OWN_FIGURES_FROM {
            // 0x42170D .. 0x421755
            x87_block!(
                "fild dword ptr [{rc} + 0x213F04]",
                "fmul dword ptr [{rc} + 0x1C5D98]",
                "fdiv dword ptr [{rc} + 0x1C5D18]",
                "fistp dword ptr [{rc} + 0x214BC0]",
                "fild dword ptr [{rc} + 0x213F08]",
                "fmul dword ptr [{rc} + 0x1C5D98]",
                "fdiv dword ptr [{rc} + 0x1C5D18]",
                "fistp dword ptr [{rc} + 0x214BC4]",
                "fild dword ptr [{rc} + 0x213F0C]",
                "fmul dword ptr [{rc} + 0x1C5D98]",
                "fdiv dword ptr [{rc} + 0x1C5D18]",
                "fistp dword ptr [{rc} + 0x214BC8]";
                rc = crate::race::store::asm_base()
            );
        }
        p(0x614BA0, g(0x613EFC));
        p(0x614BA4, g(0x613F00));
        p(0x614BA8, g(0x613F14));
        // 0x421794 .. 0x4217B2
        x87_block!(
            "fld dword ptr [{s} + 0x38]",
            "fmul dword ptr [{rc} + 0x213FC8]",
            "fstp dword ptr [{rc} + 0x214BAC]",
            "fld dword ptr [{s} + 0x3C]",
            "fmul dword ptr [{rc} + 0x213FC8]",
            "fstp dword ptr [{rc} + 0x214BB0]";
            s = s, rc = crate::race::store::asm_base()
        );
        p(0x62400C, at(s + 0x5C));
        p(0x61476C, at(s + 0x60));
        // 0x4217CA .. 0x4217E3
        x87_block!(
            "fld dword ptr [{rc} + {k}*4 + 0x213FB8]",
            "fst dword ptr [{rc} + 0x2140F8]",
            "fadd dword ptr [{rc} + 0x1C5D14]",
            "fstp dword ptr [{rc} + 0x2140FC]";
            k = drive, rc = crate::race::store::asm_base()
        );
        Ok(())
    }
}
