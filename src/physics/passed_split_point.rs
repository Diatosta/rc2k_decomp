//! `physics::passed_split_point` (0x44165E): whether the work car has passed the next
//! split's point. Nothing (CF clear) with `no_place` set. The car's x / z into
//! `ground_x` / `ground_z`; the split's checkpoint (`dword_601030[+0x90]`), its
//! face (`g_checkpointFaces`; past `g_courseFaceCount` the original logs
//! "[car0.currsegment]" .. and raises fatal error 0x80601843) and that face's first point;
//! the distance (`fild` of the two gaps, squared, added, `fsqrt`, / 65536) into
//! `split_point_distance`; within `split_distance`: the angle from the point to the car (`fpatan`, /
//! pi, x 0.5, x 65536, `fistp`, low 16 bits, shifted up 16) into `split_word`, the
//! checkpoint's (`dword_9ED808`) into `split_heading`; CF when the angle is not below the
//! checkpoint's less a quarter turn (by the sign).
//!
//! The x87 runs (with the integer steps between) are the arithmetic of the original's
//! instructions, in plain Rust over `X` (`math/x87_float.rs`); the instructions are
//! `asm_reference` in the tests.
//!
//! Caller: `physics::track_checkpoints`. Raises fatal error 0x80601843 where the original
//! does. Refuses (before anything) a split past 0x40, a checkpoint past 0x1418, no faces or
//! points.

use crate::errors::{BadInput, FatalError, Refused};
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::physics::car_store::Car;
use crate::physics::ground_store::GroundWork;
use crate::physics::split_tables_store::SplitTables;
use crate::physics::step_families::CheckpointInputs;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use rc2k_core::physics::progress;
use std::cmp::Ordering;

/// 0x4416EC .. 0x441727: the distance from `point`: the whole-number gaps
/// (`GROUND_WORK.ground_x` and `ground_z` less the point's x and z; the last one is left in
/// the scratch dword +0x22400C), their squares added, `fsqrt`, x `one_over_65536`, into
/// +0x214D04.
fn distance(
    tables: &PhysicsTables,
    numbers: &Numbers,
    ground: &GroundWork,
    point: [u32; 3],
    scene: &crate::scene::owner::Scene,
) {
    let drawing = &scene.drawing;
    let word = |k: usize| point[k] as i32;
    let dx = word(0).wrapping_sub(ground.ground_x.get() as i32);
    drawing.scene.scratch.set_bits(dx as u32);
    let first = int(drawing.scene.scratch.bits() as i32);
    let dz = word(2).wrapping_sub(ground.ground_z.get() as i32);
    drawing.scene.scratch.set_bits(dz as u32);
    let second = int(drawing.scene.scratch.bits() as i32);
    tables.split_point_distance.set_bits(st(
        (first * first + second * second).sqrt() * ld(numbers.one_over_65536.bits())
    ));
}

/// 0x441770 .. 0x4417AD: the angle from `point` to the car, into `scratch`: the gaps
/// (the car's x and z less the point's, through the scratch dword +0x22400C), `fpatan`,
/// over pi, x `half` x `k65536`, `fistp` into the scratch, its low 16 bits.
fn angle(
    numbers: &Numbers,
    ground: &GroundWork,
    point: [u32; 3],
    scene: &crate::scene::owner::Scene,
) {
    let drawing = &scene.drawing;
    let word = |k: usize| point[k] as i32;
    let dx = (ground.ground_x.get() as i32).wrapping_sub(word(0));
    drawing.scene.scratch.set_bits(dx as u32);
    let y = int(drawing.scene.scratch.bits() as i32);
    let dz = (ground.ground_z.get() as i32).wrapping_sub(word(2));
    drawing.scene.scratch.set_bits(dz as u32);
    let x = int(drawing.scene.scratch.bits() as i32);
    let turns = y.atan2_over_pi(x) * ld(numbers.half.bits()) * ld(numbers.k65536.bits());
    drawing.scene.scratch.set_bits(to_int(turns) as u32);
    drawing
        .scene
        .scratch
        .set_bits(drawing.scene.scratch.bits() & 0xFFFF);
}

/// The split's checkpoint and its face's first point's number, refused or fatal as said.
fn point(split_tables: &SplitTables, course: &Course, car: &Car) -> Result<(u32, u32), Refused> {
    let split = car.splits_passed.get();
    let cp =
        progress::check_split_point(split, || split_tables.checkpoints[1 + split as usize].get())?;
    let face = crate::race::checkpoints_store::face(cp as usize).get();
    if !progress::face_known(face, course.course.course_face_count.get()) {
        return Err(FatalError {
            code: progress::SEGMENT_FATAL,
            message: "\u{1}",
        }
        .into());
    }
    if course.course.course_faces.get() == 0 || course.course.scene_points.get() == 0 {
        return Err(BadInput("no course faces or points").into());
    }
    let index = crate::race::course_places::face_field::<u32>(course, face, 4)
        .unwrap_or_else(|e| crate::fatal::raise::end("physics::passed_split_point's operand", e));
    Ok((cp, index))
}

/// Whether the car has passed the split's point (CF).
// Converted: `rc2k_core::physics::progress::{check_split_point, face_known, passed_point}`
// (the distance and the angle stay x87).
pub fn passed_split_point(
    c: &CheckpointInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    if c.finish.no_place.get() != 0 {
        return Ok(false);
    }
    let (cp, pt) = point(&c.physics.split_tables, c.course, c.car)?;
    // The point read from `race::scene_points`; the x87 runs read a copy.
    let pt = crate::race::scene_points::point_number(c.course, pt)?;
    c.physics.ground.ground_x.set(c.car.position[0].get());
    c.physics.ground.ground_z.set(c.car.position[2].get());
    distance(c.tables, c.numbers, &c.physics.ground, pt, scene);
    let within = !matches!(
        f32::from_bits(c.tables.split_point_distance.bits())
            .partial_cmp(&f32::from_bits(c.tables.split_distance.bits())),
        Some(Ordering::Greater | Ordering::Equal)
    );
    if !within {
        return Ok(false);
    }
    angle(c.numbers, &c.physics.ground, pt, scene);
    let car_angle = scene.drawing.scene.scratch.get() << 16;
    c.tables.split_word.set(car_angle);
    let heading = crate::race::checkpoints_store::angle(cp as usize).get();
    c.tables.split_heading.set(heading);
    let (carry, _) = progress::passed_point(car_angle, heading);
    Ok(carry)
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;

    /// The point in a test's operand buffer.
    fn point_at(s: usize) -> [u32; 3] {
        crate::pointers::read_or_end(s as u32, "a split point")
    }
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
                plain: |a| {
                    distance(
                        &crate::race::owner::race().shared.physics_tables,
                        &crate::race::owner::race().course.numbers,
                        &crate::race::owner::race().physics.ground,
                        point_at(a[1]),
                        crate::scene::owner::scene(),
                    );
                },
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
                plain: |a| {
                    angle(
                        &crate::race::owner::race().course.numbers,
                        &crate::race::owner::race().physics.ground,
                        point_at(a[1]),
                        crate::scene::owner::scene(),
                    );
                },
            },
        ];
        sparse(&blocks, 6000, 202);
    }
}
