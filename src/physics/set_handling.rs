//! `physics::set_handling` (0x43EB5D): `physics::grade_or_reset`, then nine handling
//! figures from the car's setup (whole numbers in the work copy, 0 .. 256 by the look of
//! it): each lo + n / 256 (hi - lo) between two constants of 0x613F1C .. 0x613F88:
//! `flt_614C0C`, `flt_614C08` (by `steering_sensitivity`), `flt_614BF0` (`brake_balance`),
//! `flt_614BF4`, `flt_614BFC` (`ride_height`; the second from the pair for
//! `g_carDriveKind`, then times `ride_height_handling_scale`), `handling`, `flt_614BF8`, `flt_614C00`,
//! `flt_614C04` (`suspension_stiffness`). What each is, is a reading.
//!
//! Returns: `g_carDriveKind`
//!
//! Callers: `physics::car_physics_step` (every physics step) and `race::settle_car`. Refuses
//! (before anything) a drive kind above 1 (the cars' records have 0 and 1; the pair tables hold
//! two).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87::int_lerp;
use crate::physics::car_store::Car;
use crate::physics::grade_or_reset::grade_or_reset;
use crate::physics::tables_store::PhysicsTables;
use crate::race::owner::Race;
use crate::session::{Dword, Float};

/// (out, setting, hi, lo) in the original's order, but the drive kind's.
fn before<'a>(
    car: &'a Car,
    tables: &'a PhysicsTables,
) -> [(&'a Float, &'a Dword, &'a Float, &'a Float); 4] {
    [
        (
            &tables.handling[8],
            &car.steering_sensitivity,
            &tables.handling_ranges_a[3],
            &tables.handling_ranges_a[2],
        ),
        (
            &tables.handling[7],
            &car.steering_sensitivity,
            &tables.handling_ranges_a[5],
            &tables.handling_ranges_a[4],
        ),
        (
            &tables.handling[1],
            &car.brake_balance,
            &tables.handling_ranges_a[1],
            &tables.handling_ranges_a[0],
        ),
        (
            &tables.handling[2],
            &car.ride_height,
            &tables.handling_ranges_b[1],
            &tables.handling_ranges_b[0],
        ),
    ]
}
fn after<'a>(
    car: &'a Car,
    tables: &'a PhysicsTables,
) -> [(&'a Float, &'a Dword, &'a Float, &'a Float); 4] {
    [
        (
            &tables.handling[0],
            &car.suspension_stiffness,
            &tables.handling_ranges_c[7],
            &tables.handling_ranges_c[6],
        ),
        (
            &tables.handling[3],
            &car.suspension_stiffness,
            &tables.handling_ranges_c[1],
            &tables.handling_ranges_c[0],
        ),
        (
            &tables.handling[5],
            &car.suspension_stiffness,
            &tables.handling_ranges_c[3],
            &tables.handling_ranges_c[2],
        ),
        (
            &tables.handling[6],
            &car.suspension_stiffness,
            &tables.handling_ranges_c[5],
            &tables.handling_ranges_c[4],
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
    for v in before(&race.shared.car, &race.shared.physics_tables) {
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
        &race.shared.physics_tables.handling[4],
        race.shared.physics_tables.ride_height_handling_scale.get(),
    ));
    for v in after(&race.shared.car, &race.shared.physics_tables) {
        lerp(v);
    }
    Ok(())
}
