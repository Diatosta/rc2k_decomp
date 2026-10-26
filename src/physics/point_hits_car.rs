//! `physics::point_hits_car` (0x45E800): whether a point of an object lies inside the
//! work car's box. `dword_614110` + 1; the point (three whole numbers at esi) plus the
//! object's place (ebp +0x30 ..) plus the offset `dword_6D3B40` .. less the car's place
//! (`unk_71A10C` ..) into `dword_6D3B4C` ..; that / `flt_5C5D98` into `dword_6D3B78` ..
//! and `flt_6D3B58` .., turned by the car's heading (`physics::turn_by_heading`, ebx the
//! y) and back to whole numbers (x `flt_5C5D98`, `fistp`) into `dword_6D3090` ... Inside
//! when x is below `dword_614BC8` and not below minus it, and z below `dword_614BC0` and
//! not below minus `dword_614BC4` (each by the sign of the `cmp`, as the original). Inside
//! with the contact kind `dword_6D3624` 10, 1, 5 or 6 also: the point's x, 0, z (/
//! `flt_5C5D98`) into `unk_71A1FC` .., the car's speed's length (`unk_71A170`,
//! `physics::vector_length`) into `flt_6D3108`, `unk_71A208` raised to its size,
//! `unk_71A20C` = 1, `unk_71A1CC` = 0x32. All in `pusha` / `popa`.
//!
//! In:  esi the point, ebp the object
//! Out: CF set when inside; every register kept
//!
//! Callers: `physics::object_hits_car`, `physics::scene_point_hits_car` (both `jb`).

use crate::errors::Refused;
use crate::math::x87_float::X;
use crate::physics::car_store::CAR;
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::tables_store::PHYSICS;
use crate::physics::turn_by_heading::turn_by_heading;
use crate::physics::vector_length::vector_length;
use crate::pointers::GamePtr;
use crate::scene::matrix3::rounded;
use crate::scene::x87_mem::{fild, fistp, fld, fstp};
use rc2k_core::physics::contacts;

/// The Rust version: whether the point at `point` of the object at `object` is inside.
// Converted: `rc2k_core::physics::contacts::{point_part, inside_box, kind_hits}` (the x87
// steps stay).
pub unsafe fn point_hits_car(point: u32, object: u32) -> Result<bool, Refused> {
    let rd = |x: u32, what: &str| GamePtr::new(x).read::<u32>(what);
    PHYSICS.point_hits.set(
        crate::physics::tables_store::PHYSICS
            .point_hits
            .get()
            .wrapping_add(1),
    );
    let mut y = 0;
    for k in 0..3u32 {
        let v = contacts::point_part(
            rd(point + 4 * k, "a scene point's coordinate")?,
            rd(object + 0x30 + 4 * k, "an object's place")?,
            CONTACT_WORK.object_offset[k as usize].get(),
            CAR.position[k as usize].get(),
        );
        CONTACT_WORK.object_point_offset[k as usize].set(v);
        if k == 1 {
            y = v;
        }
    }
    x87_1(crate::race::store::asm_base() as usize);
    for k in 0..3 {
        CONTACT_WORK.object_point_turned[k].set_bits(CONTACT_WORK.object_point_scaled[k].bits());
    }
    unsafe { turn_by_heading(CONTACT_WORK.object_point_turned[0].addr() as u32, y) };
    x87_2(crate::race::store::asm_base() as usize);
    let (x, z) = (
        CONTACT_WORK.contact_point[0].bits(),
        CONTACT_WORK.contact_point[2].bits(),
    );
    if !contacts::inside_box(
        x,
        z,
        PHYSICS.spec_ints[2].get(),
        PHYSICS.spec_ints[0].get(),
        PHYSICS.spec_ints[1].get(),
    ) {
        return Ok(false);
    }
    if contacts::kind_hits(CONTACT_WORK.contact_kind.get()) {
        x87_3(crate::race::store::asm_base() as usize);
        CONTACT_WORK.contact_speed_length.set_bits(rounded(unsafe {
            vector_length(CAR.velocity[0].addr() as u32)
        }));
        if let Some(hit) = rc2k_core::physics::walls::raised_hit(
            f32::from_bits(CONTACT_WORK.contact_speed_length.bits()),
            f32::from_bits(CAR.contact_depth.bits()),
        ) {
            CAR.contact_depth.set_bits(hit.to_bits());
        }
        CAR.contact_flag.set(1);
        CAR.wall_hit_timer.set(0x32);
    }
    Ok(true)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2D3B78] = int [rc + 0x2D3B4C] / [rc + 0x1C5D98]`
/// `[rc + 0x2D3B7C] = int [rc + 0x2D3B50] / [rc + 0x1C5D98]`
/// `[rc + 0x2D3B80] = int [rc + 0x2D3B54] / [rc + 0x1C5D98]`
fn x87_1(rc: usize) {
    let t0 = fild(rc, 0x2D3B4C);
    let t1 = fld(rc, 0x1C5D98);
    let t2 = t0 / t1;
    fstp(rc, 0x2D3B78, t2);
    let t3 = fild(rc, 0x2D3B50);
    let t4 = fld(rc, 0x1C5D98);
    let t5 = t3 / t4;
    fstp(rc, 0x2D3B7C, t5);
    let t6 = fild(rc, 0x2D3B54);
    let t7 = fld(rc, 0x1C5D98);
    let t8 = t6 / t7;
    fstp(rc, 0x2D3B80, t8);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x2D3090] = [rc + 0x2D3B58] * [rc + 0x1C5D98]`
