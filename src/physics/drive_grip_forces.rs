//! `physics::drive_grip_forces` (0x43CD79): the driven wheels' grip forces for the step.
//! In order: `flt_614768` = -(+0x1F4 / 50 - 1) held to 0 ..= 1; +0x210 the steer's sine
//! (+0x64C >> 20 into `g_sines`) x speed / `dword_614BA0` (twice over, as the original);
//! when moving: the slip limit `flt_614260` (a grip figure x `flt_613FC8` x a drive-kind
//! factor x `dword_614B74` / speed, `fabs`), +0x210 held to ±it, 0 below `flt_613E84`
//! speed with the drive speed not; with bit 12 of +0x468 +0x644 scaled by +0x210's share
//! of what it was (held 0 ..= 1, `fimul`, `fistp`), or 0; +0x210 x `flt_613E5C` /
//! `flt_613E60` with +0x284 1; the drive-kind 0 lean force +0x21C; the engine braking
//! +0x214 (by drive kind, the counts at +0x63C / +0x638 / +0x640, scaled, x `flt_613EB4`
//! against +0x210's sign, x `flt_613E54` / `58` with +0x284 -1); the drive-kind 1 yaw force
//! +0x218 (speed's share of `flt_613E7C`, +0x180 held to ±`flt_613E80`, negated in place
//! and put back); +0x204 the sum of the four; force records 19 and 20 (`g_forceRecords`)
//! 0, or with +0x254 clear the yaw twist eased toward +0x204 by `flt_613EA4`
//! (`flt_5C5E48`) and split over the rear wheels' places (+0xCF0 ..).
//!
//! The flow, the compares and the integer steps are Rust; each straight x87 run is a small
//! block of the original's instructions (generated from IDA, `physics/flow.rs`).
//!
//! In:  nothing
//! Out: eax, ebx and edx as the instructions leave them; no flags read (the caller calls
//!      on)
//!
//! Caller: `sub_43B635` (0x43B7A3), every physics step.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): drive kinds 0 / 1, speeds 0, small and large, the switches, under three
//! precisions.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87;
use crate::math::x87_run::{abs_compare, mul_status};
use crate::physics::flow::{a, c3, ease_zero, g, hold_high, hold_low, negate, p, x87_run};

/// `edx` = the steer angle's table index (+0x64C >> 20).
fn steer_index() -> u32 {
    g(0x71A624) >> 20
}

/// 0x43CDFF / 0x43CE21: +0x210 = sine x speed / `dword_614BA0`.
fn steer_force(e: u32) {
    x87_run!(
        "fld dword ptr [{bf} + {e}*4 + 0x623AE4]",
        "fmul dword ptr [{rc} + 0x31A184]",
        "fdiv dword ptr [{rc} + 0x214BA0]",
        "fstp dword ptr [{rc} + 0x31A1E8]";
        bf = crate::race::store::buffers_asm_base(), e = e, rc = crate::race::store::asm_base()
    );
}

