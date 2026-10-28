//! `physics::passed_split_point` (0x44165E): whether the work car has passed the next
//! split's point. Nothing (CF clear) with `dword_64BB18` set. The car's x / z into
//! `dword_639824` / `dword_63982C`; the split's checkpoint (`dword_601030[+0x90]`), its
//! face (`g_checkpointFaces`; past `g_courseFaceCount` the original logs
//! "[car0.currsegment]" .. and raises fatal error 0x80601843) and that face's first point;
//! the distance (`fild` of the two gaps, squared, added, `fsqrt`, / 65536) into
//! `flt_614D04`; within `flt_614C70`: the angle from the point to the car (`fpatan`, /
//! pi, x 0.5, x 65536, `fistp`, low 16 bits, shifted up 16) into `dword_614CFC`, the
//! checkpoint's (`dword_9ED808`) into `dword_614D00`; CF when the angle is not below the
//! checkpoint's less a quarter turn (by the sign).
//!
//! The x87 runs (with the integer steps between) are the arithmetic of the original's
//! instructions, in plain Rust over `X` (`math/x87_float.rs`); the instructions are
//! `asm_reference` in the tests.
//!
//! In:  nothing
//! Out: CF; eax, ebx, ecx, esi as left; no other flag read (the caller `jnb`s)
//!
//! Caller: `physics::track_checkpoints`. Raises fatal error 0x80601843 where the original
//! does. Refuses (before anything) a split past 0x40, a checkpoint past 0x1418, no faces or
//! points.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::drive::finish_store::FINISH;
use crate::errors::{BadInput, FatalError, Refused};
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::physics::car_store::CAR;
use crate::physics::ground_store::GROUND_WORK;
use crate::physics::split_tables_store::SPLIT_TABLES;
use crate::physics::tables_store::PHYSICS;
use crate::race::course_store::COURSE;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use rc2k_core::physics::progress;
use std::cmp::Ordering;

/// 0x4416EC .. 0x441727: the distance to the point at `point`.
fn distance(point: [u32; 3]) {
    let at = crate::pointers::game_address(point.as_ptr());
    distance_at(crate::race::store::asm_base() as usize, at as usize);
}

/// The distance over the race core at `rc` and the point at `s`: the whole-number gaps
/// (+0x239824 and +0x23982C less the point's x and z; the last one is left in the scratch
/// dword +0x22400C), their squares added, `fsqrt`, x `flt_5C5DA0`, into +0x214D04.
fn distance_at(rc: usize, s: usize) {
    unsafe {
        let (rc, s) = (
            crate::pointers::host_at(rc),
            crate::pointers::host_at(s) as *const u8,
        );
        let word = |p: *const u8| p.cast::<i32>().read_unaligned();
        let dx = word(s).wrapping_sub(word(rc.add(0x239824)));
        DRAW.scratch.set_bits(dx as u32);
        let first = int(DRAW.scratch.bits() as i32);
        let dz = word(s.add(8)).wrapping_sub(word(rc.add(0x23982C)));
        DRAW.scratch.set_bits(dz as u32);
        let second = int(DRAW.scratch.bits() as i32);
        PHYSICS.split_point_distance.set_bits(st(
            (first * first + second * second).sqrt() * ld(NUMBERS.one_over_65536.bits())
        ));
    }
}

/// 0x441770 .. 0x4417AD: the angle from the point at `point` to the car, into
/// `dword_62400C` (`fistp`, then the low 16 bits).
fn angle(point: [u32; 3]) {
    let at = crate::pointers::game_address(point.as_ptr());
    angle_at(crate::race::store::asm_base() as usize, at as usize);
}

/// The angle over the race core at `rc` and the point at `s`: the gaps (the car's x and z less
/// the point's, through the scratch dword +0x22400C), `fpatan`, over pi, x `flt_5C5E40` x
/// `flt_5C5D98`, `fistp` into the scratch, its low 16 bits.
fn angle_at(rc: usize, s: usize) {
    unsafe {
        let (rc, s) = (
            crate::pointers::host_at(rc),
            crate::pointers::host_at(s) as *const u8,
        );
        let word = |p: *const u8| p.cast::<i32>().read_unaligned();
        let dx = word(rc.add(0x239824)).wrapping_sub(word(s));
        DRAW.scratch.set_bits(dx as u32);
        let y = int(DRAW.scratch.bits() as i32);
        let dz = word(rc.add(0x23982C)).wrapping_sub(word(s.add(8)));
        DRAW.scratch.set_bits(dz as u32);
        let x = int(DRAW.scratch.bits() as i32);
        let turns = y.atan2_over_pi(x) * ld(NUMBERS.half.bits()) * ld(NUMBERS.k65536.bits());
        DRAW.scratch.set_bits(to_int(turns) as u32);
        let low = rc.add(0x22400C).cast::<u32>();
        low.write_unaligned(low.read_unaligned() & 0xFFFF);
    }
}

