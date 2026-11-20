//! Which side of the edge between two points (whole-number x and z) the ground point
//! (`ground_x`, `ground_z`) lies on: (X - x0)(z1 - z0) - (Z - z0)(x1 - x0) in 64 bits.

use crate::physics::ground_query::GroundEnv;
use rc2k_core::physics::faces;

/// The ground point's side of the edge from scene point `from` to `to` (their numbers): whether
/// the product is negative, or none when the edge gives no side.
pub fn edge_side(
    env: GroundEnv<'_>,
    point: (u32, u32),
    from: u32,
    to: u32,
) -> Result<Option<bool>, crate::errors::Refused> {
    let scene = |n: u32| -> Result<(u32, u32), crate::errors::BadInput> {
        Ok((
            crate::race::scene_points::point_word(env.course, env.points, n, 0)?,
            crate::race::scene_points::point_word(env.course, env.points, n, 2)?,
        ))
    };
    Ok(faces::edge_side(point, scene(from)?, scene(to)?).map(|(_, negative)| negative))
}
