//! `physics::wheel_surfaces` (0x43D59A): the surface under each of the five ground points
//! (`unk_71ADC8[c]`, their faces) and the grip figures from it.
//!
//! For each point c: the face and its texture's record (`g_textureInfos`, by the face's
//! +1); the cell (the point's shares `dword_6D2B14[c]` / `dword_6D2A14[c]` x 65536,
//! `fistp`, bit 15, flipped with `dword_6D2914[c]` 1) turned by the face's +0 and +0x20
//! (`physics::flip_cell_index`) into `dword_614710[c]` / `dword_614724[c]`; the two
//! textures' surface kinds and two more bytes by those cells into +0x920 / +0x95C /
//! +0x984 [c] and +0x934 / +0x970 / +0x998 [c]; the blend +0x9AC [c] 0, or (unless the
//! second texture's kind (`dword_63D9D0`) is 4 or 5, its surface 0, or the face's +0x21
//! 0) the face's three blend bytes by the shares / 256; with bit 0x1000 of the face's +2
//! both surfaces 5 (wet, by the look) and the bytes 0x3C, 0. Then each figure blended
//! between the two surface records (`off_613978`, 0x30 each): grip, two more, the record's
//! scaled tables by `g_car` +0x2C and +0x280, into `flt_6142C0`, +0x948, `flt_614284`,
//! `flt_614298`, `flt_6142AC`, `flt_6146C0` .. [c].
//!
//! After the five: `flt_614268` = (the first four `flt_614284`) x `flt_613FEC`; the count
//! of points on hard surfaces (1, 2, 3, 0xC) into +0x258 (`dword_614150` ..); +0x60 the
//! least of the four grips x `flt_614C04`; `flt_614230` by +0x280 and +0x34; four
//! figures copied on.
//!
//! The x87 runs are plain Rust over `X` (`math/x87_float.rs`), the original's instructions
//! (from IDA) being `asm_reference` in the tests; the integer steps in Rust.
//!
//! In:  nothing
//! Out: the registers as the last steps leave them (eax the count, ebx, ecx 5, edx, esi,
//!      edi); no flags read
//!
//! Callers: `sub_43B635` (0x43B70F), `sub_43A1CA`. Refuses no faces, a table index (+0x2C)
//! past 5 (before anything) and a surface past 15 (what was written by then is put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::CAR;
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::flip_cell_index::flip_cell_index;
use crate::physics::tables_store::PHYSICS;
use crate::pointers::GamePtr;
use crate::race::course_store::COURSE;
use crate::race::view_store::RACE_VIEW;
use crate::scene::draw_store::DRAW;
use crate::scene::x87_mem::{fild, fistp, fld, fstp};
use std::cmp::Ordering;

fn byte(a: u32) -> Result<u32, Refused> {
    Ok(u32::from(
        GamePtr::new(a).read::<u8>("a face's or texture record's byte")?,
    ))
}

/// `fld v[c] x 65536; fistp` through `dword_62400C` (`v` the table's offset from the base).
fn share_bit(v: u32, c: u32) -> u32 {
    // The tables (0x6D2A14, 0x6D2B14) are the race core's (`race::store`).
    let b = crate::race::store::asm_base();
    x87_1(
        (b + v) as usize,
        c as usize,
        crate::race::store::asm_base() as usize,
    );
    DRAW.scratch.get()
}

/// Everything the function reads that the checks need, refused as said.
fn check() -> Result<(), Refused> {
    let faces = COURSE.course_faces.get();
    if faces == 0 {
        return Err(BadInput("no course faces").into());
    }
    if CAR.tyre_kind.get() > 5 {
        return Err(BadInput("a surface table index past 5").into());
    }
    Ok(())
}

