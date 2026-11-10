//! `physics::set_handling` (0x43EB5D): `physics::grade_or_reset`, then nine handling
//! figures from the car's setup (whole numbers in the work copy, 0 .. 256 by the look of
//! it): each lo + n / 256 (hi - lo) between two constants of 0x613F1C .. 0x613F88:
//! `flt_614C0C`, `flt_614C08` (by `dword_71A014`), `flt_614BF0` (`dword_719FF8`),
//! `flt_614BF4`, `flt_614BFC` (`dword_71A008`; the second from the pair for
//! `g_carDriveKind`, then times `dword_614C18`), `flt_614BEC`, `flt_614BF8`, `flt_614C00`,
//! `flt_614C04` (`dword_71A00C`). What each is, is a reading.
//!
//! Returns: `g_carDriveKind`
//!
//! Callers: `physics::car_physics_step` (every physics step) and `race::settle_car`. Refuses
//! (before anything) a drive kind above 1 (the cars' records have 0 and 1; the pair tables hold
//! two).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_run::int_lerp;
use crate::physics::grade_or_reset::grade_or_reset;
use crate::race::owner::Race;
use crate::session::{Dword, Float};

/// (out, setting, hi, lo) in the original's order, but the drive kind's.
fn before(race: &Race) -> [(&Float, &Dword, &Float, &Float); 4] {
    [
        (
            &race.shared.physics_tables.handling[8],
            &race.shared.car.steering_sensitivity,
            &race.shared.physics_tables.handling_ranges_a[3],
            &race.shared.physics_tables.handling_ranges_a[2],
        ),
        (
            &race.shared.physics_tables.handling[7],
            &race.shared.car.steering_sensitivity,
            &race.shared.physics_tables.handling_ranges_a[5],
            &race.shared.physics_tables.handling_ranges_a[4],
        ),
        (
            &race.shared.physics_tables.handling[1],
            &race.shared.car.brake_balance,
            &race.shared.physics_tables.handling_ranges_a[1],
            &race.shared.physics_tables.handling_ranges_a[0],
        ),
        (
            &race.shared.physics_tables.handling[2],
            &race.shared.car.ride_height,
            &race.shared.physics_tables.handling_ranges_b[1],
            &race.shared.physics_tables.handling_ranges_b[0],
        ),
    ]
}
fn after(race: &Race) -> [(&Float, &Dword, &Float, &Float); 4] {
    [
        (
            &race.shared.physics_tables.handling[0],
            &race.shared.car.suspension_stiffness,
            &race.shared.physics_tables.handling_ranges_c[7],
            &race.shared.physics_tables.handling_ranges_c[6],
        ),
        (
            &race.shared.physics_tables.handling[3],
            &race.shared.car.suspension_stiffness,
            &race.shared.physics_tables.handling_ranges_c[1],
            &race.shared.physics_tables.handling_ranges_c[0],
        ),
        (
            &race.shared.physics_tables.handling[5],
            &race.shared.car.suspension_stiffness,
            &race.shared.physics_tables.handling_ranges_c[3],
            &race.shared.physics_tables.handling_ranges_c[2],
        ),
        (
            &race.shared.physics_tables.handling[6],
            &race.shared.car.suspension_stiffness,
            &race.shared.physics_tables.handling_ranges_c[5],
            &race.shared.physics_tables.handling_ranges_c[4],
        ),
    ]
}

/// The exact version, for `race::settle_car` and `physics::car_physics_step`.
pub fn set_handling(race: &Race) -> Result<(), Refused> {
    let kind = race.shared.car.drive_kind.get() as usize;
    if kind > 1 {
        return Err(BadInput("a drive kind above 1 (the pair tables hold two)").into());
    }
    grade_or_reset(race);
    let k256: u32 = race.course.numbers.k256.bits();
    let lerp = |(out, n, hi, lo): (&Float, &Dword, &Float, &Float)| {
        out.set_bits(int_lerp(n.get() as i32, k256, hi.bits(), lo.bits()));
    };
    for v in before(race) {
        lerp(v);
    }
    let r = &race.shared.physics_tables.handling_ranges_b;
    lerp((
        &race.shared.physics_tables.handling[4],
        &race.shared.car.ride_height,
        &r[4 + kind],
        &r[2 + kind],
    ));
    race.shared.physics_tables.handling[4].set_bits(x87::mul(
        race.shared.physics_tables.handling[4].bits(),
        race.shared.physics_tables.spec_a0.get(),
    ));
    for v in after(race) {
        lerp(v);
    }
    Ok(())
}
