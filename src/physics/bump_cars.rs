//! `physics::bump_cars` (0x45F7B1): the work car and another (`dword_6D3BB8`, its number
//! `dword_6D3CD4`) bumped. The corner met (`flt_6D3BC8` ..) into `dword_6D3090` ..; a
//! note of the meeting at `dword_6D3CA0` .. (the current car, the other, the corner, the
//! point; two parts 0, as `flt_5C5D90`); contact record 6's force 0, `unk_71A20C` = 1,
//! the other's +0x234 = 1; the speed between them (+0x198 ..) into `flt_6D3CD8` .. and
//! its length into `flt_6D3CD0`; the speeds shared out (each part a x `flt_6D3BB0` + b x
//! `flt_6D3BAC` and the other way); the places' difference / 65536 x `flt_6D3B88` into
//! `flt_6D3B9C` .., times `flt_5C5D40` (`physics::scale_vector`), added to the work car's
//! speed and taken from the other's; both speeds turned into the cars' frames (+0x9CC,
//! `scene::rotate_point`, into +0x1A4 ..); `physics::place_sound_e1_other`.
//!
//! The flow and the calls are Rust; the straight x87 runs are plain Rust over `X`
//! (`math/x87_float.rs`) in the `x87_N` functions below, the original's instructions
//! being `asm_reference` in the tests.
//!
//! In:  edi the other car's block (as `dword_6D3BB8`)
//! Out: esi `dword_6D3BB8`, edi `g_car`; the rest kept; no flags read (the caller
//!      returns)
//!
//! Caller: `physics::corner_hits_car` (its tail). Refuses a null block (before anything)
//! and what `place_sound_e1_other` refuses (what was written put back).

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::{CAR, Car};
use crate::physics::contact_store::CONTACT_WORK;
use crate::race::numbers_store::NUMBERS;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;
use crate::session::{Dword, Float};

use crate::math::x87::loaded;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::physics::scale_vector::scale_vector;
use crate::physics::tables_store::PHYSICS;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

/// The block's speed (+0x198 ..) turned by its matrix (+0x9CC) into +0x1A4 ...
fn turn_speed(block: &Car) {
    let turned = rotate_point(
        &block.car_inverse_matrix,
        [0, 1, 2].map(|k| extended(block.velocity[k].bits())),
    );
    for (cell, e) in block.local_velocity.iter().zip(turned) {
        cell.set_bits(rounded(e));
    }
}

fn steps(edi: u32) -> Result<(), Refused> {
    for k in 0..3 {
        CONTACT_WORK.contact_point[k].set_bits(loaded(CONTACT_WORK.corner_turned[k].bits()));
    }
    let zero = NUMBERS.zero.bits();
    let note = [
        RACE_WORK.current_car.get(),
        CONTACT_WORK.other_car_number.get(),
        CONTACT_WORK.corner_in[0].bits(),
        zero,
        CONTACT_WORK.corner_in[2].bits(),
        CONTACT_WORK.contact_point[0].bits(),
        zero,
        CONTACT_WORK.contact_point[2].bits(),
    ];
    for (k, v) in note.iter().enumerate() {
        CONTACT_WORK.bump_note[k].set(*v);
    }
    for v in &PHYSICS.contacts[6].force {
        v.set_bits(0);
    }
    CAR.contact_flag.set(1);
    let edi = crate::race::trap::car_at(edi)?;
    edi.contact_flag.set(1);
    x87_1(&edi.velocity);
    CONTACT_WORK.bump_speed_length.set_bits(rounded({
        vector_length(&CONTACT_WORK.bump_relative_speed)
    }));
    let esi = Car::at(CONTACT_WORK.other_car_block.get());
    x87_2(&esi.velocity);
    // 0x45F98C: pushed apart by the places' difference.
    let other = Car::at(CONTACT_WORK.other_car_block.get());
    x87_3(&other.position);
    let factor = extended(NUMBERS.three.bits());
    scale_vector(&CONTACT_WORK.bump_push, factor);
    DRAW.scratch.set(rounded(factor));
    x87_4(&other.velocity);
    turn_speed(esi);
    turn_speed(&CAR);
    { crate::physics::place_sound_e1_other::place_sound_e1_other() }?;
    Ok(())
}

