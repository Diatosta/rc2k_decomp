//! `physics::edge_side` (0x4425EF): which side of the edge from the point at ebp to the
//! point at edi (whole-number x at +0, z at +8) the point (`dword_639824`,
//! `dword_63982C`) lies: (X - x0)(z1 - z0) - (Z - z0)(x1 - x0) in 64 bits (`imul`, then
//! `sub`/`sbb`), the second product kept in `qword_620460`.
//!
//! Takes: the edge's ends
//!
//! Callers (8): `physics::face_under_point`, the face under a point (each edge of a triangle or a
//! quad).

use crate::physics::flag_result::FlagResult;
use crate::physics::ground_query::GroundQuery;
use rc2k_core::physics::faces;

/// The Rust version: the ground point's side of the edge from scene point `from` to `to`
/// (their numbers).
pub fn edge_side(
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
            crate::race::scene_points::point_word(n, 0)?,
            crate::race::scene_points::point_word(n, 2)?,
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
