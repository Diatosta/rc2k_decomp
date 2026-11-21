//! Whether the work car has passed the next split's point. False with `no_place` set. The car's x
//! and z go into `ground_x` and `ground_z`; then the split's checkpoint, its face (past
//! `course_face_count` it logs "[car0.currsegment]" .. and raises fatal error 0x80601843) and that
//! face's first point; the distance (the two gaps squared, added, square-rooted, / 65536) goes into
//! `split_point_distance`; within `split_distance`: the angle from the point to the car
//! (arctangent, / pi, x 0.5, x 65536, rounded, low 16 bits, shifted up 16) goes into `split_word`
//! and the checkpoint's angle into `split_heading`; true when the angle is not below the
//! checkpoint's less a quarter turn.
//!
//! The x87 runs (with the integer steps between) are plain Rust over `X` (`math/x87_float.rs`).
//!
//! Raises fatal error 0x80601843 as above. Refuses (before anything) a split past 0x40, a
//! checkpoint past 0x1418, no faces or points.

use crate::errors::{BadInput, FatalError, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::x87_float::int;
use crate::math::x87_float::to_int;
use crate::physics::car_store::Car;
use crate::physics::figures::SPLIT_DISTANCE;
use crate::physics::ground_query::GroundSearch;
use crate::physics::split_tables_store::SplitTables;
use crate::physics::step_families::CheckpointInputs;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use rc2k_core::physics::progress;
use std::cmp::Ordering;

/// The distance from `point`: the whole-number gaps (the search's `x` and `z` less the
/// point's x and z), their squares added, square-rooted,
/// x `one_over_65536`, into the split point distance.
fn distance(tables: &PhysicsTables, numbers: &Numbers, ground: &GroundSearch, point: [u32; 3]) {
    let word = |k: usize| point[k] as i32;
    let dx = word(0).wrapping_sub(ground.x as i32);
    let first = int(dx);
    let dz = word(2).wrapping_sub(ground.z as i32);
    let second = int(dz);
    tables
        .split_point_distance
        .st((first * first + second * second).sqrt() * numbers.one_over_65536.ld());
}

/// The angle from `point` to the car: the gaps (the car's x and z less the point's), the
/// arctangent, over pi, x `half` x `k65536`, rounded; its low 16 bits.
fn angle(numbers: &Numbers, ground: &GroundSearch, point: [u32; 3]) -> u32 {
    let word = |k: usize| point[k] as i32;
    let dx = (ground.x as i32).wrapping_sub(word(0));
    let y = int(dx);
    let dz = (ground.z as i32).wrapping_sub(word(2));
    let x = int(dz);
    let turns = y.atan2_over_pi(x) * numbers.half.ld() * numbers.k65536.ld();
    to_int(turns) as u32 & 0xFFFF
}

/// The split's checkpoint and its face's first point's number, refused or fatal as said.
fn point(
    split_tables: &SplitTables,
    course: &Course,
    checkpoints: &crate::race::checkpoints_store::Checkpoints,
    car: &Car,
) -> Result<(u32, u32), Refused> {
    let split = car.splits_passed.get();
    let cp = progress::check_split_point(split, || split_tables.checkpoints[1 + split as usize])?;
    let face = crate::race::checkpoints_store::face_run_on(checkpoints, cp as usize).get();
    if !progress::face_known(face, course.course.get().course_face_count.get()) {
        return Err(FatalError {
            code: progress::SEGMENT_FATAL,
            message: "\u{1}",
        }
        .into());
    }
    if !crate::race::course_places::held(&course.stage.course)
        || !crate::race::scene_points::held(&course.stage.course)
    {
        return Err(BadInput("no course faces or points").into());
    }
    let index = crate::race::course_places::face_field::<u32>(course.reads(), face, 4)
        .unwrap_or_else(|e| crate::fatal::raise::end("physics::passed_split_point's operand", e));
    Ok((cp, index))
}

/// Whether the car has passed the split's point.
// Core: `rc2k_core::physics::progress::{check_split_point, face_known, passed_point}` (the distance
// and the angle stay x87).
pub fn passed_split_point(c: &CheckpointInputs<'_>) -> Result<bool, Refused> {
    if c.no_place != 0 {
        return Ok(false);
    }
    let (cp, pt) = point(&c.physics.split_tables(), c.course, c.checkpoints, c.car)?;
    // The point read from `race::scene_points`; the x87 runs read a copy.
    let pt = crate::race::scene_points::point_number(
        c.course.reads(),
        &crate::race::scene_points::no_own(),
        pt,
    )?;
    {
        let mut search = c.physics.ground_search().borrow_mut();
        search.x = c.car.position[0].get();
        search.z = c.car.position[2].get();
    }
    distance(c.tables, c.numbers, &c.physics.ground_search().borrow(), pt);
    let within = !matches!(
        c.tables
            .split_point_distance
            .get()
            .partial_cmp(&SPLIT_DISTANCE),
        Some(Ordering::Greater | Ordering::Equal)
    );
    if !within {
        return Ok(false);
    }
    let car_angle = angle(c.numbers, &c.physics.ground_search().borrow(), pt) << 16;
    let heading = crate::race::checkpoints_store::angle_run_on(c.checkpoints, cp as usize).get();
    let (carry, _) = progress::passed_point(car_angle, heading);
    Ok(carry)
}

#[cfg(test)]
mod tests;
