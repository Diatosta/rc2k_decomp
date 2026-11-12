//! `physics::edge_side` (0x4425EF): which side of the edge between two points (whole-number x
//! at +0, z at +8) the ground point (`ground_x`, `ground_z`) lies on: (X - x0)(z1 - z0) -
//! (Z - z0)(x1 - x0) in 64 bits, the second product kept in `product`.
//!
//! Takes: the edge's ends
//!
//! Callers (8): `physics::face_under_point`, the face under a point (each edge of a triangle or a
//! quad).

use crate::physics::flag_result::FlagResult;
use crate::physics::ground_query::GroundQuery;
use crate::race::owner_course::CourseReads;
use rc2k_core::physics::faces;

/// The ground point's side of the edge from scene point `from` to `to` (their numbers).
pub fn edge_side(
    course: CourseReads<'_>,
    q: &GroundQuery<'_>,
    from: u32,
    to: u32,
) -> Result<FlagResult, crate::errors::Refused> {
    let result = |eax, carry, zero| FlagResult {
        eax,
        ecx: None,
        ebx: None,
        carry,
        zero: Some(zero),
    };
    let point = (q.x.get(), q.z.get());
    let scene = |n: u32| -> Result<(u32, u32), crate::errors::BadInput> {
        Ok((
            crate::race::scene_points::point_word(course, q.points, n, 0)?,
            crate::race::scene_points::point_word(course, q.points, n, 2)?,
        ))
    };
    Ok(match faces::edge_side(point, scene(from)?, scene(to)?) {
        None => result(1, false, false),
        Some((kept, negative)) => {
            q.product[0].set(kept as u32);
            q.product[1].set((kept >> 32) as u32);
            result(0, negative, true)
        }
    })
}
