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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

#[allow(unused_imports)] // the generated body uses some of them
use crate::math::x87::mul;
use crate::math::x87_float::X;
#[allow(unused_imports)]
use crate::math::x87_run::{abs_compare, compare_full};
#[allow(unused_imports)]
use crate::physics::flow::{a, g, hold_high, hold_low, p, table_lerp};
use crate::scene::x87_mem::{fild, fld, fstp};

#[allow(dead_code)]
fn rd(x: u32) -> u32 {
    unsafe { (x as *const u32).read_unaligned() }
}

/// The steps (generated from the disassembly by idiom; the flagged places by hand).
pub fn steps() {
    let _ = X(1.0);
    p(0x75B4A0, g(0x719FE4));
    table_lerp(0x75B398, 0x75B4A8, 0x75B350, 0x75B45C);
    x87_1(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B584,
        0x35B3B0,
        0x31A284,
        0x31A284,
    );
    hold_low(0x71A284, 0x75B350);
    hold_high(0x71A284, 0x75B34C);
    x87_1(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B584,
        0x35B3B0,
        0x31A288,
        0x31A288,
    );
    hold_low(0x71A288, 0x75B350);
    hold_high(0x71A288, 0x75B34C);
    x87_2(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B588,
        0x31A284,
        0x31A284,
    );
    hold_low(0x71A284, 0x75B350);
    hold_high(0x71A284, 0x75B34C);
    x87_2(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B588,
        0x31A288,
        0x31A288,
    );
    hold_low(0x71A288, 0x75B350);
    hold_high(0x71A288, 0x75B34C);
    x87_1(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B58C,
        0x35B3A8,
        0x31A27C,
        0x31A27C,
    );
    hold_low(0x71A27C, 0x75B350);
    hold_high(0x71A27C, 0x75B34C);
    x87_1(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B58C,
        0x35B3AC,
        0x31A280,
        0x31A280,
    );
    hold_low(0x71A280, 0x75B350);
    hold_high(0x71A280, 0x75B34C);
    x87_2(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B590,
        0x31A27C,
        0x31A27C,
    );
    hold_low(0x71A27C, 0x75B350);
    hold_high(0x71A27C, 0x75B34C);
    x87_2(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B590,
        0x31A280,
        0x31A280,
    );
    hold_low(0x71A280, 0x75B350);
    hold_high(0x71A280, 0x75B34C);
    x87_1(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B594,
        0x35B3B4,
        0x31A278,
        0x31A278,
    );
    hold_low(0x71A278, 0x75B350);
    hold_high(0x71A278, 0x75B34C);
    x87_2(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B598,
        0x31A278,
        0x31A278,
    );
    hold_low(0x71A278, 0x75B350);
    hold_high(0x71A278, 0x75B34C);
    x87_3(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B59C,
        0x35B3B0,
        0x31A278,
        0x31A278,
    );
    hold_low(0x71A278, 0x75B350);
    hold_high(0x71A278, 0x75B34C);
    table_lerp(0x71A29C, 0x75B4B8, 0x75B350, 0x75B460);
    x87_4(crate::physics::store::asm_base() as usize);
    hold_low(0x75B464, 0x75B350);
    x87_1(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B578,
        0x35B464,
        0x31A28C,
        0x31A28C,
    );
    hold_low(0x71A28C, 0x75B350);
    hold_high(0x71A28C, 0x75B34C);
    x87_2(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B57C,
        0x31A28C,
        0x31A28C,
    );
    hold_low(0x71A28C, 0x75B350);
    hold_high(0x71A28C, 0x75B34C);
    x87_3(
        crate::physics::store::asm_base() as usize,
        crate::race::store::asm_base() as usize,
        0x35B580,
        0x35B39C,
        0x31A28C,
        0x31A28C,
    );
    hold_low(0x71A28C, 0x75B350);
    hold_high(0x71A28C, 0x75B34C);
    table_lerp(0x71A430, 0x75B660, 0x75B350, 0x75B44C);
    x87_5(
        crate::physics::store::asm_base() as usize,
        0x35B44C,
        0x35B44C,
    );
    table_lerp(0x71A430, 0x75B640, 0x75B350, 0x75B448);
    x87_5(
        crate::physics::store::asm_base() as usize,
        0x35B448,
        0x35B448,
    );
    table_lerp(0x71A434, 0x75B650, 0x75B350, 0x75B468);
    x87_5(
        crate::physics::store::asm_base() as usize,
        0x35B468,
        0x35B468,
    );
    table_lerp(0x75B3A0, 0x75B4D8, 0x75B350, 0x75B46C);
    table_lerp(0x75B3A4, 0x75B4D8, 0x75B350, 0x75B470);
    x87_6(crate::physics::store::asm_base() as usize);
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
    x87_7(
        first as usize,
        crate::physics::store::asm_base() as usize,
        second as usize,
    );
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o4] = (([m + o0] * [m + o1]) * [m + 0x35B390]) + [rc + o3]`
fn x87_1(m: usize, rc: usize, o0: usize, o1: usize, o3: usize, o4: usize) {
    unsafe {
        let m = m as *const u8;
        let rc = rc as *mut u8;
        let t0 = fld(m, o0);
        let t1 = fld(m, o1);
        let t2 = t0 * t1;
        let t3 = fld(m, 0x35B390);
        let t4 = t2 * t3;
        let t5 = fld(rc, o3);
        let t6 = t4 + t5;
        fstp(rc, o4, t6);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o3] = -([m + o0] * [m + 0x35B390]) + [rc + o2]`
