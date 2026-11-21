//! Turns the x and z floats of a point by minus the work car's heading (`car_heading`) with
//! `math::rotate_float_point_about_y` (y passed through) and stores them back.

use crate::math::rotate_float_point_about_y::rotate_float_point_about_y;
use crate::physics::car_store::Car;

/// `point` is the three floats' bits (x, y, z); x and z are turned, y stays. `ebx` is the whole
/// number the rotation takes in y's place (the original's register).
pub(super) fn turn_by_heading(
    engine_tables: &crate::math::engine_tables::EngineTables,
    car: &Car,
    point: [u32; 3],
    ebx: u32,
) -> [u32; 3] {
    let angle = car.car_heading.get().wrapping_neg();
    let ([x, _, z], _) = rotate_float_point_about_y(
        engine_tables,
        f32::from_bits(point[0]),
        ebx,
        f32::from_bits(point[2]),
        angle,
    );
    [x.to_bits(), point[1], z.to_bits()]
}
