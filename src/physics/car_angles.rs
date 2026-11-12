//! `physics::car_angles` (0x43BEF4): `physics::matrix_angles` of `g_crashMatrix`, then
//! each angle times 65536 (`k65536`), 4096 (`k4096`) and 1 / 2pi (`one_over_two_pi`),
//! `fistp`, shifted left 4: `car_pitch` into `car_pitch`, `D8` into `car_heading`,
//! `DC` into `car_roll` (`g_car` +0x9C4, +0x9C0, +0x9C8): the car's heading, pitch and
//! roll (`car_heading`, `car_pitch`, `car_roll`) in the game's whole-number units (<< 4).
//!
//! Returns: `g_crashMatrix`
//!
//! Caller: `physics::update_car_angles`.

use crate::math::x87::mul3_to_int;
use crate::physics::car_store::Car;
use crate::physics::matrix_angles::matrix_angles;
use crate::race::owner_course::Course;

/// `matrix_angles` of the crash matrix, each angle scaled to the car's whole-number angle (pitch,
/// heading, roll).
pub fn car_angles(course: &Course, car: &Car, scene: &crate::scene::owner::Scene) {
    matrix_angles(course, &car.crash_matrix, scene);
    let scale = |a: u32| {
        mul3_to_int(
            a,
            &course.numbers.get().k65536,
            &course.numbers.get().k4096,
            &course.numbers.get().one_over_two_pi,
        )
        .wrapping_shl(4)
    };
    let l = course.lists.get();
    car.car_pitch.set(scale(l.car_pitch.bits()) as u32);
    car.car_heading.set(scale(l.car_heading.bits()) as u32);
    car.car_roll.set(scale(l.car_roll.bits()) as u32);
}
