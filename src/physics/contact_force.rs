//! `physics::contact_force` (0x45F17E): the force of one contact into the record that
//! `dword_614B40` names. From the contact's point (`dword_6D3090` .., car frame), the
//! car's speed (`dword_6D30A8` ..), its turn speed (`dword_6D30B4` ..) and the surface's
//! normal (`qword_6D3078` ..): the point's speed (turn x point + speed) into
//! `flt_6D30CC` ..; its depth along the normal, held at least 1 - |normal y| + `flt_5C5CFC`,
//! into `flt_6D31DC`; the push the turn would give (point x normal turned by the inverse
//! matrix, times `flt_613FA8` .., turned back, x point) along the normal into `flt_6D31D8`
//! and the share `flt_6D1DF4` / (it + 1) into `flt_6D31E0`; the normal push
//! (`qword_6D3130` ..), the speed along the normal and across it (`qword_6D30E4` ..,
//! `dword_6D30F0` ..; their lengths `flt_6D3108`, `dword_6D310C`), with `dword_614148`
//! a part of the normal x `flt_6D1DF8` x it / `dword_61414C` added; the slide against the
//! motion (`dword_6D3140` ..) held to `flt_6D3100` long; their sum (`qword_6D3150` ..)
//! turned by the inverse matrix and / `flt_61388C` into the record's force, the point
//! turned into its point (+0xC ..). The turns are `physics::turn_by_inverse` /
//! `turn_by_car_matrix`, the products `scene::cross_product`, `physics::dot_product`,
//! `multiply_parts`, `scale_vector`, `vector_length`.
//!
//! The flow and the calls are Rust; each straight x87 run is plain Rust over `X`
//! (`math/x87_float.rs`) in the `x87_N` functions below, the original's instructions
//! being `asm_reference` in the tests.
//!
//! In:  nothing (`dword_614B40` the record)
//! Out: eax `dword_6D3098` (the point's z turned), ebx and esi 0x6D3090, ecx 0x6D31F4,
//!      edx 0x6D31E4, edi the record; no flags read (callers call on)
//!
//! Callers: `sub_45D8FB`, `sub_45DA14`, `sub_45DC3E`, `sub_45E3BF`,
//! `physics::crash_contact`, `physics::ground_contact`.

use crate::math::x87_float::X;
use crate::math::x87_float::compare;
use crate::math::x87_run::{compare_80, neg_rounded};
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::dot_product::dot_product;
use crate::physics::flow::{hold_low, negate, p};
use crate::physics::multiply_parts::multiply_parts;
use crate::physics::scale_vector::scale_vector;
use crate::physics::tables_store::PHYSICS;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::physics::turn_by_inverse::turn_by_inverse;
use crate::physics::vector_length::vector_length;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::view_matrix::cross_product;
use crate::scene::x87_mem::{fild, fld, fnstsw, fstp, fstp80};
use crate::session::Float;

/// `scene::cross_product` with the original's ebx, ecx, edx.
fn cross(ebx: &Float, ecx: &Float, edx: &Float) {
    unsafe { cross_product(ebx.addr(), ecx.addr(), edx.addr()) };
}

/// Three floats copied as bits, in order.
fn copy3(to: &[Float], from: &[Float]) {
    for k in 0..3 {
        to[k].set_bits(from[k].bits());
    }
}

/// Three floats set to the bits `v`.
fn fill3(to: &[Float], v: u32) {
    for cell in &to[..3] {
        cell.set_bits(v);
    }
}

