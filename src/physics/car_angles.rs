//! The car's heading, pitch and roll from its crash matrix (`physics::matrix_angles`): each angle
//! times 65536, 4096 and 1 / 2pi (`k65536`, `k4096`, `one_over_two_pi`), rounded to an integer
//! and shifted left 4, into `car_heading`, `car_pitch` and `car_roll`: whole-number angle units
//! (<< 4).
//!
//! Returns the crash matrix.

use crate::math::x87::mul3_to_int;
use crate::physics::car_store::Car;
use crate::physics::matrix_angles::matrix_angles;
use crate::race::owner_course::Course;

/// `matrix_angles` of the crash matrix, each angle scaled to the car's whole-number angle (pitch,
/// heading, roll).
pub fn car_angles(course: &Course, car: &mut Car) {
    matrix_angles(course, &car.crash_matrix);
    let scale = |a: u32| {
        mul3_to_int(
            a,
            course.numbers.k65536,
            course.numbers.k4096,
            course.numbers.one_over_two_pi,
        )
        .wrapping_shl(4)
    };
    let l = course.turn_work();
    car.car_pitch = scale(l.car_pitch.bits()) as u32;
    car.car_heading = scale(l.car_heading.bits()) as u32;
    car.car_roll = scale(l.car_roll.bits()) as u32;
}