/// The Rust version: `edi` the other car's block (an address in the game's memory).
pub fn bump_cars(edi: u32) -> Result<(), Refused> {
    if edi == 0 || CONTACT_WORK.other_car_block.get() == 0 {
        return Err(BadInput("a null car block").into());
    }
    steps(edi)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2D3CD8] = [rc + 0x31A170] - [d + 0x198]`
/// `[rc + 0x2D3CDC] = [rc + 0x31A174] - [d + 0x19C]`
/// `[rc + 0x2D3CE0] = [rc + 0x31A178] - [d + 0x1A0]`
fn x87_1(d: &[Float; 3]) {
    let t0 = ld(CAR.velocity[0].bits());
    let t1 = ld(d[0].bits());
    let t2 = t0 - t1;
    CONTACT_WORK.bump_relative_speed[0].set_bits(st(t2));
    let t3 = ld(CAR.velocity[1].bits());
    let t4 = ld(d[1].bits());
    let t5 = t3 - t4;
    CONTACT_WORK.bump_relative_speed[1].set_bits(st(t5));
    let t6 = ld(CAR.velocity[2].bits());
    let t7 = ld(d[2].bits());
    let t8 = t6 - t7;
    CONTACT_WORK.bump_relative_speed[2].set_bits(st(t8));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A170] = ([rc + 0x31A170] * [rc + 0x2D3BAC]) + ([s + 0x198] * [rc + 0x2D3BB0])`
/// `[s + 0x198] = ([rc + 0x31A170] * [rc + 0x2D3BB0]) + ([s + 0x198] * [rc + 0x2D3BAC])`
/// `[rc + 0x31A174] = ([rc + 0x31A174] * [rc + 0x2D3BAC]) + ([s + 0x19C] * [rc + 0x2D3BB0])`
/// `[s + 0x19C] = ([rc + 0x31A174] * [rc + 0x2D3BB0]) + ([s + 0x19C] * [rc + 0x2D3BAC])`
/// `[rc + 0x31A178] = ([rc + 0x31A178] * [rc + 0x2D3BAC]) + ([s + 0x1A0] * [rc + 0x2D3BB0])`
/// `[s + 0x1A0] = ([rc + 0x31A178] * [rc + 0x2D3BB0]) + ([s + 0x1A0] * [rc + 0x2D3BAC])`
fn x87_2(s: &[Float; 3]) {
    let t0 = ld(CAR.velocity[0].bits());
    let t1 = ld(CONTACT_WORK.bump_share_b.bits());
    let t2 = t0 * t1;
    let t3 = ld(s[0].bits());
    let t4 = ld(CONTACT_WORK.bump_share_a.bits());
    let t5 = t3 * t4;
    let t6 = t2 + t5;
    let t7 = ld(CAR.velocity[0].bits());
    let t8 = ld(CONTACT_WORK.bump_share_a.bits());
    let t9 = t7 * t8;
    let t10 = ld(s[0].bits());
    let t11 = ld(CONTACT_WORK.bump_share_b.bits());
    let t12 = t10 * t11;
    let t13 = t9 + t12;
    CAR.velocity[0].set_bits(st(t13));
    s[0].set_bits(st(t6));
    let t14 = ld(CAR.velocity[1].bits());
    let t15 = ld(CONTACT_WORK.bump_share_b.bits());
    let t16 = t14 * t15;
    let t17 = ld(s[1].bits());
    let t18 = ld(CONTACT_WORK.bump_share_a.bits());
    let t19 = t17 * t18;
    let t20 = t16 + t19;
    let t21 = ld(CAR.velocity[1].bits());
    let t22 = ld(CONTACT_WORK.bump_share_a.bits());
    let t23 = t21 * t22;
    let t24 = ld(s[1].bits());
    let t25 = ld(CONTACT_WORK.bump_share_b.bits());
    let t26 = t24 * t25;
    let t27 = t23 + t26;
    CAR.velocity[1].set_bits(st(t27));
    s[1].set_bits(st(t20));
    let t28 = ld(CAR.velocity[2].bits());
    let t29 = ld(CONTACT_WORK.bump_share_b.bits());
    let t30 = t28 * t29;
    let t31 = ld(s[2].bits());
    let t32 = ld(CONTACT_WORK.bump_share_a.bits());
    let t33 = t31 * t32;
    let t34 = t30 + t33;
    let t35 = ld(CAR.velocity[2].bits());
    let t36 = ld(CONTACT_WORK.bump_share_a.bits());
    let t37 = t35 * t36;
    let t38 = ld(s[2].bits());
    let t39 = ld(CONTACT_WORK.bump_share_b.bits());
    let t40 = t38 * t39;
    let t41 = t37 + t40;
    CAR.velocity[2].set_bits(st(t41));
    s[2].set_bits(st(t34));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2D3B9C] = ((int [rc + 0x31A10C] - int [d + 0x134]) / [rc + 0x1C5D98]) * [rc + 0x2D3B88]`
/// `[rc + 0x2D3BA0] = ((int [rc + 0x31A110] - int [d + 0x138]) / [rc + 0x1C5D98]) * [rc + 0x2D3B88]`
/// `[rc + 0x2D3BA4] = ((int [rc + 0x31A114] - int [d + 0x13C]) / [rc + 0x1C5D98]) * [rc + 0x2D3B88]`
fn x87_3(d: &[Dword; 3]) {
    let t0 = int(CAR.position[0].bits() as i32);
    let t1 = int(d[0].bits() as i32);
    let t2 = t0 - t1;
    let t3 = ld(NUMBERS.k65536.bits());
    let t4 = t2 / t3;
    let t5 = ld(CONTACT_WORK.bump_push_scale.bits());
    let t6 = t4 * t5;
    CONTACT_WORK.bump_push[0].set_bits(st(t6));
    let t7 = int(CAR.position[1].bits() as i32);
    let t8 = int(d[1].bits() as i32);
    let t9 = t7 - t8;
    let t10 = ld(NUMBERS.k65536.bits());
    let t11 = t9 / t10;
    let t12 = ld(CONTACT_WORK.bump_push_scale.bits());
    let t13 = t11 * t12;
    CONTACT_WORK.bump_push[1].set_bits(st(t13));
    let t14 = int(CAR.position[2].bits() as i32);
    let t15 = int(d[2].bits() as i32);
    let t16 = t14 - t15;
    let t17 = ld(NUMBERS.k65536.bits());
    let t18 = t16 / t17;
    let t19 = ld(CONTACT_WORK.bump_push_scale.bits());
    let t20 = t18 * t19;
    CONTACT_WORK.bump_push[2].set_bits(st(t20));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A170] = [rc + 0x31A170] + [rc + 0x2D3B9C]`
