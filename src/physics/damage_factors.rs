//! `physics::damage_factors` (0x4AB5E6): the damage model's figures for this step
//! (`physics/damage_model.rs`). Three car figures / `flt_75B35C` into `flt_75B3F8` ..;
//! for the four gauges `unk_71A30C` .. `71A318`: a table lerp (`unk_75B4C8`) into
//! `flt_75B474` .., the factor `unk_71A420` + 4k (held 0 ..= `flt_75B34C`) and
//! `unk_71A410` + 4k (both set when the gauge is at the top); their sum x `flt_75B364`
//! into `unk_71A430`; then shares of the engine's and the car's speeds against their
//! limits (`flt_75B398` .. `flt_75B3AC`), the switches `flt_75B3F4`, `flt_75B3B0`
//! (`unk_71A614` at 0x10 or more and moving), `flt_75B3C0` (the engine past its limit x
//! `flt_75B37C`), `flt_75B3B4` (`unk_71A908` at 5), copies into `dword_75B3B8` ..
//! (the gear `dword_75B3BC`), the side hit `dword_75B3C4` (1 .. 4 by `unk_71A1FC`,
//! `unk_71A204`, when `unk_71A208` is set) and its size `flt_75B3C8`, the four wheels'
//! loads (`dword_75B3CC` .., blended by `flt_75B360`), and the hit time `unk_71A438`.
//!
//! In:  nothing
//! Out: eax the side hit's figure, ebx 4, esi the last table; the rest kept; no flags
//!      read (the caller calls on)
//!
//! Caller: `physics::damage_step`.
//! Self-test: `physics/damage_model.rs` (`damage_test`).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

#[allow(unused_imports)] // the generated body uses some of them
use crate::math::x87::mul;
#[allow(unused_imports)]
use crate::math::x87_run::{abs_compare, compare_full};
use crate::physics::damage_model::ONE;
#[allow(unused_imports)]
use crate::physics::flow::{a, base, g, hold_high, hold_low, p, table_lerp, x87_run};

#[allow(dead_code)]
fn rd(x: u32) -> u32 {
    unsafe { (x as *const u32).read_unaligned() }
}

