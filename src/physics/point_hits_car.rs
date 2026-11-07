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
//! `unk_71A20C` = 1, `unk_71A1CC` = 0x32.
//!
//! Takes: the point, the object
//! Out: CF set when inside; every register kept
//!
//! Callers: `physics::object_hits_car`, `physics::scene_point_hits_car` (both `jb`).

use crate::errors::Refused;
use crate::math::x87_float::X;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::physics::turn_by_heading::turn_by_heading;
use crate::physics::vector_length::vector_length;
use crate::race::owner::Race;
use crate::scene::matrix3::rounded;
use rc2k_core::physics::contacts;

/// The Rust version: whether the point `point` (its three whole numbers, read by the
/// caller) of the object of course face `face` (its number; its place at +0x30) is inside.
// Converted: `rc2k_core::physics::contacts::{point_part, inside_box, kind_hits}` (the x87
// steps stay).
pub fn point_hits_car(race: &Race, point: [u32; 3], face: u32) -> Result<bool, Refused> {
    let physics_store = &race.physics;
    let rd =
        |offset: u32, what: &str| crate::race::course_places::face_field::<u32>(face, offset, what);
    race.shared
        .physics_tables
        .point_hits
        .set(race.shared.physics_tables.point_hits.get().wrapping_add(1));
    let mut y = 0;
    for k in 0..3u32 {
        let v = contacts::point_part(
            point[k as usize],
            rd(0x30 + 4 * k, "an object's place")?,
            physics_store.contact.object_offset[k as usize].get(),
            race.shared.car.position[k as usize].get(),
        );
        physics_store.contact.object_point_offset[k as usize].set(v);
        if k == 1 {
            y = v;
        }
    }
    x87_1(race);
    for k in 0..3 {
        physics_store.contact.object_point_turned[k]
            .set_bits(physics_store.contact.object_point_scaled[k].bits());
    }
    turn_by_heading(&race.shared, &physics_store.contact.object_point_turned, y);
    x87_2(race);
    let (x, z) = (
        physics_store.contact.contact_point[0].bits(),
        physics_store.contact.contact_point[2].bits(),
    );
    if !contacts::inside_box(
        x,
        z,
        race.shared.physics_tables.spec_ints[2].get(),
        race.shared.physics_tables.spec_ints[0].get(),
        race.shared.physics_tables.spec_ints[1].get(),
    ) {
        return Ok(false);
    }
    if contacts::kind_hits(physics_store.contact.contact_kind.get()) {
        x87_3(race);
        physics_store
            .contact
            .contact_speed_length
            .set_bits(rounded(vector_length(&race.shared.car.velocity)));
        if let Some(hit) = rc2k_core::physics::walls::raised_hit(
            f32::from_bits(physics_store.contact.contact_speed_length.bits()),
            f32::from_bits(race.shared.car.contact_depth.bits()),
        ) {
            race.shared.car.contact_depth.set_bits(hit.to_bits());
        }
        race.shared.car.contact_flag.set(1);
        race.shared.car.wall_hit_timer.set(0x32);
    }
    Ok(true)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2D3B78] = int [rc + 0x2D3B4C] / [rc + 0x1C5D98]`
/// `[rc + 0x2D3B7C] = int [rc + 0x2D3B50] / [rc + 0x1C5D98]`
/// `[rc + 0x2D3B80] = int [rc + 0x2D3B54] / [rc + 0x1C5D98]`
fn x87_1(race: &Race) {
    let t0 = int(race.physics.contact.object_point_offset[0].bits() as i32);
    let t1 = ld(race.course.numbers.k65536.bits());
    let t2 = t0 / t1;
    race.physics.contact.object_point_scaled[0].set_bits(st(t2));
    let t3 = int(race.physics.contact.object_point_offset[1].bits() as i32);
    let t4 = ld(race.course.numbers.k65536.bits());
    let t5 = t3 / t4;
    race.physics.contact.object_point_scaled[1].set_bits(st(t5));
    let t6 = int(race.physics.contact.object_point_offset[2].bits() as i32);
    let t7 = ld(race.course.numbers.k65536.bits());
    let t8 = t6 / t7;
    race.physics.contact.object_point_scaled[2].set_bits(st(t8));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x2D3090] = [rc + 0x2D3B58] * [rc + 0x1C5D98]`
/// `int [rc + 0x2D3094] = [rc + 0x2D3B5C] * [rc + 0x1C5D98]`
/// `int [rc + 0x2D3098] = [rc + 0x2D3B60] * [rc + 0x1C5D98]`
fn x87_2(race: &Race) {
    let t0 = ld(race.physics.contact.object_point_turned[0].bits());
    let t1 = ld(race.course.numbers.k65536.bits());
    let t2 = t0 * t1;
    race.physics.contact.contact_point[0].set_bits(to_int(t2) as u32);
    let t3 = ld(race.physics.contact.object_point_turned[1].bits());
    let t4 = ld(race.course.numbers.k65536.bits());
    let t5 = t3 * t4;
    race.physics.contact.contact_point[1].set_bits(to_int(t5) as u32);
    let t6 = ld(race.physics.contact.object_point_turned[2].bits());
    let t7 = ld(race.course.numbers.k65536.bits());
    let t8 = t6 * t7;
    race.physics.contact.contact_point[2].set_bits(to_int(t8) as u32);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A1FC] = int [rc + 0x2D3090] / [rc + 0x1C5D98]`
/// `[rc + 0x31A200] = 0`
/// `[rc + 0x31A204] = int [rc + 0x2D3098] / [rc + 0x1C5D98]`
fn x87_3(race: &Race) {
    let shared = &race.shared;
    let t0 = int(race.physics.contact.contact_point[0].bits() as i32);
    let t1 = ld(race.course.numbers.k65536.bits());
    let t2 = t0 / t1;
    shared.car.contact_point[0].set_bits(st(t2));
    let t3 = X(0.0);
    shared.car.contact_point[1].set_bits(st(t3));
    let t4 = int(race.physics.contact.contact_point[2].bits() as i32);
    let t5 = ld(race.course.numbers.k65536.bits());
    let t6 = t4 / t5;
    shared.car.contact_point[2].set_bits(st(t6));
}

#[cfg(all(test, target_arch = "x86"))]
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
                operands: vec![Operand::RaceBlock {
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
                plain: |_| x87_1(crate::race::owner::race()),
            },
            Block {
                name: "point_hits_car line 63",
                operands: vec![Operand::RaceBlock {
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
                plain: |_| x87_2(crate::race::owner::race()),
            },
            Block {
                name: "point_hits_car line 80",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F), (0x2D3090, In::I), (0x2D3098, In::I)],
                    writes: &[(0x31A1FC, Out::F), (0x31A200, Out::F), (0x31A204, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0]) },
                plain: |_| x87_3(crate::race::owner::race()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
