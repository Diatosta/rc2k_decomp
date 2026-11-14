//! `math::rotate_float_point_about_y`: a point of three floats turned in x and z through
//! the scratch point by an angle. A sibling of `math::rotate_point_about_y` without the
//! whole numbers.

use crate::math::rotate_scratch_about_y::{
    Scratch, rotate_scratch_about_y, rotate_scratch_about_y_plain,
};
use crate::math::x87::Flt;

/// x', y, z' (the first) and the angle table's entry (the second).
pub fn rotate_float_point_about_y(
    tables: &crate::scene::engine_tables_store::EngineTables,
    x: impl Flt,
    y: impl Flt,
    z: impl Flt,
    angle: u32,
) -> ([f32; 3], u32) {
    turn_float_with(
        tables,
        [x.bits(), y.bits(), z.bits()],
        angle,
        rotate_scratch_about_y,
    )
}

/// The converted version, on `rotate_scratch_about_y_plain`.
pub fn rotate_float_point_about_y_plain(
    tables: &crate::scene::engine_tables_store::EngineTables,
    x: impl Flt,
    y: impl Flt,
    z: impl Flt,
    angle: u32,
) -> ([f32; 3], u32) {
    turn_float_with(
        tables,
        [x.bits(), y.bits(), z.bits()],
        angle,
        rotate_scratch_about_y_plain,
    )
}

/// The point (x, y, z, as they are held) through the scratch point.
fn turn_float_with(
    tables: &crate::scene::engine_tables_store::EngineTables,
    [x, y, z]: [u32; 3],
    angle: u32,
    turn: fn(&crate::scene::engine_tables_store::EngineTables, &mut Scratch, u32) -> u32,
) -> ([f32; 3], u32) {
    let mut point = Scratch {
        x: f32::from_bits(x),
        y: f32::from_bits(y),
        z: f32::from_bits(z),
    };
    let entry = { turn(tables, &mut point, angle) };
    ([point.x, point.y, point.z], entry)
}
