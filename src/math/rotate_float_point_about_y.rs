//! `math::rotate_float_point_about_y` (0x41FE74): a point of three floats in eax, ebx, ecx (as
//! dwords) turned in x and z: stored in the scratch point (`g_scratchX`, `Y`, `Z`),
//! `math::rotate_scratch_about_y` by edx, and loaded back. A sibling of `math::rotate_point_about_y`
//! without the whole numbers.
//!
//! Takes: x, y, z (floats), the angle
//!
//! Callers: `physics::turn_by_heading`, `physics::wall_edge_contact`,
//! `physics::crossed_edge_contact`, `hud::turned_vertex`.

use crate::math::rotate_scratch_about_y::{
    Scratch, rotate_scratch_about_y, rotate_scratch_about_y_plain,
};

/// The Rust version: x', y, z' and the entry left in edx.
pub fn rotate_float_point_about_y(x: u32, y: u32, z: u32, angle: u32) -> [u32; 4] {
    turn_float_with(x, y, z, angle, rotate_scratch_about_y)
}

/// The converted version (`rotate_scratch_about_y_plain`).
pub fn rotate_float_point_about_y_plain(x: u32, y: u32, z: u32, angle: u32) -> [u32; 4] {
    turn_float_with(x, y, z, angle, rotate_scratch_about_y_plain)
}

fn turn_float_with(
    x: u32,
    y: u32,
    z: u32,
    angle: u32,
    turn: fn(&mut Scratch, u32) -> u32,
) -> [u32; 4] {
    let mut point = Scratch {
        x: f32::from_bits(x),
        y: f32::from_bits(y),
        z: f32::from_bits(z),
    };
    let entry = { turn(&mut point, angle) };
    [
        point.x.to_bits(),
        point.y.to_bits(),
        point.z.to_bits(),
        entry,
    ]
}
