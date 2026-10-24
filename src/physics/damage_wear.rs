//! `physics::damage_wear` (0x4ABEDF): the wear of this step (`physics/damage_model.rs`):
//! the clock (`unk_719FE4`) into `dword_75B4A0`; table lerps of the speed shares
//! (`unk_75B4A8` .., into `flt_75B45C` ..); the gauges raised and lowered by rates
//! (`flt_75B584` ..) times the shares and `flt_75B390`, each held 0 ..= `flt_75B34C`;
//! the hit figures `flt_75B450` .. `flt_75B458`; and `flt_75B484` from the table
//! `dword_75B73C` (row `dword_75B3E8`, two columns through `dword_75B8C0` / `75B8C4`,
//! as the original indexes them) blended by `dword_75B3F0`, x `flt_75B8BC`.
//!
//! In:  nothing
//! Out: eax the second column, ebx the row's offset, esi the last table; the rest kept;
//!      no flags read
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
    p(0x75B4A0, g(0x719FE4));
    table_lerp(0x75B398, 0x75B4A8, 0x75B350, 0x75B45C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B584]", // fld     flt_75B584
        "fmul dword ptr [{b} + 0x35B3B0]", // fmul    flt_75B3B0
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        "fadd dword ptr [{b} + 0x31A284]", // fadd    dword ptr unk_71A284
        "fstp dword ptr [{b} + 0x31A284]"; // fstp    dword ptr unk_71A284
        b = base()
    );
    hold_low(0x71A284, 0x75B350);
    hold_high(0x71A284, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B584]", // fld     flt_75B584
        "fmul dword ptr [{b} + 0x35B3B0]", // fmul    flt_75B3B0
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        "fadd dword ptr [{b} + 0x31A288]", // fadd    dword ptr unk_71A288
        "fstp dword ptr [{b} + 0x31A288]"; // fstp    dword ptr unk_71A288
        b = base()
    );
    hold_low(0x71A288, 0x75B350);
    hold_high(0x71A288, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B588]", // fld     flt_75B588
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        ".byte 0xD9, 0xE0", // fchs
        "fadd dword ptr [{b} + 0x31A284]", // fadd    dword ptr unk_71A284
        "fstp dword ptr [{b} + 0x31A284]"; // fstp    dword ptr unk_71A284
        b = base()
    );
    hold_low(0x71A284, 0x75B350);
    hold_high(0x71A284, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B588]", // fld     flt_75B588
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        ".byte 0xD9, 0xE0", // fchs
        "fadd dword ptr [{b} + 0x31A288]", // fadd    dword ptr unk_71A288
        "fstp dword ptr [{b} + 0x31A288]"; // fstp    dword ptr unk_71A288
        b = base()
    );
    hold_low(0x71A288, 0x75B350);
    hold_high(0x71A288, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B58C]", // fld     flt_75B58C
        "fmul dword ptr [{b} + 0x35B3A8]", // fmul    flt_75B3A8
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        "fadd dword ptr [{b} + 0x31A27C]", // fadd    dword ptr unk_71A27C
        "fstp dword ptr [{b} + 0x31A27C]"; // fstp    dword ptr unk_71A27C
        b = base()
    );
    hold_low(0x71A27C, 0x75B350);
    hold_high(0x71A27C, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B58C]", // fld     flt_75B58C
        "fmul dword ptr [{b} + 0x35B3AC]", // fmul    flt_75B3AC
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        "fadd dword ptr [{b} + 0x31A280]", // fadd    dword ptr unk_71A280
        "fstp dword ptr [{b} + 0x31A280]"; // fstp    dword ptr unk_71A280
        b = base()
    );
    hold_low(0x71A280, 0x75B350);
    hold_high(0x71A280, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B590]", // fld     flt_75B590
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        ".byte 0xD9, 0xE0", // fchs
        "fadd dword ptr [{b} + 0x31A27C]", // fadd    dword ptr unk_71A27C
        "fstp dword ptr [{b} + 0x31A27C]"; // fstp    dword ptr unk_71A27C
        b = base()
    );
    hold_low(0x71A27C, 0x75B350);
    hold_high(0x71A27C, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B590]", // fld     flt_75B590
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        ".byte 0xD9, 0xE0", // fchs
        "fadd dword ptr [{b} + 0x31A280]", // fadd    dword ptr unk_71A280
        "fstp dword ptr [{b} + 0x31A280]"; // fstp    dword ptr unk_71A280
        b = base()
    );
    hold_low(0x71A280, 0x75B350);
    hold_high(0x71A280, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B594]", // fld     flt_75B594
        "fmul dword ptr [{b} + 0x35B3B4]", // fmul    flt_75B3B4
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        "fadd dword ptr [{b} + 0x31A278]", // fadd    dword ptr unk_71A278
        "fstp dword ptr [{b} + 0x31A278]"; // fstp    dword ptr unk_71A278
        b = base()
    );
    hold_low(0x71A278, 0x75B350);
    hold_high(0x71A278, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B598]", // fld     flt_75B598
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        ".byte 0xD9, 0xE0", // fchs
        "fadd dword ptr [{b} + 0x31A278]", // fadd    dword ptr unk_71A278
        "fstp dword ptr [{b} + 0x31A278]"; // fstp    dword ptr unk_71A278
        b = base()
    );
    hold_low(0x71A278, 0x75B350);
    hold_high(0x71A278, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B59C]", // fld     flt_75B59C
        "fmul dword ptr [{b} + 0x35B3B0]", // fmul    flt_75B3B0
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        ".byte 0xD9, 0xE0", // fchs
        "fadd dword ptr [{b} + 0x31A278]", // fadd    dword ptr unk_71A278
        "fstp dword ptr [{b} + 0x31A278]"; // fstp    dword ptr unk_71A278
        b = base()
    );
    hold_low(0x71A278, 0x75B350);
    hold_high(0x71A278, 0x75B34C);
    table_lerp(0x71A29C, 0x75B4B8, 0x75B350, 0x75B460);
    x87_run!(
        "fld dword ptr [{b} + 0x35B398]", // fld     flt_75B398
        "fsub dword ptr [{b} + 0x35B460]", // fsub    flt_75B460
        "fstp dword ptr [{b} + 0x35B464]"; // fstp    flt_75B464
        b = base()
    );
    hold_low(0x75B464, 0x75B350);
    x87_run!(
        "fld dword ptr [{b} + 0x35B578]", // fld     flt_75B578
        "fmul dword ptr [{b} + 0x35B464]", // fmul    flt_75B464
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        "fadd dword ptr [{b} + 0x31A28C]", // fadd    dword ptr unk_71A28C
        "fstp dword ptr [{b} + 0x31A28C]"; // fstp    dword ptr unk_71A28C
        b = base()
    );
    hold_low(0x71A28C, 0x75B350);
    hold_high(0x71A28C, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B57C]", // fld     flt_75B57C
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        ".byte 0xD9, 0xE0", // fchs
        "fadd dword ptr [{b} + 0x31A28C]", // fadd    dword ptr unk_71A28C
        "fstp dword ptr [{b} + 0x31A28C]"; // fstp    dword ptr unk_71A28C
        b = base()
    );
    hold_low(0x71A28C, 0x75B350);
    hold_high(0x71A28C, 0x75B34C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B580]", // fld     flt_75B580
        "fmul dword ptr [{b} + 0x35B39C]", // fmul    flt_75B39C
        "fmul dword ptr [{b} + 0x35B390]", // fmul    flt_75B390
        ".byte 0xD9, 0xE0", // fchs
        "fadd dword ptr [{b} + 0x31A28C]", // fadd    dword ptr unk_71A28C
        "fstp dword ptr [{b} + 0x31A28C]"; // fstp    dword ptr unk_71A28C
        b = base()
    );
    hold_low(0x71A28C, 0x75B350);
    hold_high(0x71A28C, 0x75B34C);
    table_lerp(0x71A430, 0x75B660, 0x75B350, 0x75B44C);
    x87_run!(
        "fld dword ptr [{b} + 0x35B44C]", // fld     flt_75B44C
        ".byte 0xD8, 0xC8", // fmul    st, st
        "fstp dword ptr [{b} + 0x35B44C]"; // fstp    flt_75B44C
        b = base()
    );
    table_lerp(0x71A430, 0x75B640, 0x75B350, 0x75B448);
    x87_run!(
        "fld dword ptr [{b} + 0x35B448]", // fld     flt_75B448
        ".byte 0xD8, 0xC8", // fmul    st, st
        "fstp dword ptr [{b} + 0x35B448]"; // fstp    flt_75B448
        b = base()
    );
    table_lerp(0x71A434, 0x75B650, 0x75B350, 0x75B468);
    x87_run!(
        "fld dword ptr [{b} + 0x35B468]", // fld     flt_75B468
        ".byte 0xD8, 0xC8", // fmul    st, st
        "fstp dword ptr [{b} + 0x35B468]"; // fstp    flt_75B468
        b = base()
    );
    table_lerp(0x75B3A0, 0x75B4D8, 0x75B350, 0x75B46C);
    table_lerp(0x75B3A4, 0x75B4D8, 0x75B350, 0x75B470);
    x87_run!(
        "fld dword ptr [{b} + 0x35B3E0]", // fld     flt_75B3E0
        "fmul dword ptr [{b} + 0x35B39C]", // fmul    flt_75B39C
        "fmul dword ptr [{b} + 0x35B6F8]", // fmul    flt_75B6F8
        "fmul dword ptr [{b} + 0x35B44C]", // fmul    flt_75B44C
        "fmul dword ptr [{b} + 0x35B468]", // fmul    flt_75B468
        "fmul dword ptr [{b} + 0x35B3F4]", // fmul    flt_75B3F4
        "fstp dword ptr [{b} + 0x35B454]", // fstp    flt_75B454
        "fld dword ptr [{b} + 0x35B3E0]", // fld     flt_75B3E0
        "fmul dword ptr [{b} + 0x35B39C]", // fmul    flt_75B39C
        "fmul dword ptr [{b} + 0x35B6F8]", // fmul    flt_75B6F8
        "fmul dword ptr [{b} + 0x35B3F4]", // fmul    flt_75B3F4
        "fst dword ptr [{b} + 0x35B450]", // fst     flt_75B450
        "fmul dword ptr [{b} + 0x35B448]", // fmul    flt_75B448
        "fstp dword ptr [{b} + 0x35B458]"; // fstp    flt_75B458
        b = base()
    );
    // 0x4AC7C9: two figures of the table `dword_75B73C` (row `dword_75B3E8`, columns
    // from `dword_75B8C0` by `dword_75B3EC` & 0xF, then from `dword_75B8C4` by that
    // column, as the original), blended by `dword_75B3F0`, x `flt_75B8BC`.
    let row = g(0x75B3E8) << 6;
    let column = rd(a(0x75B8C0).wrapping_add((g(0x75B3EC) & 0xF).wrapping_mul(8)));
    let first = a(0x75B73C)
        .wrapping_add(row)
        .wrapping_add(column.wrapping_mul(4));
    let column = rd(a(0x75B8C4).wrapping_add(column.wrapping_mul(8)));
    let second = a(0x75B73C)
        .wrapping_add(row)
        .wrapping_add(column.wrapping_mul(4));
    x87_run!(
        "fild dword ptr [{f}]",
        ".byte 0xD9, 0xE8", // fld1
        "fsub dword ptr [{b} + 0x35B3F0]",
        ".byte 0xDE, 0xC9", // fmulp st(1), st
        "fild dword ptr [{s}]",
        "fld dword ptr [{b} + 0x35B3F0]",
        ".byte 0xDE, 0xC9", // fmulp st(1), st
        ".byte 0xDE, 0xC1", // faddp st(1), st
        "fmul dword ptr [{b} + 0x35B8BC]",
        "fstp dword ptr [{b} + 0x35B484]";
        b = base(), f = first, s = second
    );
}