/// The steps (generated from the disassembly by idiom; the flagged places by hand).
pub fn steps() {
    let _ = ONE;
    x87_run!(
        "fild dword ptr [{rc} + 0x31A008]", // fild    dword ptr unk_71A008
        "fdiv dword ptr [{m} + 0x35B35C]", // fdiv    flt_75B35C
        "fstp dword ptr [{m} + 0x35B3F8]", // fstp    flt_75B3F8
        "fild dword ptr [{rc} + 0x31A00C]", // fild    dword ptr unk_71A00C
        "fdiv dword ptr [{m} + 0x35B35C]", // fdiv    flt_75B35C
        "fstp dword ptr [{m} + 0x35B3FC]", // fstp    flt_75B3FC
        "fild dword ptr [{rc} + 0x319FF8]", // fild    dword ptr unk_719FF8
        "fdiv dword ptr [{m} + 0x35B35C]", // fdiv    flt_75B35C
        "fstp dword ptr [{m} + 0x35B400]";
        // fstp    flt_75B400
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    table_lerp(0x71A30C, 0x75B4C8, 0x75B350, 0x75B474);
    x87_run!(
        "fld dword ptr [{m} + 0x35B3FC]", // fld     flt_75B3FC
        "fmul dword ptr [{m} + 0x35B34C]", // fmul    flt_75B34C
        "fsub dword ptr [{m} + 0x35B474]", // fsub    flt_75B474
        "fstp dword ptr [{rc} + 0x31A420]";
        // fstp    dword ptr unk_71A420
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    hold_low(0x71A420, 0x75B350);
    hold_high(0x71A420, 0x75B34C);
    x87_run!(
        "fld dword ptr [{m} + 0x35B3F8]", // fld     flt_75B3F8
        "fmul dword ptr [{m} + 0x35B34C]", // fmul    flt_75B34C
        "fstp dword ptr [{rc} + 0x31A410]";
        // fstp    dword ptr unk_71A410
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    if compare_full(g(0x71A30C), g(0x75B34C)) & 0x4000 != 0 {
        x87_run!(
            "fld dword ptr [{m} + 0x35B34C]", // fld     flt_75B34C
            "fstp dword ptr [{rc} + 0x31A420]", // fstp    dword ptr unk_71A420
            "fld dword ptr [{m} + 0x35B350]", // fld     flt_75B350
            "fstp dword ptr [{rc} + 0x31A410]";
            // fstp    dword ptr unk_71A410
            m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
        );
    }
    table_lerp(0x71A310, 0x75B4C8, 0x75B350, 0x75B478);
    x87_run!(
        "fld dword ptr [{m} + 0x35B3FC]", // fld     flt_75B3FC
        "fmul dword ptr [{m} + 0x35B34C]", // fmul    flt_75B34C
        "fsub dword ptr [{m} + 0x35B478]", // fsub    flt_75B478
        "fstp dword ptr [{rc} + 0x31A424]";
        // fstp    dword ptr unk_71A424
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    hold_low(0x71A424, 0x75B350);
    hold_high(0x71A424, 0x75B34C);
    x87_run!(
        "fld dword ptr [{m} + 0x35B3F8]", // fld     flt_75B3F8
        "fmul dword ptr [{m} + 0x35B34C]", // fmul    flt_75B34C
        "fstp dword ptr [{rc} + 0x31A414]";
        // fstp    dword ptr unk_71A414
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    if compare_full(g(0x71A310), g(0x75B34C)) & 0x4000 != 0 {
        x87_run!(
            "fld dword ptr [{m} + 0x35B34C]", // fld     flt_75B34C
            "fstp dword ptr [{rc} + 0x31A424]", // fstp    dword ptr unk_71A424
            "fld dword ptr [{m} + 0x35B350]", // fld     flt_75B350
            "fstp dword ptr [{rc} + 0x31A414]";
            // fstp    dword ptr unk_71A414
            m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
        );
    }
    table_lerp(0x71A314, 0x75B4C8, 0x75B350, 0x75B47C);
    x87_run!(
        "fld dword ptr [{m} + 0x35B3FC]", // fld     flt_75B3FC
        "fmul dword ptr [{m} + 0x35B34C]", // fmul    flt_75B34C
        "fsub dword ptr [{m} + 0x35B47C]", // fsub    flt_75B47C
        "fstp dword ptr [{rc} + 0x31A428]";
        // fstp    dword ptr unk_71A428
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    hold_low(0x71A428, 0x75B350);
    hold_high(0x71A428, 0x75B34C);
    x87_run!(
        "fld dword ptr [{m} + 0x35B3F8]", // fld     flt_75B3F8
        "fmul dword ptr [{m} + 0x35B34C]", // fmul    flt_75B34C
        "fstp dword ptr [{rc} + 0x31A418]";
        // fstp    dword ptr unk_71A418
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    if compare_full(g(0x71A314), g(0x75B34C)) & 0x4000 != 0 {
        x87_run!(
            "fld dword ptr [{m} + 0x35B34C]", // fld     flt_75B34C
            "fstp dword ptr [{rc} + 0x31A428]", // fstp    dword ptr unk_71A428
            "fld dword ptr [{m} + 0x35B350]", // fld     flt_75B350
            "fstp dword ptr [{rc} + 0x31A418]";
            // fstp    dword ptr unk_71A418
            m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
        );
    }
    table_lerp(0x71A318, 0x75B4C8, 0x75B350, 0x75B480);
    x87_run!(
        "fld dword ptr [{m} + 0x35B3FC]", // fld     flt_75B3FC
        "fmul dword ptr [{m} + 0x35B34C]", // fmul    flt_75B34C
        "fsub dword ptr [{m} + 0x35B480]", // fsub    flt_75B480
        "fstp dword ptr [{rc} + 0x31A42C]";
        // fstp    dword ptr unk_71A42C
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    hold_low(0x71A42C, 0x75B350);
    hold_high(0x71A42C, 0x75B34C);
    x87_run!(
        "fld dword ptr [{m} + 0x35B3F8]", // fld     flt_75B3F8
        "fmul dword ptr [{m} + 0x35B34C]", // fmul    flt_75B34C
        "fstp dword ptr [{rc} + 0x31A41C]";
        // fstp    dword ptr unk_71A41C
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    if compare_full(g(0x71A318), g(0x75B34C)) & 0x4000 != 0 {
        x87_run!(
            "fld dword ptr [{m} + 0x35B34C]", // fld     flt_75B34C
            "fstp dword ptr [{rc} + 0x31A42C]", // fstp    dword ptr unk_71A42C
            "fld dword ptr [{m} + 0x35B350]", // fld     flt_75B350
            "fstp dword ptr [{rc} + 0x31A41C]";
            // fstp    dword ptr unk_71A41C
            m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
        );
    }
    x87_run!(
        "fld dword ptr [{rc} + 0x31A410]", // fld     dword ptr unk_71A410
        "fadd dword ptr [{rc} + 0x31A414]", // fadd    dword ptr unk_71A414
        "fadd dword ptr [{rc} + 0x31A418]", // fadd    dword ptr unk_71A418
        "fadd dword ptr [{rc} + 0x31A41C]", // fadd    dword ptr unk_71A41C
        "fmul dword ptr [{m} + 0x35B364]", // fmul    flt_75B364
        "fstp dword ptr [{rc} + 0x31A434]", // fstp    dword ptr unk_71A434
        "fld dword ptr [{rc} + 0x31A420]", // fld     dword ptr unk_71A420
        "fadd dword ptr [{rc} + 0x31A424]", // fadd    dword ptr unk_71A424
        "fadd dword ptr [{rc} + 0x31A428]", // fadd    dword ptr unk_71A428
        "fadd dword ptr [{rc} + 0x31A42C]", // fadd    dword ptr unk_71A42C
        "fmul dword ptr [{m} + 0x35B364]", // fmul    flt_75B364
        "fstp dword ptr [{rc} + 0x31A430]", // fstp    dword ptr unk_71A430
        "fld dword ptr [{rc} + 0x31A018]", // fld     dword ptr g_carDriveKind
        "fstp dword ptr [{m} + 0x35B3E4]", // fstp    flt_75B3E4
        "fld dword ptr [{rc} + 0x31A1B0]", // fld     dword ptr g_carEngineSpeed
        ".byte 0xD9, 0xE1", // fabs
        "fstp dword ptr [{m} + 0x35B36C]";
        // fstp    flt_75B36C
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    hold_high(0x75B36C, 0x71A1B4);
    x87_run!(
        "fld dword ptr [{m} + 0x35B36C]", // fld     flt_75B36C
        "fdiv dword ptr [{rc} + 0x31A1B4]", // fdiv    dword ptr unk_71A1B4
        "fmul dword ptr [{m} + 0x35B34C]", // fmul    flt_75B34C
        "fstp dword ptr [{m} + 0x35B398]", // fstp    flt_75B398
        "fld dword ptr [{rc} + 0x31A184]", // fld     dword ptr g_carSpeed
        ".byte 0xD9, 0xE1", // fabs
        "fstp dword ptr [{m} + 0x35B36C]";
        // fstp    flt_75B36C
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    hold_high(0x75B36C, 0x75B378);
    x87_run!(
        "fld dword ptr [{m} + 0x35B36C]", // fld     flt_75B36C
        "fdiv dword ptr [{m} + 0x35B378]", // fdiv    flt_75B378
        "fmul dword ptr [{m} + 0x35B34C]", // fmul    flt_75B34C
        "fstp dword ptr [{m} + 0x35B39C]", // fstp    flt_75B39C
        "fld dword ptr [{rc} + 0x31A930]", // fld     dword ptr unk_71A930
        "fstp dword ptr [{m} + 0x35B3E0]", // fstp    flt_75B3E0
        "fld dword ptr [{rc} + 0x31A184]", // fld     dword ptr g_carSpeed
        "fsub dword ptr [{rc} + 0x31A190]", // fsub    dword ptr g_carDriveSpeed
        ".byte 0xD9, 0xE1", // fabs
        "fstp dword ptr [{m} + 0x35B36C]";
        // fstp    flt_75B36C
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    hold_high(0x75B36C, 0x75B378);
    x87_run!(
        "fld dword ptr [{m} + 0x35B36C]", // fld     flt_75B36C
        "fdiv dword ptr [{m} + 0x35B378]", // fdiv    flt_75B378
        "fstp dword ptr [{m} + 0x35B3A0]", // fstp    flt_75B3A0
        "fld dword ptr [{rc} + 0x31A184]", // fld     dword ptr g_carSpeed
        "fsub dword ptr [{rc} + 0x31A194]", // fsub    dword ptr g_carDriveSpeed2
        ".byte 0xD9, 0xE1", // fabs
        "fstp dword ptr [{m} + 0x35B36C]";
        // fstp    flt_75B36C
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    hold_high(0x75B36C, 0x75B378);
    x87_run!(
        "fld dword ptr [{m} + 0x35B36C]", // fld     flt_75B36C
        "fdiv dword ptr [{m} + 0x35B378]", // fdiv    flt_75B378
        "fstp dword ptr [{m} + 0x35B3A4]", // fstp    flt_75B3A4
        "fld dword ptr [{rc} + 0x31A190]", // fld     dword ptr g_carDriveSpeed
        "fsub dword ptr [{rc} + 0x31A184]", // fsub    dword ptr g_carSpeed
        "fmul dword ptr [{m} + 0x35B6F4]", // fmul    flt_75B6F4
        "fadd dword ptr [{rc} + 0x31A190]", // fadd    dword ptr g_carDriveSpeed
        ".byte 0xD9, 0xE1", // fabs
        "fstp dword ptr [{m} + 0x35B36C]";
        // fstp    flt_75B36C
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    hold_high(0x75B36C, 0x75B378);
    x87_run!(
        "fld dword ptr [{m} + 0x35B36C]", // fld     flt_75B36C
        "fdiv dword ptr [{m} + 0x35B378]", // fdiv    flt_75B378
        "fstp dword ptr [{m} + 0x35B3A8]", // fstp    flt_75B3A8
        "fld dword ptr [{rc} + 0x31A194]", // fld     dword ptr g_carDriveSpeed2
        "fsub dword ptr [{rc} + 0x31A184]", // fsub    dword ptr g_carSpeed
        "fmul dword ptr [{m} + 0x35B6F4]", // fmul    flt_75B6F4
        "fadd dword ptr [{rc} + 0x31A194]", // fadd    dword ptr g_carDriveSpeed2
        ".byte 0xD9, 0xE1", // fabs
        "fstp dword ptr [{m} + 0x35B36C]";
        // fstp    flt_75B36C
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    hold_high(0x75B36C, 0x75B378);
    x87_run!(
        "fld dword ptr [{m} + 0x35B36C]", // fld     flt_75B36C
        "fdiv dword ptr [{m} + 0x35B378]", // fdiv    flt_75B378
        "fstp dword ptr [{m} + 0x35B3AC]", // fstp    flt_75B3AC
        ".byte 0xD9, 0xEE", // fldz
        "fstp dword ptr [{m} + 0x35B3F4]"; // fstp    flt_75B3F4
        m = crate::physics::store::asm_base()
    );
    if g(0x71A22C) == 0x0 {
        x87_run!(
            ".byte 0xD9, 0xE8", // fld1
            "fstp dword ptr [{m} + 0x35B3F4]"; // fstp    flt_75B3F4
            m = crate::physics::store::asm_base()
        );
    }
    x87_run!(
        ".byte 0xD9, 0xEE", // fldz
        "fstp dword ptr [{m} + 0x35B3B0]"; // fstp    flt_75B3B0
        m = crate::physics::store::asm_base()
    );
    if abs_compare(g(0x71A184), g(0x75B348)) & 0x100 == 0
        && (g(0x71A614).wrapping_sub(0x10) as i32) >= 0
    {
        x87_run!(
            "fild dword ptr [{rc} + 0x31A614]", // fild    dword ptr unk_71A614
            "fdiv dword ptr [{m} + 0x35B35C]", // fdiv    flt_75B35C
            "fstp dword ptr [{m} + 0x35B3B0]";
            // fstp    flt_75B3B0
            m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
        );
    }
    // 0x4ABCAA: 1 when the engine's speed is above the limit times `flt_75B37C`.
    p(0x75B3C0, 0);
    let mut status = 0u16;
    x87_run!(
        "fld dword ptr [{rc} + 0x31A1B4]", // fld     dword ptr unk_71A1B4
        "fmul dword ptr [{m} + 0x35B37C]", // fmul    flt_75B37C
        "fcomp dword ptr [{rc} + 0x31A1B0]", // fcomp   dword ptr g_carEngineSpeed
        "fnstsw word ptr [{s}]";
        m = crate::physics::store::asm_base(), s = &raw mut status, rc = crate::race::store::asm_base()
    );
    if status & 0x100 != 0 {
        p(0x75B3C0, ONE);
    }
    p(0x75B3B8, g(0x71A210));
    p(0x75B3BC, g(0x71A1AC));
    p(0x75B3E8, g(0x71A004));
    let mode = g(0x71A908);
    p(0x75B3EC, mode);
    x87_run!(
        ".byte 0xD9, 0xEE", // fldz
        "fstp dword ptr [{m} + 0x35B3B4]"; // fstp    flt_75B3B4
        m = crate::physics::store::asm_base()
    );
    if mode == 5 {
        p(0x75B3B4, ONE);
    }
    p(0x75B3F0, g(0x71A258));
    let mut hit = g(0x71A208);
    if hit != 0 {
        hit = 1;
        if compare_full(g(0x71A1FC), g(0x75B350)) & 0x100 == 0 {
            hit += 1;
        }
        if compare_full(g(0x71A204), g(0x75B350)) & 0x100 != 0 {
            hit += 2;
        }
    }
    p(0x75B3C4, hit);
    x87_run!(
        "fld dword ptr [{rc} + 0x31A208]", // fld     dword ptr unk_71A208
        "fmul dword ptr [{rc} + 0x31A208]", // fmul    dword ptr unk_71A208
        "fdiv dword ptr [{m} + 0x35B394]", // fdiv    flt_75B394
        "fdiv dword ptr [{m} + 0x35B378]", // fdiv    flt_75B378
        "fstp dword ptr [{m} + 0x35B3C8]", // fstp    flt_75B3C8
        "fld dword ptr [{rc} + 0x31A180]", // fld     dword ptr unk_71A180
        ".byte 0xD9, 0xE0", // fchs
        "fsub dword ptr [{m} + 0x35B624]", // fsub    flt_75B624
        "fstp dword ptr [{m} + 0x35B36C]";
        // fstp    flt_75B36C
        m = crate::physics::store::asm_base(), rc = crate::race::store::asm_base()
    );
    hold_low(0x75B36C, 0x75B350);
    hold_high(0x75B36C, 0x75B354);
    x87_run!(
        "fld dword ptr [{m} + 0x35B36C]", // fld     flt_75B36C
        ".byte 0xD8, 0xC8", // fmul    st, st
        "fmul dword ptr [{m} + 0x35B620]", // fmul    flt_75B620
        "fstp dword ptr [{m} + 0x35B36C]"; // fstp    flt_75B36C
        m = crate::physics::store::asm_base()
    );
    for k in 0..4usize {
        p(0x75B3CC + 4 * k, 0);
        if g(0x71A5E8 + 4 * k) == 0 && g(0x71A5D8 + 4 * k) != 0 {
            p(0x75B3CC + 4 * k, mul(g(0x71A5D8 + 4 * k), g(0x75B36C)));
        }
    }
    x87_run!(
        "fld dword ptr [{m} + 0x35B3CC]", // fld     dword_75B3CC
        "fadd dword ptr [{m} + 0x35B3D0]", // fadd    flt_75B3D0
        "fadd dword ptr [{m} + 0x35B3D4]", // fadd    flt_75B3D4
        "fadd dword ptr [{m} + 0x35B3D8]", // fadd    flt_75B3D8
        "fmul dword ptr [{m} + 0x35B364]", // fmul    flt_75B364
        "fstp dword ptr [{m} + 0x35B3DC]", // fstp    flt_75B3DC
        "fld dword ptr [{m} + 0x35B3DC]", // fld     flt_75B3DC
        "fmul dword ptr [{m} + 0x35B360]", // fmul    flt_75B360
        "fstp dword ptr [{m} + 0x35B36C]", // fstp    flt_75B36C
        "fld dword ptr [{m} + 0x35B3CC]", // fld     dword_75B3CC
        "fmul dword ptr [{m} + 0x35B360]", // fmul    flt_75B360
        "fadd dword ptr [{m} + 0x35B36C]", // fadd    flt_75B36C
        "fstp dword ptr [{m} + 0x35B3CC]", // fstp    dword_75B3CC
        "fld dword ptr [{m} + 0x35B3D0]", // fld     flt_75B3D0
        "fmul dword ptr [{m} + 0x35B360]", // fmul    flt_75B360
        "fadd dword ptr [{m} + 0x35B36C]", // fadd    flt_75B36C
        "fstp dword ptr [{m} + 0x35B3D0]", // fstp    flt_75B3D0
        "fld dword ptr [{m} + 0x35B3D4]", // fld     flt_75B3D4
        "fmul dword ptr [{m} + 0x35B360]", // fmul    flt_75B360
        "fadd dword ptr [{m} + 0x35B36C]", // fadd    flt_75B36C
        "fstp dword ptr [{m} + 0x35B3D4]", // fstp    flt_75B3D4
        "fld dword ptr [{m} + 0x35B3D8]", // fld     flt_75B3D8
        "fmul dword ptr [{m} + 0x35B360]", // fmul    flt_75B360
        "fadd dword ptr [{m} + 0x35B36C]", // fadd    flt_75B36C
        "fstp dword ptr [{m} + 0x35B3D8]"; // fstp    flt_75B3D8
        m = crate::physics::store::asm_base()
    );
    if g(0x75B3C8) != 0x0 && g(0x71A438) != 0x0 {
        p(0x75B3C8, 0x0);
        p(0x71A438, 0x64);
    }
    // retn
}