/// `int [rc + 0x2D3094] = [rc + 0x2D3B5C] * [rc + 0x1C5D98]`
/// `int [rc + 0x2D3098] = [rc + 0x2D3B60] * [rc + 0x1C5D98]`
fn x87_2(rc: usize) {
    let t0 = fld(rc, 0x2D3B58);
    let t1 = fld(rc, 0x1C5D98);
    let t2 = t0 * t1;
    fistp(rc, 0x2D3090, t2);
    let t3 = fld(rc, 0x2D3B5C);
    let t4 = fld(rc, 0x1C5D98);
    let t5 = t3 * t4;
    fistp(rc, 0x2D3094, t5);
    let t6 = fld(rc, 0x2D3B60);
    let t7 = fld(rc, 0x1C5D98);
    let t8 = t6 * t7;
    fistp(rc, 0x2D3098, t8);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A1FC] = int [rc + 0x2D3090] / [rc + 0x1C5D98]`
/// `[rc + 0x31A200] = 0`
/// `[rc + 0x31A204] = int [rc + 0x2D3098] / [rc + 0x1C5D98]`
fn x87_3(rc: usize) {
    let t0 = fild(rc, 0x2D3090);
    let t1 = fld(rc, 0x1C5D98);
    let t2 = t0 / t1;
    fstp(rc, 0x31A1FC, t2);
    let t3 = X(0.0);
    fstp(rc, 0x31A200, t3);
    let t4 = fild(rc, 0x2D3098);
    let t5 = fld(rc, 0x1C5D98);
    let t6 = t4 / t5;
    fstp(rc, 0x31A204, t6);
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) "fild dword ptr [{rc} + 0x2D3B4C]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x2D3B78]", "fild dword ptr [{rc} + 0x2D3B50]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x2D3B7C]", "fild dword ptr [{rc} + 0x2D3B54]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x2D3B80]");
    reference!(ref_1(rc) "fld dword ptr [{rc} + 0x2D3B58]", "fmul dword ptr [{rc} + 0x1C5D98]", "fistp dword ptr [{rc} + 0x2D3090]", "fld dword ptr [{rc} + 0x2D3B5C]", "fmul dword ptr [{rc} + 0x1C5D98]", "fistp dword ptr [{rc} + 0x2D3094]", "fld dword ptr [{rc} + 0x2D3B60]", "fmul dword ptr [{rc} + 0x1C5D98]", "fistp dword ptr [{rc} + 0x2D3098]");
    reference!(ref_2(rc) "fild dword ptr [{rc} + 0x2D3090]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x31A1FC]", ".byte 0xD9, 0xEE", "fstp dword ptr [{rc} + 0x31A200]", "fild dword ptr [{rc} + 0x2D3098]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x31A204]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "point_hits_car line 47",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x1C5D98, In::F),
                        (0x2D3B4C, In::I),
                        (0x2D3B50, In::I),
                        (0x2D3B54, In::I),
                    ],
                    writes: &[(0x2D3B78, Out::F), (0x2D3B7C, Out::F), (0x2D3B80, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0]) },
                plain: |a| x87_1(a[0]),
            },
            Block {
                name: "point_hits_car line 63",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x1C5D98, In::F),
                        (0x2D3B58, In::F),
                        (0x2D3B5C, In::F),
                        (0x2D3B60, In::F),
                    ],
                    writes: &[(0x2D3090, Out::I), (0x2D3094, Out::I), (0x2D3098, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0]) },
                plain: |a| x87_2(a[0]),
            },
            Block {
                name: "point_hits_car line 80",
                operands: vec![Operand::Buffer {
                    reads: &[(0x1C5D98, In::F), (0x2D3090, In::I), (0x2D3098, In::I)],
                    writes: &[(0x31A1FC, Out::F), (0x31A200, Out::F), (0x31A204, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0]) },
                plain: |a| x87_3(a[0]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