/// The steps (a refusal may come after some are written: `wheel_surfaces` puts them
/// back).
fn steps() -> Result<(), Refused> {
    check()?;
    let infos = crate::scene::polygon::texture_infos_addr();
    let surfaces = crate::pointers::game_address(PHYSICS.surfaces.as_ptr());
    for c in 0..5u32 {
        let cc = c as usize;
        let f = (CAR.probe_point_face[cc].get() << 6).wrapping_add(COURSE.course_faces.get());
        let t1 = infos + (byte(f + 1)? << 6);
        let mut edx = share_bit(0x2D2B14, c);
        let mut ebx = share_bit(0x2D2A14, c);
        edx = (edx >> 15) & 1;
        ebx = (ebx >> 15) & 1;
        if CONTACT_WORK.probe_face_flip[cc].get() == 1 {
            edx ^= 1;
            ebx ^= 1;
        }
        let (_, _, d1) = flip_cell_index(byte(f)?, ebx, edx);
        PHYSICS.surface_firsts[cc].set(d1);
        let (_, _, d2) = flip_cell_index(byte(f + 0x20)?, ebx, edx);
        PHYSICS.surface_seconds[cc].set(d2);
        {
            let e = PHYSICS.surface_firsts[cc].get();
            CAR.wheel_surface_a[cc].set(byte(e + t1 + 0x10)?);
            CAR.wheel_cell_a_byte18[cc].set(byte(e + t1 + 0x18)?);
            CAR.wheel_cell_a_byte14[cc].set(byte(e + t1 + 0x14)?);
            let t2 = infos + (byte(f + 0x21)? << 6);
            let e2 = PHYSICS.surface_seconds[cc].get();
            CAR.wheel_surface_b[cc].set(byte(e2 + t2 + 0x10)?);
            CAR.wheel_cell_b_byte18[cc].set(byte(e2 + t2 + 0x18)?);
            CAR.wheel_cell_b_byte14[cc].set(byte(e2 + t2 + 0x14)?);
            CAR.wheel_surface_blend[cc].set_bits(0);
            let kind_index = byte(t2 + 0x2C)?;
            // The index is a byte: past the four kinds the original reads on (a slip).
            let kind = match RACE_VIEW.surface_kinds.get(kind_index as usize) {
                Some(cell) => cell.get(),
                None => GamePtr::new(
                    RACE_VIEW.surface_kinds[0].addr() + (4 * kind_index as usize) as u32,
                )
                .read::<u32>("the surface kinds (a byte index past the table)")?,
            };
            if kind != 5 && kind != 4 && CAR.wheel_surface_b[cc].get() != 0 && byte(f + 0x21)? != 0
            {
                let (x, y, z) = if CONTACT_WORK.probe_face_flip[cc].get() == 1 {
                    (byte(f + 0x25)?, byte(f + 0x24)?, byte(f + 0x23)?)
                } else {
                    (byte(f + 0x22)?, byte(f + 0x23)?, byte(f + 0x24)?)
                };
                DRAW.surface_point[0].set(x);
                DRAW.surface_point[1].set(y.wrapping_sub(x));
                DRAW.surface_point[2].set(z.wrapping_sub(x));
                x87_2(crate::race::store::asm_base() as usize, c as usize);
            }
            if GamePtr::new(f + 2).read::<u16>("a face's flags")? & 0x1000 != 0 {
                CAR.wheel_surface_a[cc].set(5);
                CAR.wheel_surface_b[cc].set(5);
                CAR.wheel_cell_a_byte18[cc].set(0x3C);
                CAR.wheel_cell_b_byte18[cc].set(0x3C);
                CAR.wheel_cell_a_byte14[cc].set(0);
                CAR.wheel_cell_b_byte14[cc].set(0);
            }
            let (s1, s2) = (CAR.wheel_surface_a[cc].get(), CAR.wheel_surface_b[cc].get());
            if s1 > 15 || s2 > 15 {
                return Err(BadInput("a surface past 15").into());
            }
            let esi = surfaces + s1 * 0x30;
            let edi = surfaces + s2 * 0x30;
            x87_3(
                esi as usize,
                edi as usize,
                crate::race::store::asm_base() as usize,
                c as usize,
            );
            let (ps, pd) = (
                GamePtr::new(esi).read::<u32>("a surface record")?,
                GamePtr::new(edi).read::<u32>("a surface record")?,
            );
            let k = CAR.tyre_kind.get();
            x87_4(
                ps as usize,
                crate::race::store::asm_base() as usize,
                pd as usize,
                k as usize,
            );
            x87_5(
                esi as usize,
                crate::race::store::asm_base() as usize,
                edi as usize,
                c as usize,
            );
        }
    }
    x87_6(crate::race::store::asm_base() as usize);
    for c in 0..5usize {
        let hard = |s: u32| matches!(s, 1 | 2 | 3 | 0xC);
        let on = hard(CAR.wheel_surface_a[c].get()) || hard(CAR.wheel_surface_b[c].get());
        PHYSICS.hard_ground[c].set(on as u32);
    }
    let hard = PHYSICS.hard_ground[0]
        .get()
        .wrapping_add(PHYSICS.hard_ground[1].get())
        .wrapping_add(PHYSICS.hard_ground[2].get())
        .wrapping_add(PHYSICS.hard_ground[3].get());
    CAR.hard_ground_count.set(hard);
    let below = |a: u32, b: u32| {
        !matches!(
            f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
            Some(Ordering::Greater | Ordering::Equal)
        )
    };
    let mut least = if below(PHYSICS.point_grips[0].bits(), PHYSICS.point_grips[1].bits()) {
        PHYSICS.point_grips[0].bits()
    } else {
        PHYSICS.point_grips[1].bits()
    };
    CAR.ground_bump_scale
        .set_bits(crate::math::x87::loaded(least));
    for k in &PHYSICS.point_grips[2..5] {
        least = if below(CAR.ground_bump_scale.bits(), k.bits()) {
            CAR.ground_bump_scale.bits()
        } else {
            k.bits()
        };
        CAR.ground_bump_scale
            .set_bits(crate::math::x87::loaded(least));
    }
    x87_7(crate::race::store::asm_base() as usize);
    Ok(())
}