/// The Rust version. The eax, ebx and edx it leaves are read by none of its callers.
pub unsafe fn drive_grip_forces() {
    let mut edx;
    let kind = g(0x71A018);
    // 0x43CD79: the share of +0x1F4 out of 50, from 1, held 0 ..= 1.
    p(0x62400C, 0x32);
    x87_run!(
        "fild dword ptr [{rc} + 0x31A1CC]",
        "fild dword ptr [{rc} + 0x22400C]",
        ".byte 0xDE, 0xF9", // fdivp st(1), st
        "fsub dword ptr [{rc} + 0x1C5D38]",
        ".byte 0xD9, 0xE0", // fchs
        "fstp dword ptr [{rc} + 0x214768]";
        rc = crate::race::store::asm_base()
    );
    hold_low(0x614768, 0x5C5D90);
    hold_high(0x614768, 0x5C5D38);
    p(0x71A1E8, 0);
    if g(0x71A220) == 0 {
        steer_force(steer_index());
    }
    edx = steer_index();
    steer_force(edx);
    if !c3(g(0x71A184), g(0x5C5D90)) {
        // 0x43CE53: the slip limit, by the drive kind's factor.
        let factor = if kind == 1 { 0x613EB0 } else { 0x613EAC };
        x87_run!(
            "fld dword ptr [{rc} + 0x2142A8]",
            "fmul dword ptr [{rc} + 0x213FC8]",
            "fmul dword ptr [{f}]",
            "fmul dword ptr [{rc} + 0x214B74]",
            "fdiv dword ptr [{rc} + 0x31A184]",
            ".byte 0xD9, 0xE1", // fabs
            "fstp dword ptr [{rc} + 0x214260]";
            f = a(factor), rc = crate::race::store::asm_base()
        );
        p(0x62400C, g(0x71A1E8));
        // +0x210 held to ±the limit (the value negated in place between).
        hold_high(0x71A1E8, 0x614260);
        negate(0x71A1E8);
        hold_high(0x71A1E8, 0x614260);
        negate(0x71A1E8);
        let slow = |v: usize| abs_compare(g(v), g(0x613E84)) & 0x100 != 0;
        if !slow(0x71A184) && slow(0x71A190) {
            p(0x71A1E8, 0);
        }
        if g(0x71A440) & 0x1000 != 0 {
            if g(0x62400C) == 0 {
                p(0x71A61C, 0);
            } else {
                x87_run!(
                    "fld dword ptr [{rc} + 0x31A1E8]",
                    "fdiv dword ptr [{rc} + 0x22400C]",
                    "fstp dword ptr [{rc} + 0x22400C]";
                    rc = crate::race::store::asm_base()
                );
                hold_high(0x62400C, 0x5C5D38);
                hold_low(0x62400C, 0x5C5D90);
                x87_run!(
                    "fld dword ptr [{rc} + 0x22400C]",
                    "fimul dword ptr [{rc} + 0x31A61C]",
                    "fistp dword ptr [{rc} + 0x31A61C]";
                    rc = crate::race::store::asm_base()
                );
            }
        }
    }
    // 0x43CFB7
    if g(0x71A25C) == 1 {
        let k = if kind == 1 { 0x613E5C } else { 0x613E60 };
        p(0x71A1E8, x87::mul(g(0x71A1E8), g(k)));
    }
    p(0x71A1F4, 0);
    if kind == 0 {
        x87_run!(
            "fld dword ptr [{rc} + 0x31A17C]",
            ".byte 0xD8, 0xC8", // fmul st, st
            "fld dword ptr [{rc} + 0x31A184]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1", // faddp st(1), st
            ".byte 0xD9, 0xFA", // fsqrt
            "fstp dword ptr [{rc} + 0x22400C]";
            rc = crate::race::store::asm_base()
        );
        hold_high(0x62400C, 0x613F34);
        edx = steer_index();
        x87_run!(
            "fld dword ptr [{rc} + 0x213F34]",
            "fsub dword ptr [{rc} + 0x22400C]",
            "fdiv dword ptr [{rc} + 0x213F34]",
            "fmul dword ptr [{bf} + {e}*4 + 0x623AE4]",
            "fimul dword ptr [{rc} + 0x31A618]",
            "fdiv dword ptr [{rc} + 0x1C5E2C]",
            "fmul dword ptr [{rc} + 0x31A190]",
            "fmul dword ptr [{rc} + 0x213F38]",
            "fimul dword ptr [{rc} + 0x31A610]",
            "fdiv dword ptr [{rc} + 0x1C5E2C]",
            "fstp dword ptr [{rc} + 0x31A1F4]";
            bf = crate::race::store::buffers_asm_base(), e = edx, rc = crate::race::store::asm_base()
        );
    }
    // 0x43D08E: the engine braking.
    let eax = if kind != 0 {
        x87_run!(
            "fld dword ptr [{rc} + 0x214BF0]",
            "fimul dword ptr [{rc} + 0x31A614]",
            "fmul dword ptr [{rc} + 0x1C5DB4]",
            "fistp dword ptr [{rc} + 0x22400C]";
            rc = crate::race::store::asm_base()
        );
        ((g(0x71A614).wrapping_sub(g(0x71A610)) as i32 >> 1) as u32)
            .wrapping_add(0x80)
            .wrapping_add(g(0x62400C))
            .wrapping_add(g(0x71A618))
    } else {
        x87_run!(
            "fld dword ptr [{rc} + 0x214BF0]",
            "fimul dword ptr [{rc} + 0x31A614]",
            ".byte 0xD8, 0xC0", // fadd st, st
            "fistp dword ptr [{rc} + 0x22400C]";
            rc = crate::race::store::asm_base()
        );
        0x100u32
            .wrapping_sub(g(0x71A610))
            .wrapping_add(g(0x62400C))
            .wrapping_add(g(0x71A618))
    };
    p(0x614164, eax);
    x87_run!(
        "fild dword ptr [{rc} + 0x214164]",
        "fdiv dword ptr [{rc} + 0x1C5E2C]",
        "fmul dword ptr [{rc} + 0x31A1A0]",
        ".byte 0xD9, 0xE0", // fchs
        "fmul dword ptr [{rc} + 0x213ECC]",
        "fmul dword ptr [{rc} + 0x214B78]",
        "fstp dword ptr [{rc} + 0x31A1EC]";
        rc = crate::race::store::asm_base()
    );
    if mul_status(g(0x71A1EC), g(0x71A1E8), g(0x5C5D90)) & 0x100 != 0 {
        p(0x71A1EC, x87::mul(g(0x71A1EC), g(0x613EB4)));
    }
    if g(0x71A25C) == u32::MAX {
        let k = if kind == 1 { 0x613E54 } else { 0x613E58 };
        p(0x71A1EC, x87::mul(g(0x71A1EC), g(k)));
    }
    p(0x71A1F0, 0);
    if kind == 1 {
        x87_run!(
            "fld dword ptr [{rc} + 0x31A184]",
            ".byte 0xD9, 0xE1", // fabs
            "fstp dword ptr [{rc} + 0x22401C]";
            rc = crate::race::store::asm_base()
        );
        hold_high(0x62401C, 0x613E7C);
        x87_run!(
            "fld dword ptr [{rc} + 0x213E7C]",
            "fsub dword ptr [{rc} + 0x22401C]",
            "fdiv dword ptr [{rc} + 0x213E7C]",
            "fstp dword ptr [{rc} + 0x22401C]";
            rc = crate::race::store::asm_base()
        );
        p(0x62400C, x87::loaded(g(0x71A158)));
        // Held to ±`flt_613E80`, the limit negated in place between and put back.
        hold_high(0x62400C, 0x613E80);
        negate(0x613E80);
        hold_low(0x62400C, 0x613E80);
        negate(0x613E80);
        x87_run!(
            "fld dword ptr [{rc} + 0x22400C]",
            "fdiv dword ptr [{rc} + 0x213E80]",
            "fstp dword ptr [{rc} + 0x22400C]";
            rc = crate::race::store::asm_base()
        );
        edx = steer_index();
        x87_run!(
            "fld dword ptr [{bf} + {e}*4 + 0x623AE4]",
            "fdiv dword ptr [{rc} + 0x214BA0]",
            "fimul dword ptr [{rc} + 0x31A610]",
            "fdiv dword ptr [{rc} + 0x1C5E2C]",
            "fmul dword ptr [{rc} + 0x22400C]",
            "fmul dword ptr [{rc} + 0x22401C]",
            "fmul dword ptr [{rc} + 0x213E78]",
            "fstp dword ptr [{rc} + 0x31A1F0]";
            bf = crate::race::store::buffers_asm_base(), e = edx, rc = crate::race::store::asm_base()
        );
    }
    // 0x43D2C9: the sum, and the yaw twist on the rear wheels.
    x87_run!(
        "fld dword ptr [{rc} + 0x31A1E8]",
        "fadd dword ptr [{rc} + 0x31A1EC]",
        "fadd dword ptr [{rc} + 0x31A1F0]",
        "fadd dword ptr [{rc} + 0x31A1F4]",
        "fstp dword ptr [{rc} + 0x31A1DC]";
        rc = crate::race::store::asm_base()
    );
    p(0x6149D0, 0);
    p(0x6149F0, 0);
    if g(0x71A22C) != 0 {
        return;
    }
    p(0x71A1D8, x87::loaded(g(0x71A160)));
    let old = g(0x71A1D8);
    p(0x5C5E48, x87::sub(old, g(0x71A1DC)));
    ease_zero(0x5C5E48, 0x613EA4);
    let held = [old];
    x87_run!(
        "fld dword ptr [{rc} + 0x1C5E48]",
        "fadd dword ptr [{rc} + 0x31A1DC]",
        "fstp dword ptr [{rc} + 0x31A1D8]",
        "fld dword ptr [{o}]",
        "fsub dword ptr [{rc} + 0x31A1D8]",
        ".byte 0xD9, 0xE0", // fchs
        "fstp dword ptr [{rc} + 0x22400C]",
        "fld dword ptr [{rc} + 0x22400C]",
        "fdiv dword ptr [{rc} + 0x21388C]",
        "fmul dword ptr [{rc} + 0x1C5E40]",
        "fdiv dword ptr [{rc} + 0x213FAC]",
        "fst dword ptr [{rc} + 0x2149D0]",
        ".byte 0xD9, 0xE0", // fchs
        "fstp dword ptr [{rc} + 0x2149F0]",
        ".byte 0xD9, 0xE8", // fld1
        "fst dword ptr [{rc} + 0x2149E4]",
        ".byte 0xD9, 0xE0",
        "fstp dword ptr [{rc} + 0x214A04]",
        "fld dword ptr [{rc} + 0x2149D0]",
        "fdiv dword ptr [{rc} + 0x31ACC8]",
        "fstp dword ptr [{rc} + 0x2149D0]",
        "fld dword ptr [{rc} + 0x2149E4]",
        "fmul dword ptr [{rc} + 0x31ACC8]",
        "fstp dword ptr [{rc} + 0x2149E4]",
        "fld dword ptr [{rc} + 0x2149F0]",
        "fdiv dword ptr [{rc} + 0x31ACD4]",
        ".byte 0xD9, 0xE0",
        "fstp dword ptr [{rc} + 0x2149F0]",
        "fld dword ptr [{rc} + 0x214A04]",
        "fmul dword ptr [{rc} + 0x31ACD4]",
        ".byte 0xD9, 0xE0",
        "fstp dword ptr [{rc} + 0x214A04]";
        o = held.as_ptr(), rc = crate::race::store::asm_base()
    );
}