/// `[rc + 0x31A174] = [rc + 0x31A174] + [rc + 0x2D3BA0]`
/// `[rc + 0x31A178] = [rc + 0x31A178] + [rc + 0x2D3BA4]`
/// `[d + 0x198] = [d + 0x198] - [rc + 0x2D3B9C]`
/// `[d + 0x19C] = [d + 0x19C] - [rc + 0x2D3BA0]`
/// `[d + 0x1A0] = [d + 0x1A0] - [rc + 0x2D3BA4]`
fn x87_4(d: &[Float; 3]) {
    let t0 = ld(CAR.velocity[0].bits());
    let t1 = ld(CONTACT_WORK.bump_push[0].bits());
    let t2 = t0 + t1;
    CAR.velocity[0].set_bits(st(t2));
    let t3 = ld(CAR.velocity[1].bits());
    let t4 = ld(CONTACT_WORK.bump_push[1].bits());
    let t5 = t3 + t4;
    CAR.velocity[1].set_bits(st(t5));
    let t6 = ld(CAR.velocity[2].bits());
    let t7 = ld(CONTACT_WORK.bump_push[2].bits());
    let t8 = t6 + t7;
    CAR.velocity[2].set_bits(st(t8));
    let t9 = ld(d[0].bits());
    let t10 = ld(CONTACT_WORK.bump_push[0].bits());
    let t11 = t9 - t10;
    d[0].set_bits(st(t11));
    let t12 = ld(d[1].bits());
    let t13 = ld(CONTACT_WORK.bump_push[1].bits());
    let t14 = t12 - t13;
    d[1].set_bits(st(t14));
    let t15 = ld(d[2].bits());
    let t16 = ld(CONTACT_WORK.bump_push[2].bits());
    let t17 = t15 - t16;
    d[2].set_bits(st(t17));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::view;

    // The original's instructions, one function per run.
    reference!(ref_0(rc, d) "fld dword ptr [{rc} + 0x31A170]", "fsub dword ptr [{d} + 0x198]", "fstp dword ptr [{rc} + 0x2D3CD8]", "fld dword ptr [{rc} + 0x31A174]", "fsub dword ptr [{d} + 0x19C]", "fstp dword ptr [{rc} + 0x2D3CDC]", "fld dword ptr [{rc} + 0x31A178]", "fsub dword ptr [{d} + 0x1A0]", "fstp dword ptr [{rc} + 0x2D3CE0]");
    reference!(ref_1(rc, s) "fld dword ptr [{rc} + 0x31A170]", "fmul dword ptr [{rc} + 0x2D3BB0]", "fld dword ptr [{s} + 0x198]", "fmul dword ptr [{rc} + 0x2D3BAC]", ".byte 0xDE, 0xC1", "fld dword ptr [{rc} + 0x31A170]", "fmul dword ptr [{rc} + 0x2D3BAC]", "fld dword ptr [{s} + 0x198]", "fmul dword ptr [{rc} + 0x2D3BB0]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + 0x31A170]", "fstp dword ptr [{s} + 0x198]", "fld dword ptr [{rc} + 0x31A174]", "fmul dword ptr [{rc} + 0x2D3BB0]", "fld dword ptr [{s} + 0x19C]", "fmul dword ptr [{rc} + 0x2D3BAC]", ".byte 0xDE, 0xC1", "fld dword ptr [{rc} + 0x31A174]", "fmul dword ptr [{rc} + 0x2D3BAC]", "fld dword ptr [{s} + 0x19C]", "fmul dword ptr [{rc} + 0x2D3BB0]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + 0x31A174]", "fstp dword ptr [{s} + 0x19C]", "fld dword ptr [{rc} + 0x31A178]", "fmul dword ptr [{rc} + 0x2D3BB0]", "fld dword ptr [{s} + 0x1A0]", "fmul dword ptr [{rc} + 0x2D3BAC]", ".byte 0xDE, 0xC1", "fld dword ptr [{rc} + 0x31A178]", "fmul dword ptr [{rc} + 0x2D3BAC]", "fld dword ptr [{s} + 0x1A0]", "fmul dword ptr [{rc} + 0x2D3BB0]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + 0x31A178]", "fstp dword ptr [{s} + 0x1A0]");
    reference!(ref_2(rc, d) "fild dword ptr [{rc} + 0x31A10C]", "fisub dword ptr [{d} + 0x134]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fmul dword ptr [{rc} + 0x2D3B88]", "fstp dword ptr [{rc} + 0x2D3B9C]", "fild dword ptr [{rc} + 0x31A110]", "fisub dword ptr [{d} + 0x138]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fmul dword ptr [{rc} + 0x2D3B88]", "fstp dword ptr [{rc} + 0x2D3BA0]", "fild dword ptr [{rc} + 0x31A114]", "fisub dword ptr [{d} + 0x13C]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fmul dword ptr [{rc} + 0x2D3B88]", "fstp dword ptr [{rc} + 0x2D3BA4]");
    reference!(ref_3(rc, d) "fld dword ptr [{rc} + 0x31A170]", "fadd dword ptr [{rc} + 0x2D3B9C]", "fstp dword ptr [{rc} + 0x31A170]", "fld dword ptr [{rc} + 0x31A174]", "fadd dword ptr [{rc} + 0x2D3BA0]", "fstp dword ptr [{rc} + 0x31A174]", "fld dword ptr [{rc} + 0x31A178]", "fadd dword ptr [{rc} + 0x2D3BA4]", "fstp dword ptr [{rc} + 0x31A178]", "fld dword ptr [{d} + 0x198]", "fsub dword ptr [{rc} + 0x2D3B9C]", "fstp dword ptr [{d} + 0x198]", "fld dword ptr [{d} + 0x19C]", "fsub dword ptr [{rc} + 0x2D3BA0]", "fstp dword ptr [{d} + 0x19C]", "fld dword ptr [{d} + 0x1A0]", "fsub dword ptr [{rc} + 0x2D3BA4]", "fstp dword ptr [{d} + 0x1A0]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "bump_cars line 66",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A170, In::F), (0x31A174, In::F), (0x31A178, In::F)],
                        writes: &[(0x2D3CD8, Out::F), (0x2D3CDC, Out::F), (0x2D3CE0, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x198, In::F), (0x19C, In::F), (0x1A0, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| x87_1(view(a[1], 0x198)),
            },
            Block {
                name: "bump_cars line 80",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x2D3BAC, In::F),
                            (0x2D3BB0, In::F),
                            (0x31A170, In::F),
                            (0x31A174, In::F),
                            (0x31A178, In::F),
                        ],
                        writes: &[(0x31A170, Out::F), (0x31A174, Out::F), (0x31A178, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x198, In::F), (0x19C, In::F), (0x1A0, In::F)],
                        writes: &[(0x198, Out::F), (0x19C, Out::F), (0x1A0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| x87_2(view(a[1], 0x198)),
            },
            Block {
                name: "bump_cars line 121",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5D98, In::F),
                            (0x2D3B88, In::F),
                            (0x31A10C, In::I),
                            (0x31A110, In::I),
                            (0x31A114, In::I),
                        ],
                        writes: &[(0x2D3B9C, Out::F), (0x2D3BA0, Out::F), (0x2D3BA4, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x134, In::I), (0x138, In::I), (0x13C, In::I)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1]) },
                plain: |a| x87_3(view(a[1], 0x134)),
            },
            Block {
                name: "bump_cars line 142",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x2D3B9C, In::F),
                            (0x2D3BA0, In::F),
                            (0x2D3BA4, In::F),
                            (0x31A170, In::F),
                            (0x31A174, In::F),
                            (0x31A178, In::F),
                        ],
                        writes: &[(0x31A170, Out::F), (0x31A174, Out::F), (0x31A178, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x198, In::F), (0x19C, In::F), (0x1A0, In::F)],
                        writes: &[(0x198, Out::F), (0x19C, Out::F), (0x1A0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| x87_4(view(a[1], 0x198)),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