/// The Rust version: what was written is put back on a refusal.
pub fn wheel_surfaces() -> Result<(), Refused> {
    steps()
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x22400C] = [t + c * 4] * [rc + 0x1C5D98]`
fn x87_1(t: usize, c: usize, rc: usize) {
    let t0 = fld(t, c * 4);
    let t1 = fld(rc, 0x1C5D98);
    let t2 = t0 * t1;
    fistp(rc, 0x22400C, t2);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_2(rc: usize, c: usize) {
    let t0 = fild(rc, 0x224044);
    let t1 = fld(rc, c * 4 + 0x2D2A14);
    let t2 = t0 * t1;
    let t3 = fild(rc, 0x224048);
    let t4 = fld(rc, c * 4 + 0x2D2B14);
    let t5 = t3 * t4;
    let t6 = fild(rc, 0x224040);
    let t7 = t5 + t6;
    let t8 = t2 + t7;
    let t9 = fld(rc, 0x1C5E2C);
    let t10 = t8 / t9;
    fstp(rc, c * 4 + 0x31A984, t10);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + c * 4 + 0x2142C0] = [s + 0x28] + (([d + 0x28] - [s + 0x28]) * [rc + c * 4 + 0x31A984])`
/// `[rc + c * 4 + 0x31A920] = [s + 0x2C] + (([d + 0x2C] - [s + 0x2C]) * [rc + c * 4 + 0x31A984])`
fn x87_3(s: usize, d: usize, rc: usize, c: usize) {
    let t0 = fld(s, 0x28);
    let t1 = fld(d, 0x28);
    let t2 = t1 - t0;
    let t3 = fld(rc, c * 4 + 0x31A984);
    let t4 = t2 * t3;
    let t5 = t0 + t4;
    fstp(rc, c * 4 + 0x2142C0, t5);
    let t6 = fld(s, 0x2C);
    let t7 = fld(d, 0x2C);
    let t8 = t7 - t6;
    let t9 = fld(rc, c * 4 + 0x31A984);
    let t10 = t8 * t9;
    let t11 = t6 + t10;
    fstp(rc, c * 4 + 0x31A920, t11);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x21411C] = (int [s + k * 4 + 4] + ((int [s + k * 4 + 0x1C] - int [s + k * 4 + 4]) * [rc + 0x31A258])) * [rc + 0x1C5CFC]`
/// `[rc + 0x214120] = (int [d + k * 4 + 4] + ((int [d + k * 4 + 0x1C] - int [d + k * 4 + 4]) * [rc + 0x31A258])) * [rc + 0x1C5CFC]`
fn x87_4(s: usize, rc: usize, d: usize, k: usize) {
    let t6 = fild(s, k * 4 + 4);
    let t7 = fild(s, k * 4 + 0x1C);
    let t8 = t7 - t6;
    let t9 = fld(rc, 0x31A258);
    let t10 = t8 * t9;
    let t11 = t6 + t10;
    let t12 = fld(rc, 0x1C5CFC);
    let t13 = t11 * t12;
    fstp(rc, 0x21411C, t13);
    let t14 = fild(d, k * 4 + 4);
    let t15 = fild(d, k * 4 + 0x1C);
    let t16 = t15 - t14;
    let t17 = fld(rc, 0x31A258);
    let t18 = t16 * t17;
    let t19 = t14 + t18;
    let t20 = fld(rc, 0x1C5CFC);
    let t21 = t19 * t20;
    fstp(rc, 0x214120, t21);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// `[rc + c * 4 + 0x2146C0] = [s + 0x20] + (([d + 0x20] - [s + 0x20]) * [rc + c * 4 + 0x31A984])`
/// `[rc + c * 4 + 0x2146D4] = [s + 0x24] + (([d + 0x24] - [s + 0x24]) * [rc + c * 4 + 0x31A984])`
/// `[rc + c * 4 + 0x2146E8] = [s + 0x14] + (([d + 0x14] - [s + 0x14]) * [rc + c * 4 + 0x31A984])`
/// `[rc + c * 4 + 0x2146FC] = [s + 0x10] + (([d + 0x10] - [s + 0x10]) * [rc + c * 4 + 0x31A984])`
fn x87_5(s: usize, rc: usize, d: usize, c: usize) {
    let t0 = fld(s, 4);
    let t1 = fld(rc, 0x21411C);
    let t2 = t0 * t1;
    let t3 = fld(d, 4);
    let t4 = fld(rc, 0x214120);
    let t5 = t3 * t4;
    let t6 = t5 - t2;
    let t7 = fld(rc, c * 4 + 0x31A984);
    let t8 = t6 * t7;
    let t9 = t2 + t8;
    let t10 = fld(rc, 0x214BBC);
    let t11 = t9 * t10;
    fstp(rc, c * 4 + 0x214284, t11);
    let t12 = fld(s, 8);
    let t13 = fld(rc, 0x21411C);
    let t14 = t12 * t13;
    let t15 = fld(d, 8);
    let t16 = fld(rc, 0x214120);
    let t17 = t15 * t16;
    let t18 = t17 - t14;
    let t19 = fld(rc, c * 4 + 0x31A984);
    let t20 = t18 * t19;
    let t21 = t14 + t20;
    let t22 = fld(rc, 0x214BB4);
    let t23 = t21 * t22;
    fstp(rc, c * 4 + 0x214298, t23);
    let t24 = fld(s, 0xC);
    let t25 = fld(rc, 0x21411C);
    let t26 = t24 * t25;
    let t27 = fld(d, 0xC);
    let t28 = fld(rc, 0x214120);
    let t29 = t27 * t28;
    let t30 = t29 - t26;
    let t31 = fld(rc, c * 4 + 0x31A984);
    let t32 = t30 * t31;
    let t33 = t26 + t32;
    let t34 = fld(rc, 0x214BB8);
    let t35 = t33 * t34;
    fstp(rc, c * 4 + 0x2142AC, t35);
    let t36 = fld(s, 0x20);
    let t37 = fld(d, 0x20);
    let t38 = t37 - t36;
    let t39 = fld(rc, c * 4 + 0x31A984);
    let t40 = t38 * t39;
    let t41 = t36 + t40;
    fstp(rc, c * 4 + 0x2146C0, t41);
    let t42 = fld(s, 0x24);
    let t43 = fld(d, 0x24);
    let t44 = t43 - t42;
    let t45 = fld(rc, c * 4 + 0x31A984);
    let t46 = t44 * t45;
    let t47 = t42 + t46;
    fstp(rc, c * 4 + 0x2146D4, t47);
    let t48 = fld(s, 0x14);
    let t49 = fld(d, 0x14);
    let t50 = t49 - t48;
    let t51 = fld(rc, c * 4 + 0x31A984);
    let t52 = t50 * t51;
    let t53 = t48 + t52;
    fstp(rc, c * 4 + 0x2146E8, t53);
    let t54 = fld(s, 0x10);
    let t55 = fld(d, 0x10);
    let t56 = t55 - t54;
    let t57 = fld(rc, c * 4 + 0x31A984);
    let t58 = t56 * t57;
    let t59 = t54 + t58;
    fstp(rc, c * 4 + 0x2146FC, t59);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214268] = ((([rc + 0x214284] + [rc + 0x214288]) + [rc + 0x21428C]) + [rc + 0x214290]) * [rc + 0x213FEC]`
