//! `math::rotate_point_about_y` (0x41FD12): a point of whole numbers turned in x and z.
//!
//! Takes: x, y, z, the angle
//!
//! The three go through the scratch point (`FrameScratch.scratch_x` x, `FrameScratch.scratch_y` y,
//! `FrameScratch.scratch_z` z; a local since 2026-10-24: `rotate_scratch_about_y::Scratch`), left there as
//! whole numbers. 29 callers (the map, the cameras, ..).

use crate::math::rotate_scratch_about_y::{Scratch, rotate_scratch_about_y};

/// `math::rotate_point_about_y`: x', y (as it comes back through a float, left only in the
/// scratch point), z' and the entry left in edx.
pub fn rotate_point_about_y(
    tables: &crate::scene::engine_tables_store::EngineTables,
    x: i32,
    y: i32,
    z: i32,
    angle: u32,
) -> [u32; 4] {
    turn_point_with(tables, x, y, z, angle, rotate_scratch_about_y)
}

fn turn_point_with(
    tables: &crate::scene::engine_tables_store::EngineTables,
    x: i32,
    y: i32,
    z: i32,
    angle: u32,
    turn: fn(&crate::scene::engine_tables_store::EngineTables, &mut Scratch, u32) -> u32,
) -> [u32; 4] {
    // Converted: `rc2k_core::math::geometry` (`fild` / `fstp` is `as f32`, `fistp` below).
    let mut point = Scratch {
        x: x as f32,
        y: y as f32,
        z: z as f32,
    };
    let entry = turn(tables, &mut point, angle);
    let back = |v: f32| rc2k_core::math::geometry::fistp(v) as u32;
    [back(point.x), back(point.y), back(point.z), entry]
}
