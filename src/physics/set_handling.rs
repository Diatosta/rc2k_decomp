//! After `physics::grade_or_reset`, nine handling figures from the car's setup (whole numbers in
//! the work copy, 0 .. 256 by the look of it): each lo + n / 256 (hi - lo) between two constants:
//! `PhysicsTables.handling[8]`, `handling[7]` (by `steering_sensitivity`), `handling[1]`
//! (`brake_balance`), `handling[2]`, `handling[4]` (`ride_height`; the second from the pair for
//! `drive_kind`, then times `ride_height_handling_scale`), `handling[0]`, `handling[3]`,
//! `handling[5]`, `handling[6]` (`suspension_stiffness`). What each is, is a reading.
//!
//! Refuses (before anything) a drive kind above 1 (the cars' records have 0 and 1; the pair tables
//! hold two).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87::int_lerp;
use crate::memory::session::PlainDword;
use crate::physics::car_store::Car;
use crate::physics::figures::{HANDLING_RANGES_A, HANDLING_RANGES_B, HANDLING_RANGES_C};
use crate::physics::grade_or_reset::grade_or_reset;
use crate::physics::store::DamageModel;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;

/// (the figure's place in `handling`, setting, hi, lo) for each figure, except the drive kind's.
fn before(car: &Car) -> [(usize, &PlainDword, f32, f32); 4] {
    [
        (
            8,
            &car.steering_sensitivity,
            HANDLING_RANGES_A[3],
            HANDLING_RANGES_A[2],
        ),
        (
            7,
            &car.steering_sensitivity,
            HANDLING_RANGES_A[5],
            HANDLING_RANGES_A[4],
        ),
        (
            1,
            &car.brake_balance,
            HANDLING_RANGES_A[1],
            HANDLING_RANGES_A[0],
        ),
        (
            2,
            &car.ride_height,
            HANDLING_RANGES_B[1],
            HANDLING_RANGES_B[0],
        ),
    ]
}
fn after(car: &Car) -> [(usize, &PlainDword, f32, f32); 4] {
    [
        (
            0,
            &car.suspension_stiffness,
            HANDLING_RANGES_C[7],
            HANDLING_RANGES_C[6],
        ),
        (
            3,
            &car.suspension_stiffness,
            HANDLING_RANGES_C[1],
            HANDLING_RANGES_C[0],
        ),
        (
            5,
            &car.suspension_stiffness,
            HANDLING_RANGES_C[3],
            HANDLING_RANGES_C[2],
        ),
        (
            6,
            &car.suspension_stiffness,
            HANDLING_RANGES_C[5],
            HANDLING_RANGES_C[4],
        ),
    ]
}

/// The exact version, for `race::settle_car` and `physics::car_physics_step`.
pub fn set_handling(
    damage: &mut DamageModel,
    numbers: &Numbers,
    shared: &RaceShared,
) -> Result<(), Refused> {
    let kind = shared.car().drive_kind.get() as usize;
    if kind > 1 {
        return Err(BadInput("a drive kind above 1 (the pair tables hold two)").into());
    }
    grade_or_reset(damage, numbers, shared);
    let k256 = numbers.k256.get();
    let lerp = |(out, n, hi, lo): (usize, &PlainDword, f32, f32)| {
        let figure = int_lerp(n.get() as i32, k256, hi, lo);
        shared.physics_tables_mut().handling[out] = figure;
    };
    for v in before(shared.car()) {
        lerp(v);
    }
    let r = &HANDLING_RANGES_B;
    lerp((4, &shared.car().ride_height, r[4 + kind], r[2 + kind]));
    let scaled = {
        let tables = shared.physics_tables();
        x87::mul(tables.handling[4], tables.ride_height_handling_scale)
    };
    shared.physics_tables_mut().handling[4] = scaled;
    for v in after(shared.car()) {
        lerp(v);
    }
    Ok(())
}
