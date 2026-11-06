//! `physics::car_angles` (0x43BEF4): `physics::matrix_angles` of `g_crashMatrix`, then
//! each angle times 65536 (`flt_5C5D98`), 4096 (`flt_5C5E28`) and 1 / 2pi (`flt_5C5DD4`),
//! `fistp`, shifted left 4: `flt_5C70D4` into `dword_71A99C`, `D8` into `dword_71A998`,
//! `DC` into `dword_71A9A0` (`g_car` +0x9C4, +0x9C0, +0x9C8): the car's angles in the
//! game's whole-number units (a reading).
//!
//! In:  nothing
//! Out: esi = `g_crashMatrix`; the rest kept; no flags read
//!
//! Caller: `physics::update_car_angles`.

use crate::math::x87_run::mul3_to_int;
use crate::physics::car_store::CAR;
use crate::physics::matrix_angles::matrix_angles;
use crate::race::owner::race;

/// The Rust version: `matrix_angles` of the crash matrix, each angle scaled to the car's
/// whole-number angle (pitch, heading, roll).
pub fn car_angles() {
    let course = &race().course;
    matrix_angles(&CAR.crash_matrix);
    let scale = |a: u32| {
        mul3_to_int(
            a,
            course.numbers.k65536.bits(),
            course.numbers.k4096.bits(),
            course.numbers.one_over_two_pi.bits(),
        )
        .wrapping_shl(4)
    };
    let l = &course.lists;
    CAR.car_pitch.set_bits(scale(l.car_pitch.bits()) as u32);
    CAR.car_heading.set_bits(scale(l.car_heading.bits()) as u32);
    CAR.car_roll.set_bits(scale(l.car_roll.bits()) as u32);
}