fn x87_2(m: usize, rc: usize, o0: usize, o2: usize, o3: usize) {
    unsafe {
        let m = m as *const u8;
        let rc = rc as *mut u8;
        let t0 = fld(m, o0);
        let t1 = fld(m, 0x35B390);
        let t2 = t0 * t1;
        let t3 = -t2;
        let t4 = fld(rc, o2);
        let t5 = t3 + t4;
        fstp(rc, o3, t5);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o4] = -(([m + o0] * [m + o1]) * [m + 0x35B390]) + [rc + o3]`
fn x87_3(m: usize, rc: usize, o0: usize, o1: usize, o3: usize, o4: usize) {
    unsafe {
        let m = m as *const u8;
        let rc = rc as *mut u8;
        let t0 = fld(m, o0);
        let t1 = fld(m, o1);
        let t2 = t0 * t1;
        let t3 = fld(m, 0x35B390);
        let t4 = t2 * t3;
        let t5 = -t4;
        let t6 = fld(rc, o3);
        let t7 = t5 + t6;
        fstp(rc, o4, t7);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B464] = [m + 0x35B398] - [m + 0x35B460]`
fn x87_4(m: usize) {
    unsafe {
        let m = m as *mut u8;
        let t0 = fld(m, 0x35B398);
        let t1 = fld(m, 0x35B460);
        let t2 = t0 - t1;
        fstp(m, 0x35B464, t2);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + o1] = [m + o0] * [m + o0]`
fn x87_5(m: usize, o0: usize, o1: usize) {
    unsafe {
        let m = m as *mut u8;
        let t0 = fld(m, o0);
        let t1 = t0 * t0;
        fstp(m, o1, t1);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B454] = (((([m + 0x35B3E0] * [m + 0x35B39C]) * [m + 0x35B6F8]) * [m + 0x35B44C]) * [m + 0x35B468]) * [m + 0x35B3F4]`
/// `[m + 0x35B450] = (([m + 0x35B3E0] * [m + 0x35B39C]) * [m + 0x35B6F8]) * [m + 0x35B3F4]`
/// `[m + 0x35B458] = ((([m + 0x35B3E0] * [m + 0x35B39C]) * [m + 0x35B6F8]) * [m + 0x35B3F4]) * [m + 0x35B448]`
fn x87_6(m: usize) {
    unsafe {
        let m = m as *mut u8;
        let t0 = fld(m, 0x35B3E0);
        let t1 = fld(m, 0x35B39C);
        let t2 = t0 * t1;
        let t3 = fld(m, 0x35B6F8);
        let t4 = t2 * t3;
        let t5 = fld(m, 0x35B44C);
        let t6 = t4 * t5;
        let t7 = fld(m, 0x35B468);
        let t8 = t6 * t7;
        let t9 = fld(m, 0x35B3F4);
        let t10 = t8 * t9;
        fstp(m, 0x35B454, t10);
        let t11 = fld(m, 0x35B3E0);
        let t12 = fld(m, 0x35B39C);
        let t13 = t11 * t12;
        let t14 = fld(m, 0x35B6F8);
        let t15 = t13 * t14;
        let t16 = fld(m, 0x35B3F4);
        let t17 = t15 * t16;
        fstp(m, 0x35B450, t17);
        let t18 = fld(m, 0x35B448);
        let t19 = t17 * t18;
        fstp(m, 0x35B458, t19);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B484] = ((int [f + 0] * (1 - [m + 0x35B3F0])) + (int [s + 0] * [m + 0x35B3F0])) * [m + 0x35B8BC]`
fn x87_7(f: usize, m: usize, s: usize) {
    unsafe {
        let f = f as *const u8;
        let m = m as *mut u8;
        let s = s as *const u8;
        let t0 = fild(f, 0);
        let t1 = X(1.0);
        let t2 = fld(m, 0x35B3F0);
        let t3 = t1 - t2;
        let t4 = t0 * t3;
        let t5 = fild(s, 0);
        let t6 = fld(m, 0x35B3F0);
        let t7 = t5 * t6;
        let t8 = t4 + t7;
        let t9 = fld(m, 0x35B8BC);
        let t10 = t8 * t9;
        fstp(m, 0x35B484, t10);
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(m, rc) "fld dword ptr [{m} + 0x35B584]", "fmul dword ptr [{m} + 0x35B3B0]", "fmul dword ptr [{m} + 0x35B390]", "fadd dword ptr [{rc} + 0x31A284]", "fstp dword ptr [{rc} + 0x31A284]");
    reference!(ref_1(m, rc) "fld dword ptr [{m} + 0x35B584]", "fmul dword ptr [{m} + 0x35B3B0]", "fmul dword ptr [{m} + 0x35B390]", "fadd dword ptr [{rc} + 0x31A288]", "fstp dword ptr [{rc} + 0x31A288]");
    reference!(ref_2(m, rc) "fld dword ptr [{m} + 0x35B588]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A284]", "fstp dword ptr [{rc} + 0x31A284]");
    reference!(ref_3(m, rc) "fld dword ptr [{m} + 0x35B588]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A288]", "fstp dword ptr [{rc} + 0x31A288]");
    reference!(ref_4(m, rc) "fld dword ptr [{m} + 0x35B58C]", "fmul dword ptr [{m} + 0x35B3A8]", "fmul dword ptr [{m} + 0x35B390]", "fadd dword ptr [{rc} + 0x31A27C]", "fstp dword ptr [{rc} + 0x31A27C]");
    reference!(ref_5(m, rc) "fld dword ptr [{m} + 0x35B58C]", "fmul dword ptr [{m} + 0x35B3AC]", "fmul dword ptr [{m} + 0x35B390]", "fadd dword ptr [{rc} + 0x31A280]", "fstp dword ptr [{rc} + 0x31A280]");
    reference!(ref_6(m, rc) "fld dword ptr [{m} + 0x35B590]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A27C]", "fstp dword ptr [{rc} + 0x31A27C]");
    reference!(ref_7(m, rc) "fld dword ptr [{m} + 0x35B590]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A280]", "fstp dword ptr [{rc} + 0x31A280]");
    reference!(ref_8(m, rc) "fld dword ptr [{m} + 0x35B594]", "fmul dword ptr [{m} + 0x35B3B4]", "fmul dword ptr [{m} + 0x35B390]", "fadd dword ptr [{rc} + 0x31A278]", "fstp dword ptr [{rc} + 0x31A278]");
    reference!(ref_9(m, rc) "fld dword ptr [{m} + 0x35B598]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A278]", "fstp dword ptr [{rc} + 0x31A278]");
    reference!(ref_10(m, rc) "fld dword ptr [{m} + 0x35B59C]", "fmul dword ptr [{m} + 0x35B3B0]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A278]", "fstp dword ptr [{rc} + 0x31A278]");
    reference!(ref_11(m) "fld dword ptr [{m} + 0x35B398]", "fsub dword ptr [{m} + 0x35B460]", "fstp dword ptr [{m} + 0x35B464]");
    reference!(ref_12(m, rc) "fld dword ptr [{m} + 0x35B578]", "fmul dword ptr [{m} + 0x35B464]", "fmul dword ptr [{m} + 0x35B390]", "fadd dword ptr [{rc} + 0x31A28C]", "fstp dword ptr [{rc} + 0x31A28C]");
    reference!(ref_13(m, rc) "fld dword ptr [{m} + 0x35B57C]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A28C]", "fstp dword ptr [{rc} + 0x31A28C]");
    reference!(ref_14(m, rc) "fld dword ptr [{m} + 0x35B580]", "fmul dword ptr [{m} + 0x35B39C]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A28C]", "fstp dword ptr [{rc} + 0x31A28C]");
    reference!(ref_15(m) "fld dword ptr [{m} + 0x35B44C]", ".byte 0xD8, 0xC8", "fstp dword ptr [{m} + 0x35B44C]");
    reference!(ref_16(m) "fld dword ptr [{m} + 0x35B448]", ".byte 0xD8, 0xC8", "fstp dword ptr [{m} + 0x35B448]");
    reference!(ref_17(m) "fld dword ptr [{m} + 0x35B468]", ".byte 0xD8, 0xC8", "fstp dword ptr [{m} + 0x35B468]");
    reference!(ref_18(m) "fld dword ptr [{m} + 0x35B3E0]", "fmul dword ptr [{m} + 0x35B39C]", "fmul dword ptr [{m} + 0x35B6F8]", "fmul dword ptr [{m} + 0x35B44C]", "fmul dword ptr [{m} + 0x35B468]", "fmul dword ptr [{m} + 0x35B3F4]", "fstp dword ptr [{m} + 0x35B454]", "fld dword ptr [{m} + 0x35B3E0]", "fmul dword ptr [{m} + 0x35B39C]", "fmul dword ptr [{m} + 0x35B6F8]", "fmul dword ptr [{m} + 0x35B3F4]", "fst dword ptr [{m} + 0x35B450]", "fmul dword ptr [{m} + 0x35B448]", "fstp dword ptr [{m} + 0x35B458]");
    reference!(ref_19(f, m, s) "fild dword ptr [{f}]", ".byte 0xD9, 0xE8", "fsub dword ptr [{m} + 0x35B3F0]", ".byte 0xDE, 0xC9", "fild dword ptr [{s}]", "fld dword ptr [{m} + 0x35B3F0]", ".byte 0xDE, 0xC9", ".byte 0xDE, 0xC1", "fmul dword ptr [{m} + 0x35B8BC]", "fstp dword ptr [{m} + 0x35B484]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "damage_wear line 36",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B3B0, In::F), (0x35B584, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A284, In::F)],
                        writes: &[(0x31A284, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| x87_1(a[0], a[1], 0x35B584, 0x35B3B0, 0x31A284, 0x31A284),
            },
            Block {
                name: "damage_wear line 47",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B3B0, In::F), (0x35B584, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A288, In::F)],
                        writes: &[(0x31A288, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| x87_1(a[0], a[1], 0x35B584, 0x35B3B0, 0x31A288, 0x31A288),
            },
            Block {
                name: "damage_wear line 58",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B588, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A284, In::F)],
                        writes: &[(0x31A284, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1], 0x35B588, 0x31A284, 0x31A284),
            },
            Block {
                name: "damage_wear line 69",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B588, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A288, In::F)],
                        writes: &[(0x31A288, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1], 0x35B588, 0x31A288, 0x31A288),
            },
            Block {
                name: "damage_wear line 80",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B3A8, In::F), (0x35B58C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A27C, In::F)],
                        writes: &[(0x31A27C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1]) },
                plain: |a| x87_1(a[0], a[1], 0x35B58C, 0x35B3A8, 0x31A27C, 0x31A27C),
            },
            Block {
                name: "damage_wear line 91",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B3AC, In::F), (0x35B58C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A280, In::F)],
                        writes: &[(0x31A280, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0], a[1]) },
                plain: |a| x87_1(a[0], a[1], 0x35B58C, 0x35B3AC, 0x31A280, 0x31A280),
            },
            Block {
                name: "damage_wear line 102",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B590, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A27C, In::F)],
                        writes: &[(0x31A27C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1], 0x35B590, 0x31A27C, 0x31A27C),
            },
            Block {
                name: "damage_wear line 113",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B590, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A280, In::F)],
                        writes: &[(0x31A280, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_7(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1], 0x35B590, 0x31A280, 0x31A280),
            },
            Block {
                name: "damage_wear line 124",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B3B4, In::F), (0x35B594, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A278, In::F)],
                        writes: &[(0x31A278, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_8(a[0], a[1]) },
                plain: |a| x87_1(a[0], a[1], 0x35B594, 0x35B3B4, 0x31A278, 0x31A278),
            },
            Block {
                name: "damage_wear line 135",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B598, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A278, In::F)],
                        writes: &[(0x31A278, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_9(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1], 0x35B598, 0x31A278, 0x31A278),
            },
            Block {
                name: "damage_wear line 146",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B3B0, In::F), (0x35B59C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A278, In::F)],
                        writes: &[(0x31A278, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_10(a[0], a[1]) },
                plain: |a| x87_3(a[0], a[1], 0x35B59C, 0x35B3B0, 0x31A278, 0x31A278),
            },
            Block {
                name: "damage_wear line 159",
                operands: vec![Operand::Buffer {
                    reads: &[(0x35B398, In::F), (0x35B460, In::F)],
                    writes: &[(0x35B464, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_11(a[0]) },
                plain: |a| x87_4(a[0]),
            },
            Block {
                name: "damage_wear line 166",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B464, In::F), (0x35B578, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A28C, In::F)],
                        writes: &[(0x31A28C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_12(a[0], a[1]) },
                plain: |a| x87_1(a[0], a[1], 0x35B578, 0x35B464, 0x31A28C, 0x31A28C),
            },
            Block {
                name: "damage_wear line 177",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B57C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A28C, In::F)],
                        writes: &[(0x31A28C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_13(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1], 0x35B57C, 0x31A28C, 0x31A28C),
            },
            Block {
                name: "damage_wear line 188",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B39C, In::F), (0x35B580, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A28C, In::F)],
                        writes: &[(0x31A28C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_14(a[0], a[1]) },
                plain: |a| x87_3(a[0], a[1], 0x35B580, 0x35B39C, 0x31A28C, 0x31A28C),
            },
            Block {
                name: "damage_wear line 201",
                operands: vec![Operand::Buffer {
                    reads: &[(0x35B44C, In::F)],
                    writes: &[(0x35B44C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_15(a[0]) },
                plain: |a| x87_5(a[0], 0x35B44C, 0x35B44C),
            },
            Block {
                name: "damage_wear line 208",
                operands: vec![Operand::Buffer {
                    reads: &[(0x35B448, In::F)],
                    writes: &[(0x35B448, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_16(a[0]) },
                plain: |a| x87_5(a[0], 0x35B448, 0x35B448),
            },
            Block {
                name: "damage_wear line 215",
                operands: vec![Operand::Buffer {
                    reads: &[(0x35B468, In::F)],
                    writes: &[(0x35B468, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_17(a[0]) },
                plain: |a| x87_5(a[0], 0x35B468, 0x35B468),
            },
            Block {
                name: "damage_wear line 223",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x35B39C, In::F),
                        (0x35B3E0, In::F),
                        (0x35B3F4, In::F),
                        (0x35B448, In::F),
                        (0x35B44C, In::F),
                        (0x35B468, In::F),
                        (0x35B6F8, In::F),
                    ],
                    writes: &[(0x35B450, Out::F), (0x35B454, Out::F), (0x35B458, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_18(a[0]) },
                plain: |a| x87_6(a[0]),
            },
            Block {
                name: "damage_wear line 252",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B3F0, In::F), (0x35B8BC, In::F)],
                        writes: &[(0x35B484, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::I)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_19(a[0], a[1], a[2]) },
                plain: |a| x87_7(a[0], a[1], a[2]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
