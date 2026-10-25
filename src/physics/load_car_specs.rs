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
//! The x87 runs are plain Rust over `X` (`math/x87_float.rs`), the original's instructions
//! (from IDA) being `asm_reference` in the tests; the compares and the integer
//! steps in Rust.
//!
//! In:  nothing
//! Out: eax the record's gear count, ebx the drive kind, esi the record; no flags read
//!
//! Callers: `sub_43B635` (0x43B63A), `sub_4217E4`. Refuses (before anything) a drive kind
//! past 3 (`flt_613FB8`'s size not worked out).

use crate::errors::Refused;
use crate::framework::{game_write, rva};
use crate::math::x87;
use crate::physics::car_store::CAR;
use crate::physics::gear_store::GEAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::car_specs_store::{CAR_SPECS, CarSpec};
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use crate::scene::x87_mem::{fild, fistp, fld, fstp};
use crate::session::{Cell32, Dword};
use rc2k_core::physics::car_specs::{OWN_FIGURES_FROM, check_drive, gear_count, record_of};
use std::cmp::Ordering;

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The record of the work car's kind (`record_of` holds it to the last).
fn record() -> &'static CarSpec {
    &CAR_SPECS.cars[record_of(CAR.car_kind.get()) as usize]
}

/// The whole-number steps both versions share, after the floats: the torque curve, the
/// gear count (held), the words copied.
fn words(spec: &CarSpec) {
    for (to, from) in PHYSICS.spec_torque.iter().zip(&spec.torque_curve) {
        to.set(from.get());
    }
    PHYSICS.spec_a0.set(spec.spec_a0.get());
    PHYSICS.gear_count.set(gear_count(spec.gear_count.get()));
    p(0x614C14, spec.copy_20.get());
    let places: [(&dyn Cell32, &Dword); 5] = [
        (&PHYSICS.spec_surface[0], &spec.surface[0]),
        (&PHYSICS.spec_surface[1], &spec.surface[1]),
        (&PHYSICS.grip_scale, &spec.grip_scale),
        (&PHYSICS.side_grip_scale, &spec.side_grip_scale),
        (&PHYSICS.spec_surface[2], &spec.surface[2]),
    ];
    for (to, from) in places {
        to.set_bits(from.get());
    }
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::test_drive`).
pub unsafe fn load_car_specs() -> Result<(), Refused> {
    let kind = CAR.car_kind.get();
    let spec = record();
    // The x87 runs read the record at their operands' offsets.
    let s = spec as *const CarSpec as usize;
    let drive = check_drive(spec.drive_kind.get())?;
    CAR.drive_kind.set(drive);
    // 0x4214A7 .. 0x421543
    x87_1(s, crate::race::store::asm_base() as usize);
    if below(CAR.engine_speed_limit.bits(), NUMBERS.thousand.bits()) {
        CAR.engine_speed_limit
            .set_bits(x87::loaded(NUMBERS.thousand.bits()));
    }
    for figure in &GEAR.figures {
        if below(figure.bits(), NUMBERS.eight.bits()) {
            figure.set_bits(x87::loaded(NUMBERS.eight.bits()));
        }
    }
    words(spec);
    // 0x4216C5 .. 0x421704
    x87_2(s, crate::race::store::asm_base() as usize);
    if kind < OWN_FIGURES_FROM {
        // 0x42170D .. 0x421755
        x87_3(crate::race::store::asm_base() as usize);
    }
    PHYSICS
        .spec_grip
        .set_bits(crate::physics::tables_store::PHYSICS.spec_default_ba0.get());
    PHYSICS
        .spec_grip_b
        .set_bits(crate::physics::tables_store::PHYSICS.spec_default_ba4.get());
    PHYSICS
        .spec_wheel
        .set_bits(crate::physics::tables_store::PHYSICS.spec_default_ba8.get());
    // 0x421794 .. 0x4217B2
    x87_4(s, crate::race::store::asm_base() as usize);
    DRAW.scratch.set(spec.scratch_value.get());
    PHYSICS.downforce_spec.set(spec.downforce.get());
    // 0x4217CA .. 0x4217E3
    x87_5(crate::race::store::asm_base() as usize, drive as usize);
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x1CB96C] = int [s + 0x64]`
/// `[rc + 0x1CB970] = int [s + 0x68]`
/// `[rc + 0x1CB974] = int [s + 0x6C]`
/// `[rc + 0x1CB978] = int [s + 0x70]`
/// `[rc + 0x31A1B4] = int [s + 0x74]`
/// `[rc + 0x31A1B8] = [rc + 0x213F10]`
/// `[rc + 0x1CB958] = [s + 0xA4]`
/// `[rc + 0x1CB964] = [s + 0xA4] * [rc + 0x213E88]`
/// `[rc + 0x1CB95C] = [s + 0xA8] + [rc + 0x1C5DDC]`
/// `[rc + 0x1CB960] = [s + 0xAC] + [rc + 0x1C5DDC]`
/// `[rc + 0x1CB968] = (int [s + 0x78] * [rc + 0x213EB8]) / [rc + 0x31A1B4]`
/// `[rc + 0x214104] = [rc + 0x31A1B4] * [rc + 0x213FE4]`
fn x87_1(s: usize, rc: usize) {
    unsafe {
        let s = s as *const u8;
        let rc = rc as *mut u8;
        let t0 = fild(s, 0x64);
        fstp(rc, 0x1CB96C, t0);
        let t1 = fild(s, 0x68);
        fstp(rc, 0x1CB970, t1);
        let t2 = fild(s, 0x6C);
        fstp(rc, 0x1CB974, t2);
        let t3 = fild(s, 0x70);
        fstp(rc, 0x1CB978, t3);
        let t4 = fild(s, 0x74);
        fstp(rc, 0x31A1B4, t4);
        let t5 = fld(rc, 0x213F10);
        fstp(rc, 0x31A1B8, t5);
        let t6 = fld(s, 0xA4);
        fstp(rc, 0x1CB958, t6);
        let t7 = fld(rc, 0x213E88);
        let t8 = t6 * t7;
        fstp(rc, 0x1CB964, t8);
        let t9 = fld(s, 0xA8);
        let t10 = fld(rc, 0x1C5DDC);
        let t11 = t9 + t10;
        fstp(rc, 0x1CB95C, t11);
        let t12 = fld(s, 0xAC);
        let t13 = fld(rc, 0x1C5DDC);
        let t14 = t12 + t13;
        fstp(rc, 0x1CB960, t14);
        let t15 = fild(s, 0x78);
        let t16 = fld(rc, 0x213EB8);
        let t17 = t15 * t16;
        let t18 = fld(rc, 0x31A1B4);
        let t19 = t17 / t18;
        fstp(rc, 0x1CB968, t19);
        let t20 = fld(rc, 0x31A1B4);
        let t21 = fld(rc, 0x213FE4);
        let t22 = t20 * t21;
        fstp(rc, 0x214104, t22);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x214BC0] = (int [s + 0x40] * [rc + 0x1C5D98]) / [rc + 0x1C5D18]`
/// `int [rc + 0x214BC4] = (int [s + 0x44] * [rc + 0x1C5D98]) / [rc + 0x1C5D18]`
/// `int [rc + 0x214BC8] = (int [s + 0x48] * [rc + 0x1C5D98]) / [rc + 0x1C5D18]`
fn x87_2(s: usize, rc: usize) {
    unsafe {
        let s = s as *const u8;
        let rc = rc as *mut u8;
        let t0 = fild(s, 0x40);
        let t1 = fld(rc, 0x1C5D98);
        let t2 = t0 * t1;
        let t3 = fld(rc, 0x1C5D18);
        let t4 = t2 / t3;
        fistp(rc, 0x214BC0, t4);
        let t5 = fild(s, 0x44);
        let t6 = fld(rc, 0x1C5D98);
        let t7 = t5 * t6;
        let t8 = fld(rc, 0x1C5D18);
        let t9 = t7 / t8;
        fistp(rc, 0x214BC4, t9);
        let t10 = fild(s, 0x48);
        let t11 = fld(rc, 0x1C5D98);
        let t12 = t10 * t11;
        let t13 = fld(rc, 0x1C5D18);
        let t14 = t12 / t13;
        fistp(rc, 0x214BC8, t14);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x214BC0] = (int [rc + 0x213F04] * [rc + 0x1C5D98]) / [rc + 0x1C5D18]`
/// `int [rc + 0x214BC4] = (int [rc + 0x213F08] * [rc + 0x1C5D98]) / [rc + 0x1C5D18]`
/// `int [rc + 0x214BC8] = (int [rc + 0x213F0C] * [rc + 0x1C5D98]) / [rc + 0x1C5D18]`
fn x87_3(rc: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let t0 = fild(rc, 0x213F04);
        let t1 = fld(rc, 0x1C5D98);
        let t2 = t0 * t1;
        let t3 = fld(rc, 0x1C5D18);
        let t4 = t2 / t3;
        fistp(rc, 0x214BC0, t4);
        let t5 = fild(rc, 0x213F08);
        let t6 = fld(rc, 0x1C5D98);
        let t7 = t5 * t6;
        let t8 = fld(rc, 0x1C5D18);
        let t9 = t7 / t8;
        fistp(rc, 0x214BC4, t9);
        let t10 = fild(rc, 0x213F0C);
        let t11 = fld(rc, 0x1C5D98);
        let t12 = t10 * t11;
        let t13 = fld(rc, 0x1C5D18);
        let t14 = t12 / t13;
        fistp(rc, 0x214BC8, t14);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214BAC] = [s + 0x38] * [rc + 0x213FC8]`
/// `[rc + 0x214BB0] = [s + 0x3C] * [rc + 0x213FC8]`
fn x87_4(s: usize, rc: usize) {
    unsafe {
        let s = s as *const u8;
        let rc = rc as *mut u8;
        let t0 = fld(s, 0x38);
        let t1 = fld(rc, 0x213FC8);
        let t2 = t0 * t1;
        fstp(rc, 0x214BAC, t2);
        let t3 = fld(s, 0x3C);
        let t4 = fld(rc, 0x213FC8);
        let t5 = t3 * t4;
        fstp(rc, 0x214BB0, t5);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2140F8] = [rc + k * 4 + 0x213FB8]`
/// `[rc + 0x2140FC] = [rc + k * 4 + 0x213FB8] + [rc + 0x1C5D14]`
fn x87_5(rc: usize, k: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let t0 = fld(rc, k * 4 + 0x213FB8);
        fstp(rc, 0x2140F8, t0);
        let t1 = fld(rc, 0x1C5D14);
        let t2 = t0 + t1;
        fstp(rc, 0x2140FC, t2);
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(s, rc) "fild dword ptr [{s} + 0x64]", "fstp dword ptr [{rc} + 0x1CB96C]", "fild dword ptr [{s} + 0x68]", "fstp dword ptr [{rc} + 0x1CB970]", "fild dword ptr [{s} + 0x6C]", "fstp dword ptr [{rc} + 0x1CB974]", "fild dword ptr [{s} + 0x70]", "fstp dword ptr [{rc} + 0x1CB978]", "fild dword ptr [{s} + 0x74]", "fstp dword ptr [{rc} + 0x31A1B4]", "fld dword ptr [{rc} + 0x213F10]", "fstp dword ptr [{rc} + 0x31A1B8]", "fld dword ptr [{s} + 0xA4]", "fst dword ptr [{rc} + 0x1CB958]", "fmul dword ptr [{rc} + 0x213E88]", "fstp dword ptr [{rc} + 0x1CB964]", "fld dword ptr [{s} + 0xA8]", "fadd dword ptr [{rc} + 0x1C5DDC]", "fstp dword ptr [{rc} + 0x1CB95C]", "fld dword ptr [{s} + 0xAC]", "fadd dword ptr [{rc} + 0x1C5DDC]", "fstp dword ptr [{rc} + 0x1CB960]", "fild dword ptr [{s} + 0x78]", "fmul dword ptr [{rc} + 0x213EB8]", "fdiv dword ptr [{rc} + 0x31A1B4]", "fstp dword ptr [{rc} + 0x1CB968]", "fld dword ptr [{rc} + 0x31A1B4]", "fmul dword ptr [{rc} + 0x213FE4]", "fstp dword ptr [{rc} + 0x214104]");
    reference!(ref_1(s, rc) "fild dword ptr [{s} + 0x40]", "fmul dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5D18]", "fistp dword ptr [{rc} + 0x214BC0]", "fild dword ptr [{s} + 0x44]", "fmul dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5D18]", "fistp dword ptr [{rc} + 0x214BC4]", "fild dword ptr [{s} + 0x48]", "fmul dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5D18]", "fistp dword ptr [{rc} + 0x214BC8]");
    reference!(ref_2(rc) "fild dword ptr [{rc} + 0x213F04]", "fmul dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5D18]", "fistp dword ptr [{rc} + 0x214BC0]", "fild dword ptr [{rc} + 0x213F08]", "fmul dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5D18]", "fistp dword ptr [{rc} + 0x214BC4]", "fild dword ptr [{rc} + 0x213F0C]", "fmul dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5D18]", "fistp dword ptr [{rc} + 0x214BC8]");
    reference!(ref_3(s, rc) "fld dword ptr [{s} + 0x38]", "fmul dword ptr [{rc} + 0x213FC8]", "fstp dword ptr [{rc} + 0x214BAC]", "fld dword ptr [{s} + 0x3C]", "fmul dword ptr [{rc} + 0x213FC8]", "fstp dword ptr [{rc} + 0x214BB0]");
    reference!(ref_4(rc, k) "fld dword ptr [{rc} + {k}*4 + 0x213FB8]", "fst dword ptr [{rc} + 0x2140F8]", "fadd dword ptr [{rc} + 0x1C5D14]", "fstp dword ptr [{rc} + 0x2140FC]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "load_car_specs line 100",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x64, In::I),
                            (0x68, In::I),
                            (0x6C, In::I),
                            (0x70, In::I),
                            (0x74, In::I),
                            (0x78, In::I),
                            (0xA4, In::F),
                            (0xA8, In::F),
                            (0xAC, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[
                            (0x1C5DDC, In::F),
                            (0x213E88, In::F),
                            (0x213EB8, In::F),
                            (0x213F10, In::F),
                            (0x213FE4, In::F),
                            (0x31A1B4, In::F),
                        ],
                        writes: &[
                            (0x1CB958, Out::F),
                            (0x1CB95C, Out::F),
                            (0x1CB960, Out::F),
                            (0x1CB964, Out::F),
                            (0x1CB968, Out::F),
                            (0x1CB96C, Out::F),
                            (0x1CB970, Out::F),
                            (0x1CB974, Out::F),
                            (0x1CB978, Out::F),
                            (0x214104, Out::F),
                            (0x31A1B4, Out::F),
                            (0x31A1B8, Out::F),
                        ],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| x87_1(a[0], a[1]),
            },
            Block {
                name: "load_car_specs line 145",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x40, In::I), (0x44, In::I), (0x48, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x1C5D18, In::F), (0x1C5D98, In::F)],
                        writes: &[(0x214BC0, Out::I), (0x214BC4, Out::I), (0x214BC8, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1]),
            },
            Block {
                name: "load_car_specs line 162",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x1C5D18, In::F),
                        (0x1C5D98, In::F),
                        (0x213F04, In::I),
                        (0x213F08, In::I),
                        (0x213F0C, In::I),
                    ],
                    writes: &[(0x214BC0, Out::I), (0x214BC4, Out::I), (0x214BC8, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0]) },
                plain: |a| x87_3(a[0]),
            },
            Block {
                name: "load_car_specs line 182",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x38, In::F), (0x3C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x213FC8, In::F)],
                        writes: &[(0x214BAC, Out::F), (0x214BB0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| x87_4(a[0], a[1]),
            },
            Block {
                name: "load_car_specs line 194 idx k=0",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x1C5D14, In::F), (0x213FB8, In::F)],
                        writes: &[(0x2140F8, Out::F), (0x2140FC, Out::F)],
                    },
                    Operand::Value(0),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1]) },
                plain: |a| x87_5(a[0], a[1]),
            },
            Block {
                name: "load_car_specs line 194 idx k=1",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x1C5D14, In::F), (0x213FBC, In::F)],
                        writes: &[(0x2140F8, Out::F), (0x2140FC, Out::F)],
                    },
                    Operand::Value(1),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1]) },
                plain: |a| x87_5(a[0], a[1]),
            },
            Block {
                name: "load_car_specs line 194 idx k=4",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x1C5D14, In::F), (0x213FC8, In::F)],
                        writes: &[(0x2140F8, Out::F), (0x2140FC, Out::F)],
                    },
                    Operand::Value(4),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1]) },
                plain: |a| x87_5(a[0], a[1]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