fn x87_6(rc: usize) {
    let t0 = fld(rc, 0x214284);
    let t1 = fld(rc, 0x214288);
    let t2 = t0 + t1;
    let t3 = fld(rc, 0x21428C);
    let t4 = t2 + t3;
    let t5 = fld(rc, 0x214290);
    let t6 = t4 + t5;
    let t7 = fld(rc, 0x213FEC);
    let t8 = t6 * t7;
    fstp(rc, 0x214268, t8);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A038] = [rc + 0x31A038] * [rc + 0x214C04]`
/// `[rc + 0x224040] = (([rc + 0x213F5C] - [rc + 0x213F64]) * [rc + 0x31A258]) + [rc + 0x213F64]`
/// `[rc + 0x224044] = (([rc + 0x213F60] - [rc + 0x213F68]) * [rc + 0x31A258]) + [rc + 0x213F68]`
/// `[rc + 0x214230] = (([rc + 0x224044] - [rc + 0x224040]) * (int [rc + 0x31A00C] / [rc + 0x1C5E2C])) + [rc + 0x224040]`
/// `[rc + 0x214760] = [rc + 0x2146D0]`
/// `[rc + 0x214764] = [rc + 0x2146E4]`
/// `[rc + 0x223DA8] = [rc + 0x2146F8]`
/// `[rc + 0x223DAC] = [rc + 0x21470C]`
fn x87_7(rc: usize) {
    let t0 = fld(rc, 0x31A038);
    let t1 = fld(rc, 0x214C04);
    let t2 = t0 * t1;
    fstp(rc, 0x31A038, t2);
    let t3 = fld(rc, 0x213F5C);
    let t4 = fld(rc, 0x213F64);
    let t5 = t3 - t4;
    let t6 = fld(rc, 0x31A258);
    let t7 = t5 * t6;
    let t8 = fld(rc, 0x213F64);
    let t9 = t7 + t8;
    fstp(rc, 0x224040, t9);
    let t10 = fld(rc, 0x213F60);
    let t11 = fld(rc, 0x213F68);
    let t12 = t10 - t11;
    let t13 = fld(rc, 0x31A258);
    let t14 = t12 * t13;
    let t15 = fld(rc, 0x213F68);
    let t16 = t14 + t15;
    fstp(rc, 0x224044, t16);
    let t17 = fld(rc, 0x224044);
    let t18 = fld(rc, 0x224040);
    let t19 = t17 - t18;
    let t20 = fild(rc, 0x31A00C);
    let t21 = fld(rc, 0x1C5E2C);
    let t22 = t20 / t21;
    let t23 = t19 * t22;
    let t24 = fld(rc, 0x224040);
    let t25 = t23 + t24;
    fstp(rc, 0x214230, t25);
    let t26 = fld(rc, 0x2146D0);
    fstp(rc, 0x214760, t26);
    let t27 = fld(rc, 0x2146E4);
    fstp(rc, 0x214764, t27);
    let t28 = fld(rc, 0x2146F8);
    fstp(rc, 0x223DA8, t28);
    let t29 = fld(rc, 0x21470C);
    fstp(rc, 0x223DAC, t29);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(t, c, rc) "fld dword ptr [{t} + {c}*4]", "fmul dword ptr [{rc} + 0x1C5D98]", "fistp dword ptr [{rc} + 0x22400C]");
    reference!(ref_1(rc, c) "fild dword ptr [{rc} + 0x224044]", "fmul dword ptr [{rc} + {c}*4 + 0x2D2A14]", "fild dword ptr [{rc} + 0x224048]", "fmul dword ptr [{rc} + {c}*4 + 0x2D2B14]", "fild dword ptr [{rc} + 0x224040]", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fstp dword ptr [{rc} + {c}*4 + 0x31A984]");
    reference!(ref_2(s, d, rc, c) "fld dword ptr [{s} + 0x28]", "fld dword ptr [{d} + 0x28]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + {c}*4 + 0x2142C0]", "fld dword ptr [{s} + 0x2C]", "fld dword ptr [{d} + 0x2C]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + {c}*4 + 0x31A920]");
    reference!(ref_3(s, rc, d, k) "fild dword ptr [{s}]", "fmul dword ptr [{rc} + 0x1C5CFC]", "fstp dword ptr [{rc} + 0x21411C]", "fild dword ptr [{d}]", "fmul dword ptr [{rc} + 0x1C5CFC]", "fstp dword ptr [{rc} + 0x214120]", "fild dword ptr [{s} + {k}*4 + 4]", "fild dword ptr [{s} + {k}*4 + 0x1C]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + 0x31A258]", ".byte 0xDE, 0xC1", "fmul dword ptr [{rc} + 0x1C5CFC]", "fstp dword ptr [{rc} + 0x21411C]", "fild dword ptr [{d} + {k}*4 + 4]", "fild dword ptr [{d} + {k}*4 + 0x1C]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + 0x31A258]", ".byte 0xDE, 0xC1", "fmul dword ptr [{rc} + 0x1C5CFC]", "fstp dword ptr [{rc} + 0x214120]");
    reference!(ref_4(s, rc, d, c) "fld dword ptr [{s} + 0x4]", "fmul dword ptr [{rc} + 0x21411C]", "fld dword ptr [{d} + 0x4]", "fmul dword ptr [{rc} + 0x214120]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fmul dword ptr [{rc} + 0x214BBC]", "fstp dword ptr [{rc} + {c}*4 + 0x214284]", "fld dword ptr [{s} + 0x8]", "fmul dword ptr [{rc} + 0x21411C]", "fld dword ptr [{d} + 0x8]", "fmul dword ptr [{rc} + 0x214120]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fmul dword ptr [{rc} + 0x214BB4]", "fstp dword ptr [{rc} + {c}*4 + 0x214298]", "fld dword ptr [{s} + 0xC]", "fmul dword ptr [{rc} + 0x21411C]", "fld dword ptr [{d} + 0xC]", "fmul dword ptr [{rc} + 0x214120]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fmul dword ptr [{rc} + 0x214BB8]", "fstp dword ptr [{rc} + {c}*4 + 0x2142AC]", "fld dword ptr [{s} + 0x20]", "fld dword ptr [{d} + 0x20]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + {c}*4 + 0x2146C0]", "fld dword ptr [{s} + 0x24]", "fld dword ptr [{d} + 0x24]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + {c}*4 + 0x2146D4]", "fld dword ptr [{s} + 0x14]", "fld dword ptr [{d} + 0x14]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + {c}*4 + 0x2146E8]", "fld dword ptr [{s} + 0x10]", "fld dword ptr [{d} + 0x10]", ".byte 0xD8, 0xE1", "fmul dword ptr [{rc} + {c}*4 + 0x31A984]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + {c}*4 + 0x2146FC]");
    reference!(ref_5(rc) "fld dword ptr [{rc} + 0x214284]", "fadd dword ptr [{rc} + 0x214288]", "fadd dword ptr [{rc} + 0x21428C]", "fadd dword ptr [{rc} + 0x214290]", "fmul dword ptr [{rc} + 0x213FEC]", "fstp dword ptr [{rc} + 0x214268]");
    reference!(ref_6(rc) "fld dword ptr [{rc} + 0x31A038]", "fmul dword ptr [{rc} + 0x214C04]", "fstp dword ptr [{rc} + 0x31A038]", "fld dword ptr [{rc} + 0x213F5C]", "fsub dword ptr [{rc} + 0x213F64]", "fmul dword ptr [{rc} + 0x31A258]", "fadd dword ptr [{rc} + 0x213F64]", "fstp dword ptr [{rc} + 0x224040]", "fld dword ptr [{rc} + 0x213F60]", "fsub dword ptr [{rc} + 0x213F68]", "fmul dword ptr [{rc} + 0x31A258]", "fadd dword ptr [{rc} + 0x213F68]", "fstp dword ptr [{rc} + 0x224044]", "fld dword ptr [{rc} + 0x224044]", "fsub dword ptr [{rc} + 0x224040]", "fild dword ptr [{rc} + 0x31A00C]", "fdiv dword ptr [{rc} + 0x1C5E2C]", ".byte 0xDE, 0xC9", "fadd dword ptr [{rc} + 0x224040]", "fstp dword ptr [{rc} + 0x214230]", "fld dword ptr [{rc} + 0x2146D0]", "fstp dword ptr [{rc} + 0x214760]", "fld dword ptr [{rc} + 0x2146E4]", "fstp dword ptr [{rc} + 0x214764]", "fld dword ptr [{rc} + 0x2146F8]", "fstp dword ptr [{rc} + 0x223DA8]", "fld dword ptr [{rc} + 0x21470C]", "fstp dword ptr [{rc} + 0x223DAC]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "wheel_surfaces line 68 idx c=0",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Value(0),
                    Operand::Buffer {
                        reads: &[(0x1C5D98, In::F)],
                        writes: &[(0x22400C, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| x87_1(a[0], a[1], a[2]),
            },
            Block {
                name: "wheel_surfaces line 68 idx c=1",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(4, In::F)],
                        writes: &[],
                    },
                    Operand::Value(1),
                    Operand::Buffer {
                        reads: &[(0x1C5D98, In::F)],
                        writes: &[(0x22400C, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| x87_1(a[0], a[1], a[2]),
            },
            Block {
                name: "wheel_surfaces line 68 idx c=4",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x10, In::F)],
                        writes: &[],
                    },
                    Operand::Value(4),
                    Operand::Buffer {
                        reads: &[(0x1C5D98, In::F)],
                        writes: &[(0x22400C, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| x87_1(a[0], a[1], a[2]),
            },
            Block {
                name: "wheel_surfaces line 134 idx c=0",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x224040, In::I),
                            (0x224044, In::I),
                            (0x224048, In::I),
                            (0x2D2A14, In::F),
                            (0x2D2B14, In::F),
                        ],
                        writes: &[(0x31A984, Out::F)],
                    },
                    Operand::Value(0),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1]),
            },
            Block {
                name: "wheel_surfaces line 134 idx c=1",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x224040, In::I),
                            (0x224044, In::I),
                            (0x224048, In::I),
                            (0x2D2A18, In::F),
                            (0x2D2B18, In::F),
                        ],
                        writes: &[(0x31A988, Out::F)],
                    },
                    Operand::Value(1),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1]),
            },
            Block {
                name: "wheel_surfaces line 134 idx c=4",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x224040, In::I),
                            (0x224044, In::I),
                            (0x224048, In::I),
                            (0x2D2A24, In::F),
                            (0x2D2B24, In::F),
                        ],
                        writes: &[(0x31A994, Out::F)],
                    },
                    Operand::Value(4),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1]),
            },
            Block {
                name: "wheel_surfaces line 161 idx c=0",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x28, In::F), (0x2C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x28, In::F), (0x2C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A984, In::F)],
                        writes: &[(0x2142C0, Out::F), (0x31A920, Out::F)],
                    },
                    Operand::Value(0),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_3(a[0], a[1], a[2], a[3]),
            },
            Block {
                name: "wheel_surfaces line 161 idx c=1",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x28, In::F), (0x2C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x28, In::F), (0x2C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A988, In::F)],
                        writes: &[(0x2142C4, Out::F), (0x31A924, Out::F)],
                    },
                    Operand::Value(1),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_3(a[0], a[1], a[2], a[3]),
            },
            Block {
                name: "wheel_surfaces line 161 idx c=4",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x28, In::F), (0x2C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x28, In::F), (0x2C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x31A994, In::F)],
                        writes: &[(0x2142D0, Out::F), (0x31A930, Out::F)],
                    },
                    Operand::Value(4),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_3(a[0], a[1], a[2], a[3]),
            },
            Block {
                name: "wheel_surfaces line 178 idx k=0",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I), (4, In::I), (0x1C, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x1C5CFC, In::F), (0x31A258, In::F)],
                        writes: &[(0x21411C, Out::F), (0x214120, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::I), (4, In::I), (0x1C, In::I)],
                        writes: &[],
                    },
                    Operand::Value(0),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_4(a[0], a[1], a[2], a[3]),
            },
            Block {
                name: "wheel_surfaces line 178 idx k=1",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I), (8, In::I), (0x20, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x1C5CFC, In::F), (0x31A258, In::F)],
                        writes: &[(0x21411C, Out::F), (0x214120, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::I), (8, In::I), (0x20, In::I)],
                        writes: &[],
                    },
                    Operand::Value(1),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_4(a[0], a[1], a[2], a[3]),
            },
            Block {
                name: "wheel_surfaces line 178 idx k=4",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I), (0x14, In::I), (0x2C, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x1C5CFC, In::F), (0x31A258, In::F)],
                        writes: &[(0x21411C, Out::F), (0x214120, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::I), (0x14, In::I), (0x2C, In::I)],
                        writes: &[],
                    },
                    Operand::Value(4),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_4(a[0], a[1], a[2], a[3]),
            },
            Block {
                name: "wheel_surfaces line 201 idx c=0",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                            (0x20, In::F),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[
                            (0x21411C, In::F),
                            (0x214120, In::F),
                            (0x214BB4, In::F),
                            (0x214BB8, In::F),
                            (0x214BBC, In::F),
                            (0x31A984, In::F),
                        ],
                        writes: &[
                            (0x214284, Out::F),
                            (0x214298, Out::F),
                            (0x2142AC, Out::F),
                            (0x2146C0, Out::F),
                            (0x2146D4, Out::F),
                            (0x2146E8, Out::F),
                            (0x2146FC, Out::F),
                        ],
                    },
                    Operand::Buffer {
                        reads: &[
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                            (0x20, In::F),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Value(0),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_5(a[0], a[1], a[2], a[3]),
            },
            Block {
                name: "wheel_surfaces line 201 idx c=1",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                            (0x20, In::F),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[
                            (0x21411C, In::F),
                            (0x214120, In::F),
                            (0x214BB4, In::F),
                            (0x214BB8, In::F),
                            (0x214BBC, In::F),
                            (0x31A988, In::F),
                        ],
                        writes: &[
                            (0x214288, Out::F),
                            (0x21429C, Out::F),
                            (0x2142B0, Out::F),
                            (0x2146C4, Out::F),
                            (0x2146D8, Out::F),
                            (0x2146EC, Out::F),
                            (0x214700, Out::F),
                        ],
                    },
                    Operand::Buffer {
                        reads: &[
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                            (0x20, In::F),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Value(1),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_5(a[0], a[1], a[2], a[3]),
            },
            Block {
                name: "wheel_surfaces line 201 idx c=4",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                            (0x20, In::F),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[
                            (0x21411C, In::F),
                            (0x214120, In::F),
                            (0x214BB4, In::F),
                            (0x214BB8, In::F),
                            (0x214BBC, In::F),
                            (0x31A994, In::F),
                        ],
                        writes: &[
                            (0x214294, Out::F),
                            (0x2142A8, Out::F),
                            (0x2142BC, Out::F),
                            (0x2146D0, Out::F),
                            (0x2146E4, Out::F),
                            (0x2146F8, Out::F),
                            (0x21470C, Out::F),
                        ],
                    },
                    Operand::Buffer {
                        reads: &[
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                            (0x20, In::F),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Value(4),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_5(a[0], a[1], a[2], a[3]),
            },
            Block {
                name: "wheel_surfaces line 258",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x213FEC, In::F),
                        (0x214284, In::F),
                        (0x214288, In::F),
                        (0x21428C, In::F),
                        (0x214290, In::F),
                    ],
                    writes: &[(0x214268, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0]) },
                plain: |a| x87_6(a[0]),
            },
            Block {
                name: "wheel_surfaces line 299",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x213F5C, In::F),
                        (0x213F60, In::F),
                        (0x213F64, In::F),
                        (0x213F68, In::F),
                        (0x2146D0, In::F),
                        (0x2146E4, In::F),
                        (0x2146F8, In::F),
                        (0x21470C, In::F),
                        (0x214C04, In::F),
                        (0x224040, In::F),
                        (0x224044, In::F),
                        (0x31A00C, In::I),
                        (0x31A038, In::F),
                        (0x31A258, In::F),
                    ],
                    writes: &[
                        (0x214230, Out::F),
                        (0x214760, Out::F),
                        (0x214764, Out::F),
                        (0x223DA8, Out::F),
                        (0x223DAC, Out::F),
                        (0x224040, Out::F),
                        (0x224044, Out::F),
                        (0x31A038, Out::F),
                    ],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0]) },
                plain: |a| x87_7(a[0]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
