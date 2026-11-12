//! `physics::set_handling` (0x43EB5D): `physics::grade_or_reset`, then nine handling
//! figures from the car's setup (whole numbers in the work copy, 0 .. 256 by the look of
//! it): each lo + n / 256 (hi - lo) between two constants of 0x613F1C .. 0x613F88:
//! `PhysicsTables.handling[8]`, `PhysicsTables.handling[7]` (by `steering_sensitivity`), `PhysicsTables.handling[1]` (`brake_balance`),
//! `PhysicsTables.handling[2]`, `PhysicsTables.handling[4]` (`ride_height`; the second from the pair for
//! `g_carDriveKind`, then times `ride_height_handling_scale`), `handling`, `PhysicsTables.handling[3]`, `PhysicsTables.handling[5]`,
//! `PhysicsTables.handling[6]` (`suspension_stiffness`). What each is, is a reading.
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
use crate::physics::store::DamageModel;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::session::{Float, PlainDword};

/// (out, setting, hi, lo) in the original's order, but the drive kind's.
fn before<'a>(
    car: &'a Car,
    tables: &'a PhysicsTables,
) -> [(&'a Float, &'a PlainDword, &'a Float, &'a Float); 4] {
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
) -> [(&'a Float, &'a PlainDword, &'a Float, &'a Float); 4] {
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
pub fn set_handling(
    damage: &DamageModel,
    numbers: &Numbers,
    shared: &RaceShared,
) -> Result<(), Refused> {
    let kind = shared.car().drive_kind.get() as usize;
    if kind > 1 {
        return Err(BadInput("a drive kind above 1 (the pair tables hold two)").into());
    }
    grade_or_reset(damage, numbers, shared);
    let k256 = numbers.k256.get();
    let lerp = |(out, n, hi, lo): (&Float, &PlainDword, &Float, &Float)| {
        out.set(int_lerp(n.get() as i32, k256, hi, lo));
    };
    for v in before(shared.car(), &shared.physics_tables) {
        lerp(v);
    }
    let r = &shared.physics_tables.handling_ranges_b;
    lerp((
        &shared.physics_tables.handling[4],
        &shared.car().ride_height,
        &r[4 + kind],
        &r[2 + kind],
    ));
    shared.physics_tables.handling[4].set(x87::mul(
        &shared.physics_tables.handling[4],
        shared.physics_tables.ride_height_handling_scale.get(),
    ));
    for v in after(shared.car(), &shared.physics_tables) {
        lerp(v);
    }
    Ok(())
}
