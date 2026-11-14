//! Turns the x and z floats of a point by minus the work car's heading (`car_heading`) with
//! `math::rotate_float_point_about_y` (y passed through) and stores them back.
//!
//! Caller: `physics::point_hits_car`.

use crate::math::rotate_float_point_about_y::rotate_float_point_about_y;
use crate::physics::car_store::Car;

/// `point` is the three floats (x, y, z); x and z are turned.
pub fn turn_by_heading(
    engine_tables: &crate::scene::engine_tables_store::EngineTables,
    car: &Car,
    point: &[impl crate::session::FloatLike; 3],
    ebx: u32,
) {
    let angle = car.car_heading.get().wrapping_neg();
    let ([x, _, z], _) =
        rotate_float_point_about_y(engine_tables, point[0].get(), ebx, point[2].get(), angle);
    point[0].set(x);
    point[2].set(z);
}
