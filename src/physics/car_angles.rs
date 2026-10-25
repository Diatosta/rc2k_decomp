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
use crate::physics::matrix_angles::{AnglePlaces, matrix_angles_at};
use crate::race::numbers_store::NUMBERS;

/// The Rust core: the matrix at `m`, `matrix_angles`' variables at `p`, the three ints at
/// `ints` (+4 from the first angle, +0 from the second, +8 from the third).
pub unsafe fn car_angles_at(m: u32, p: &AnglePlaces, ints: u32) {
    unsafe {
        matrix_angles_at(m, p);
        let scale = |a: u32| {
            mul3_to_int(
                a,
                NUMBERS.k65536.bits(),
                NUMBERS.k4096.bits(),
                NUMBERS.one_over_two_pi.bits(),
            )
            .wrapping_shl(4)
        };
        let angle = |k: u32| (p.angles as *const u32).add(k as usize).read_unaligned();
        for (k, place) in [(0u32, 4u32), (1, 0), (2, 8)] {
            let v = scale(angle(k));
            ((ints + place) as *mut i32).write_unaligned(v);
        }
    }
}

/// The Rust version.
pub unsafe fn car_angles() {
    let m = CAR.crash_matrix[0].addr() as u32;
    unsafe { car_angles_at(m, &AnglePlaces::game(), CAR.car_heading.addr() as u32) };
}