/// The Rust version.
pub unsafe fn contact_force() {
    cross(
        &CONTACT_WORK.contact_turn_speed[0],
        &CONTACT_WORK.contact_point[0],
        &CONTACT_WORK.contact_turn_cross[0],
    );
    x87_1(
        crate::race::store::asm_base() as usize,
        0x2D30C0,
        0x2D30A8,
        0x2D30CC,
    );
    p(
        &CONTACT_WORK.contact_depth,
        neg_rounded(unsafe {
            dot_product(
                CONTACT_WORK.contact_point_speed[0].addr() as u32,
                CONTACT_WORK.contact_normal[0].addr() as u32,
            )
        }),
    );
    x87_2(crate::race::store::asm_base() as usize);
    hold_low(&CONTACT_WORK.contact_depth, &DRAW.scratch);
    // 0x45F222: the push the turn would give.
    cross(
        &CONTACT_WORK.contact_point[0],
        &CONTACT_WORK.contact_normal[0],
        &CONTACT_WORK.contact_scratch_a[0],
    );
    unsafe { turn_by_inverse(CONTACT_WORK.contact_scratch_a[0].addr() as u32) };
    unsafe {
        multiply_parts(
            CONTACT_WORK.contact_scratch_a[0].addr() as u32,
            PHYSICS.inertia[0].addr() as u32,
            CONTACT_WORK.contact_scratch_b[0].addr() as u32,
        )
    };
    unsafe { turn_by_car_matrix(CONTACT_WORK.contact_scratch_b[0].addr() as u32) };
    cross(
        &CONTACT_WORK.contact_point[0],
        &CONTACT_WORK.contact_scratch_b[0],
        &CONTACT_WORK.contact_scratch_a[0],
    );
    p(
        &CONTACT_WORK.contact_turn_push,
        neg_rounded(unsafe {
            dot_product(
                CONTACT_WORK.contact_scratch_a[0].addr() as u32,
                CONTACT_WORK.contact_normal[0].addr() as u32,
            )
        }),
    );
    x87_3(crate::race::store::asm_base() as usize);
    // 0x45F2C2: the normal push, the speed along the normal and across it.
    copy3(
        &CONTACT_WORK.contact_normal_push,
        &CONTACT_WORK.contact_normal,
    );
    let factor = extended(CONTACT_WORK.contact_factor.bits());
    unsafe { scale_vector(CONTACT_WORK.contact_normal_push[0].addr() as u32, factor) };
    DRAW.scratch.set(rounded(factor));
    let along = unsafe {
        dot_product(
            CONTACT_WORK.contact_point_speed[0].addr() as u32,
            CONTACT_WORK.contact_normal[0].addr() as u32,
        )
    };
    copy3(
        &CONTACT_WORK.contact_normal_speed,
        &CONTACT_WORK.contact_normal,
    );
    unsafe { scale_vector(CONTACT_WORK.contact_normal_speed[0].addr() as u32, along) };
    DRAW.scratch.set(rounded(along));
    copy3(
        &CONTACT_WORK.contact_side_speed,
        &CONTACT_WORK.contact_point_speed,
    );
    x87_4(crate::race::store::asm_base() as usize);
    CONTACT_WORK.contact_speed_length.set_bits(rounded(unsafe {
        vector_length(CONTACT_WORK.contact_normal_speed[0].addr() as u32)
    }));
    CONTACT_WORK.contact_value.set_bits(rounded(unsafe {
        vector_length(CONTACT_WORK.contact_side_speed[0].addr() as u32)
    }));
    fill3(&CONTACT_WORK.contact_extra, 0);
    if PHYSICS.contact_step.get() != 0 {
        fill3(&CONTACT_WORK.contact_normal_push, 0);
        copy3(&CONTACT_WORK.contact_extra, &CONTACT_WORK.contact_normal);
        let mut part = [0u8; 10];
        x87_5(
            crate::race::store::asm_base() as usize,
            part.as_mut_ptr() as usize,
        );
        unsafe { scale_vector(CONTACT_WORK.contact_extra[0].addr() as u32, part) };
        DRAW.scratch.set(rounded(part));
    }
    x87_1(
        crate::race::store::asm_base() as usize,
        0x2D3130,
        0x2D3160,
        0x2D3130,
    );
    copy3(
        &CONTACT_WORK.contact_total,
        &CONTACT_WORK.contact_normal_push,
    );
    CONTACT_WORK
        .contact_slide_limit_held
        .set_bits(crate::math::x87::loaded(
            CONTACT_WORK.contact_slide_limit.bits(),
        ));
    // 0x45F48D: the slide against the motion, held to `flt_6D30FC` long.
    copy3(
        &CONTACT_WORK.contact_slide,
        &CONTACT_WORK.contact_side_speed,
    );
    negate(&CONTACT_WORK.contact_slide[0]);
    negate(&CONTACT_WORK.contact_slide[1]);
    negate(&CONTACT_WORK.contact_slide[2]);
    if compare_80(
        unsafe { vector_length(CONTACT_WORK.contact_slide[0].addr() as u32) },
        NUMBERS.zero.bits(),
    ) & 0x4000
        != 0
    {
        fill3(&CONTACT_WORK.contact_slide, 0);
    }
    let mut status = 0u16;
    x87_6(
        crate::race::store::asm_base() as usize,
        (&raw mut status) as usize,
    );
    if status & 0x4100 == 0 {
        x87_7(crate::race::store::asm_base() as usize);
    }
    // 0x45F57E: the sum into the record.
    x87_1(
        crate::race::store::asm_base() as usize,
        0x2D3140,
        0x2D3130,
        0x2D3150,
    );
    let edi = PHYSICS.contact_record.get();
    unsafe { turn_by_inverse(CONTACT_WORK.contact_total[0].addr() as u32) };
    x87_8(crate::race::store::asm_base() as usize, edi as usize);
    unsafe { turn_by_inverse(CONTACT_WORK.contact_point[0].addr() as u32) };
    for k in 0..3 {
        unsafe {
            ((edi + 0xC + 4 * k) as *mut u32)
                .write_unaligned(CONTACT_WORK.contact_point[k as usize].bits())
        };
    }
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the race core
/// at `rc`: three floats added, `[rc + out + 4 k] = [rc + a + 4 k] + [rc + b + 4 k]` (each stored
/// as made, so `out` may be one of the two).
fn x87_1(rc: usize, a: usize, b: usize, out: usize) {
    unsafe {
        let rc = rc as *mut u8;
        for k in 0..3 {
            fstp(rc, out + 4 * k, fld(rc, a + 4 * k) + fld(rc, b + 4 * k));
        }
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x22400C] = (1 - |[rc + 0x2D307C]|) + [rc + 0x1C5CFC]`
fn x87_2(rc: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let t0 = X(1.0);
        let t1 = fld(rc, 0x2D307C);
        let t2 = t1.abs();
        let t3 = t0 - t2;
        let t4 = fld(rc, 0x1C5CFC);
        let t5 = t3 + t4;
        fstp(rc, 0x22400C, t5);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2D31E0] = [rc + 0x2D1DF4] / ([rc + 0x2D31D8] + [rc + 0x1C5D38])`
/// `[rc + 0x2D3204] = [rc + 0x2D31DC] * [rc + 0x2D31E0]`
fn x87_3(rc: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let t0 = fld(rc, 0x2D1DF4);
        let t1 = fld(rc, 0x2D31D8);
        let t2 = fld(rc, 0x1C5D38);
        let t3 = t1 + t2;
        let t4 = t0 / t3;
        fstp(rc, 0x2D31E0, t4);
        let t5 = fld(rc, 0x2D31DC);
        let t6 = fld(rc, 0x2D31E0);
        let t7 = t5 * t6;
        fstp(rc, 0x2D3204, t7);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2D30F0] = [rc + 0x2D30F0] - [rc + 0x2D30E4]`
/// `[rc + 0x2D30F4] = [rc + 0x2D30F4] - [rc + 0x2D30E8]`
/// `[rc + 0x2D30F8] = [rc + 0x2D30F8] - [rc + 0x2D30EC]`
fn x87_4(rc: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let t0 = fld(rc, 0x2D30F0);
        let t1 = fld(rc, 0x2D30E4);
        let t2 = t0 - t1;
        fstp(rc, 0x2D30F0, t2);
        let t3 = fld(rc, 0x2D30F4);
        let t4 = fld(rc, 0x2D30E8);
        let t5 = t3 - t4;
        fstp(rc, 0x2D30F4, t5);
        let t6 = fld(rc, 0x2D30F8);
        let t7 = fld(rc, 0x2D30EC);
        let t8 = t6 - t7;
        fstp(rc, 0x2D30F8, t8);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `ext [o + 0] = ([rc + 0x2D1DF8] * int [rc + 0x214148]) / int [rc + 0x21414C]`
fn x87_5(rc: usize, o: usize) {
    unsafe {
        let rc = rc as *const u8;
        let o = o as *mut u8;
        let t0 = fld(rc, 0x2D1DF8);
        let t1 = fild(rc, 0x214148);
        let t2 = t0 * t1;
        let t3 = fild(rc, 0x21414C);
        let t4 = t2 / t3;
        fstp80(o, 0, t4);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x22400C] = sqrt((([rc + 0x2D3140] * [rc + 0x2D3140]) + (([rc + 0x2D3144] * [rc + 0x2D3144]) + ([rc + 0x2D3148] * [rc + 0x2D3148]))))`
/// `status [s + 0] = status`
fn x87_6(rc: usize, s: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let s = s as *mut u8;
        let t0 = fld(rc, 0x2D3140);
        let t1 = t0 * t0;
        let t2 = fld(rc, 0x2D3144);
        let t3 = t2 * t2;
        let t4 = fld(rc, 0x2D3148);
        let t5 = t4 * t4;
        let t6 = t3 + t5;
        let t7 = t1 + t6;
        let t8 = t7.sqrt();
        fstp(rc, 0x22400C, t8);
        let t9 = fld(rc, 0x2D30FC);
        let t10 = compare(t8, t9);
        fnstsw(s, 0, t10);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2D3140] = [rc + 0x2D3140] * ([rc + 0x2D30FC] / [rc + 0x22400C])`
/// `[rc + 0x2D3144] = [rc + 0x2D3144] * ([rc + 0x2D30FC] / [rc + 0x22400C])`
/// `[rc + 0x2D3148] = [rc + 0x2D3148] * ([rc + 0x2D30FC] / [rc + 0x22400C])`
/// `[rc + 0x22400C] = [rc + 0x2D30FC] / [rc + 0x22400C]`
fn x87_7(rc: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let t0 = fld(rc, 0x2D30FC);
        let t1 = fld(rc, 0x22400C);
        let t2 = t0 / t1;
        let t3 = fld(rc, 0x2D3140);
        let t4 = t3 * t2;
        fstp(rc, 0x2D3140, t4);
        let t5 = fld(rc, 0x2D3144);
        let t6 = t5 * t2;
        fstp(rc, 0x2D3144, t6);
        let t7 = fld(rc, 0x2D3148);
        let t8 = t7 * t2;
        fstp(rc, 0x2D3148, t8);
        fstp(rc, 0x22400C, t2);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[d + 0] = [rc + 0x2D3150] / [rc + 0x21388C]`
/// `[d + 4] = [rc + 0x2D3154] / [rc + 0x21388C]`
/// `[d + 8] = [rc + 0x2D3158] / [rc + 0x21388C]`
fn x87_8(rc: usize, d: usize) {
    unsafe {
        let rc = rc as *const u8;
        let d = d as *mut u8;
        let t0 = fld(rc, 0x2D3150);
        let t1 = fld(rc, 0x21388C);
        let t2 = t0 / t1;
        fstp(d, 0, t2);
        let t3 = fld(rc, 0x2D3154);
        let t4 = fld(rc, 0x21388C);
        let t5 = t3 / t4;
        fstp(d, 4, t5);
        let t6 = fld(rc, 0x2D3158);
        let t7 = fld(rc, 0x21388C);
        let t8 = t6 / t7;
        fstp(d, 8, t8);
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) "fld dword ptr [{rc} + 0x2D30C0]", "fadd dword ptr [{rc} + 0x2D30A8]", "fstp dword ptr [{rc} + 0x2D30CC]", "fld dword ptr [{rc} + 0x2D30C4]", "fadd dword ptr [{rc} + 0x2D30AC]", "fstp dword ptr [{rc} + 0x2D30D0]", "fld dword ptr [{rc} + 0x2D30C8]", "fadd dword ptr [{rc} + 0x2D30B0]", "fstp dword ptr [{rc} + 0x2D30D4]");
    reference!(ref_1(rc) ".byte 0xD9, 0xE8", "fld dword ptr [{rc} + 0x2D307C]", ".byte 0xD9, 0xE1", ".byte 0xDE, 0xE9", "fadd dword ptr [{rc} + 0x1C5CFC]", "fstp dword ptr [{rc} + 0x22400C]");
    reference!(ref_2(rc) "fld dword ptr [{rc} + 0x2D1DF4]", "fld dword ptr [{rc} + 0x2D31D8]", "fadd dword ptr [{rc} + 0x1C5D38]", ".byte 0xDE, 0xF9", "fstp dword ptr [{rc} + 0x2D31E0]", "fld dword ptr [{rc} + 0x2D31DC]", "fmul dword ptr [{rc} + 0x2D31E0]", "fstp dword ptr [{rc} + 0x2D3204]");
    reference!(ref_3(rc) "fld dword ptr [{rc} + 0x2D30F0]", "fsub dword ptr [{rc} + 0x2D30E4]", "fstp dword ptr [{rc} + 0x2D30F0]", "fld dword ptr [{rc} + 0x2D30F4]", "fsub dword ptr [{rc} + 0x2D30E8]", "fstp dword ptr [{rc} + 0x2D30F4]", "fld dword ptr [{rc} + 0x2D30F8]", "fsub dword ptr [{rc} + 0x2D30EC]", "fstp dword ptr [{rc} + 0x2D30F8]");
    reference!(ref_4(rc, o) "fld dword ptr [{rc} + 0x2D1DF8]", "fild dword ptr [{rc} + 0x214148]", ".byte 0xDE, 0xC9", "fidiv dword ptr [{rc} + 0x21414C]", "fstp tbyte ptr [{o}]");
    reference!(ref_5(rc) "fld dword ptr [{rc} + 0x2D3130]", "fadd dword ptr [{rc} + 0x2D3160]", "fstp dword ptr [{rc} + 0x2D3130]", "fld dword ptr [{rc} + 0x2D3134]", "fadd dword ptr [{rc} + 0x2D3164]", "fstp dword ptr [{rc} + 0x2D3134]", "fld dword ptr [{rc} + 0x2D3138]", "fadd dword ptr [{rc} + 0x2D3168]", "fstp dword ptr [{rc} + 0x2D3138]");
    reference!(ref_6(rc, s) "fld dword ptr [{rc} + 0x2D3140]", ".byte 0xD8, 0xC8", "fld dword ptr [{rc} + 0x2D3144]", ".byte 0xD8, 0xC8", "fld dword ptr [{rc} + 0x2D3148]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", "fst dword ptr [{rc} + 0x22400C]", "fcomp dword ptr [{rc} + 0x2D30FC]", "fnstsw word ptr [{s}]");
    reference!(ref_7(rc) "fld dword ptr [{rc} + 0x2D30FC]", "fdiv dword ptr [{rc} + 0x22400C]", "fld dword ptr [{rc} + 0x2D3140]", ".byte 0xD8, 0xC9", "fstp dword ptr [{rc} + 0x2D3140]", "fld dword ptr [{rc} + 0x2D3144]", ".byte 0xD8, 0xC9", "fstp dword ptr [{rc} + 0x2D3144]", "fld dword ptr [{rc} + 0x2D3148]", ".byte 0xD8, 0xC9", "fstp dword ptr [{rc} + 0x2D3148]", "fstp dword ptr [{rc} + 0x22400C]");
    reference!(ref_8(rc) "fld dword ptr [{rc} + 0x2D3140]", "fadd dword ptr [{rc} + 0x2D3130]", "fstp dword ptr [{rc} + 0x2D3150]", "fld dword ptr [{rc} + 0x2D3144]", "fadd dword ptr [{rc} + 0x2D3134]", "fstp dword ptr [{rc} + 0x2D3154]", "fld dword ptr [{rc} + 0x2D3148]", "fadd dword ptr [{rc} + 0x2D3138]", "fstp dword ptr [{rc} + 0x2D3158]");
    reference!(ref_9(rc, d) "fld dword ptr [{rc} + 0x2D3150]", "fdiv dword ptr [{rc} + 0x21388C]", "fstp dword ptr [{d}]", "fld dword ptr [{rc} + 0x2D3154]", "fdiv dword ptr [{rc} + 0x21388C]", "fstp dword ptr [{d} + 0x4]", "fld dword ptr [{rc} + 0x2D3158]", "fdiv dword ptr [{rc} + 0x21388C]", "fstp dword ptr [{d} + 0x8]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "contact_force line 50",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x2D30A8, In::F),
                        (0x2D30AC, In::F),
                        (0x2D30B0, In::F),
                        (0x2D30C0, In::F),
                        (0x2D30C4, In::F),
                        (0x2D30C8, In::F),
                    ],
                    writes: &[(0x2D30CC, Out::F), (0x2D30D0, Out::F), (0x2D30D4, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0]) },
                plain: |a| x87_1(a[0], 0x2D30C0, 0x2D30A8, 0x2D30CC),
            },
            Block {
                name: "contact_force line 66",
                operands: vec![Operand::Buffer {
                    reads: &[(0x1C5CFC, In::F), (0x2D307C, In::F)],
                    writes: &[(0x22400C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0]) },
                plain: |a| x87_2(a[0]),
            },
            Block {
                name: "contact_force line 86",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x1C5D38, In::F),
                        (0x2D1DF4, In::F),
                        (0x2D31D8, In::F),
                        (0x2D31DC, In::F),
                        (0x2D31E0, In::F),
                    ],
                    writes: &[(0x2D31E0, Out::F), (0x2D3204, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0]) },
                plain: |a| x87_3(a[0]),
            },
            Block {
                name: "contact_force line 107",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x2D30E4, In::F),
                        (0x2D30E8, In::F),
                        (0x2D30EC, In::F),
                        (0x2D30F0, In::F),
                        (0x2D30F4, In::F),
                        (0x2D30F8, In::F),
                    ],
                    writes: &[(0x2D30F0, Out::F), (0x2D30F4, Out::F), (0x2D30F8, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0]) },
                plain: |a| x87_4(a[0]),
            },
            Block {
                name: "contact_force line 126",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x214148, In::I), (0x21414C, In::I), (0x2D1DF8, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::T)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1]) },
                plain: |a| x87_5(a[0], a[1]),
            },
            Block {
                name: "contact_force line 137",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x2D3130, In::F),
                        (0x2D3134, In::F),
                        (0x2D3138, In::F),
                        (0x2D3160, In::F),
                        (0x2D3164, In::F),
                        (0x2D3168, In::F),
                    ],
                    writes: &[(0x2D3130, Out::F), (0x2D3134, Out::F), (0x2D3138, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0]) },
                plain: |a| x87_1(a[0], 0x2D3130, 0x2D3160, 0x2D3130),
            },
            Block {
                name: "contact_force line 160",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x2D30FC, In::F),
                            (0x2D3140, In::F),
                            (0x2D3144, In::F),
                            (0x2D3148, In::F),
                        ],
                        writes: &[(0x22400C, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::H)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1]) },
                plain: |a| x87_6(a[0], a[1]),
            },
            Block {
                name: "contact_force line 176",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x22400C, In::F),
                        (0x2D30FC, In::F),
                        (0x2D3140, In::F),
                        (0x2D3144, In::F),
                        (0x2D3148, In::F),
                    ],
                    writes: &[
                        (0x22400C, Out::F),
                        (0x2D3140, Out::F),
                        (0x2D3144, Out::F),
                        (0x2D3148, Out::F),
                    ],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_7(a[0]) },
                plain: |a| x87_7(a[0]),
            },
            Block {
                name: "contact_force line 193",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x2D3130, In::F),
                        (0x2D3134, In::F),
                        (0x2D3138, In::F),
                        (0x2D3140, In::F),
                        (0x2D3144, In::F),
                        (0x2D3148, In::F),
                    ],
                    writes: &[(0x2D3150, Out::F), (0x2D3154, Out::F), (0x2D3158, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_8(a[0]) },
                plain: |a| x87_1(a[0], 0x2D3140, 0x2D3130, 0x2D3150),
            },
            Block {
                name: "contact_force line 207",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x21388C, In::F),
                            (0x2D3150, In::F),
                            (0x2D3154, In::F),
                            (0x2D3158, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F), (4, Out::F), (8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_9(a[0], a[1]) },
                plain: |a| x87_8(a[0], a[1]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