/// The split's checkpoint, its face and the face's first point (esi), refused or fatal
/// as said.
fn point() -> Result<(u32, u32, u32), Refused> {
    let split = CAR.splits_passed.get();
    let cp =
        progress::check_split_point(split, || SPLIT_TABLES.checkpoints[1 + split as usize].get())?;
    let face = crate::race::checkpoints_store::face(cp as usize).get();
    if !progress::face_known(face, COURSE.course_face_count.get()) {
        return Err(FatalError {
            code: progress::SEGMENT_FATAL,
            message: "\u{1}",
        }
        .into());
    }
    if COURSE.course_faces.get() == 0 || COURSE.scene_points.get() == 0 {
        return Err(BadInput("no course faces or points").into());
    }
    let f = (face << 6).wrapping_add(COURSE.course_faces.get());
    let index = crate::race::course_places::read_or_end::<u32>(
        (f + 4) as u32,
        "physics::passed_split_point's operand",
    );
    Ok((
        cp,
        index,
        index
            .wrapping_mul(12)
            .wrapping_add(COURSE.scene_points.get()),
    ))
}

/// The Rust version: whether the car has passed the split's point (CF). The registers it
/// leaves are read by none of its callers.
// Converted: `rc2k_core::physics::progress::{check_split_point, face_known, passed_point}`
// (the distance and the angle stay x87).
pub fn passed_split_point() -> Result<bool, Refused> {
    if FINISH.no_place.get() != 0 {
        return Ok(false);
    }
    let (cp, _, pt) = point()?;
    // The point read from `race::scene_points` (phase 13); the x87 runs read a copy.
    let pt = crate::race::scene_points::point(pt)?;
    GROUND_WORK.ground_x.set(CAR.position[0].get());
    GROUND_WORK.ground_z.set(CAR.position[2].get());
    distance(pt);
    let within = !matches!(
        f32::from_bits(PHYSICS.split_point_distance.bits())
            .partial_cmp(&f32::from_bits(PHYSICS.split_distance.bits())),
        Some(Ordering::Greater | Ordering::Equal)
    );
    if !within {
        return Ok(false);
    }
    angle(pt);
    let eax = DRAW.scratch.get() << 16;
    PHYSICS.split_word.set(eax);
    let heading = crate::race::checkpoints_store::angle(cp as usize).get();
    PHYSICS.split_heading.set(heading);
    let (carry, _) = progress::passed_point(eax, heading);
    Ok(carry)
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference_eax, sparse};

    reference_eax!(distance_ref(rc, s)
        "mov eax, [{s}]",
        "sub eax, [{rc} + 0x239824]",
        "mov [{rc} + 0x22400C], eax",
        "fild dword ptr [{rc} + 0x22400C]",
        ".byte 0xD8, 0xC8",
        "mov eax, [{s} + 8]",
        "sub eax, [{rc} + 0x23982C]",
        "mov [{rc} + 0x22400C], eax",
        "fild dword ptr [{rc} + 0x22400C]",
        ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xFA",
        "fmul dword ptr [{rc} + 0x1C5DA0]",
        "fstp dword ptr [{rc} + 0x214D04]"
    );

    reference_eax!(angle_ref(rc, s)
        "mov eax, [{rc} + 0x239824]",
        "sub eax, [{s}]",
        "mov [{rc} + 0x22400C], eax",
        "fild dword ptr [{rc} + 0x22400C]",
        "mov eax, [{rc} + 0x23982C]",
        "sub eax, [{s} + 8]",
        "mov [{rc} + 0x22400C], eax",
        "fild dword ptr [{rc} + 0x22400C]",
        ".byte 0xD9, 0xF3",
        ".byte 0xD9, 0xEB",
        ".byte 0xDE, 0xF9",
        "fmul dword ptr [{rc} + 0x1C5E40]",
        "fmul dword ptr [{rc} + 0x1C5D98]",
        "fistp dword ptr [{rc} + 0x22400C]",
        "and dword ptr [{rc} + 0x22400C], 0xFFFF"
    );

    #[test]
    fn distance_and_angle_match_the_assembly() {
        let blocks = vec![
            Block {
                name: "passed_split_point distance",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x239824, In::I), (0x23982C, In::I), (0x1C5DA0, In::F)],
                        writes: &[(0x22400C, Out::I), (0x214D04, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::I), (8, In::I)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { distance_ref(a[0], a[1]) },
                plain: |a| distance_at(a[0], a[1]),
            },
            Block {
                name: "passed_split_point angle",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x239824, In::I),
                            (0x23982C, In::I),
                            (0x1C5E40, In::F),
                            (0x1C5D98, In::F),
                        ],
                        writes: &[(0x22400C, Out::I)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::I), (8, In::I)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { angle_ref(a[0], a[1]) },
                plain: |a| angle_at(a[0], a[1]),
            },
        ];
        sparse(&blocks, 6000, 202);
    }
}
