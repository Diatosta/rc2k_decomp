//! `physics::car_angles` (0x43BEF4): `physics::matrix_angles` of `g_crashMatrix`, then
//! each angle times 65536 (`flt_5C5D98`), 4096 (`flt_5C5E28`) and 1 / 2pi (`flt_5C5DD4`),
//! `fistp`, shifted left 4: `flt_5C70D4` into `dword_71A99C`, `D8` into `dword_71A998`,
//! `DC` into `dword_71A9A0` (`g_car` +0x9C4, +0x9C0, +0x9C8): the car's angles in the
//! game's whole-number units (a reading).
//!
//! Returns: `g_crashMatrix`
//!
//! Caller: `physics::update_car_angles`.

use crate::math::x87_run::mul3_to_int;
use crate::physics::matrix_angles::matrix_angles;
use crate::race::owner::Race;

/// `matrix_angles` of the crash matrix, each angle scaled to the car's whole-number angle (pitch,
/// heading, roll).
pub fn car_angles(race: &Race, scene: &crate::scene::owner::Scene) {
    matrix_angles(&race.course, &race.shared.car.crash_matrix, scene);
    let scale = |a: u32| {
        mul3_to_int(
            a,
            race.course.numbers.k65536.bits(),
            race.course.numbers.k4096.bits(),
            race.course.numbers.one_over_two_pi.bits(),
        )
        .wrapping_shl(4)
    };
    let l = &race.course.lists;
    race.shared
        .car
        .car_pitch
        .set_bits(scale(l.car_pitch.bits()) as u32);
    race.shared
        .car
        .car_heading
        .set_bits(scale(l.car_heading.bits()) as u32);
    race.shared
        .car
        .car_roll
        .set_bits(scale(l.car_roll.bits()) as u32);
}
