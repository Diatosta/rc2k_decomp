//! Moves the work car by a point's speed about its spin: the spin is turned into the car's frame,
//! crossed with the lever, turned back by the car's matrix, scaled by the share `share_left` and
//! by `lever_share`; its x and z times 65536 and the step (`step_time`), rounded, are added to the
//! car's x and z.

use crate::math::matrix3::extended;
use crate::math::x87::mul2_to_int;
use crate::memory::plain_floats::FloatRow;
use crate::memory::plain_floats::FloatRowMut;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::vector_bits::{bits_of, cross, scale, turn};
use crate::race::numbers_store::Numbers;

/// One lever push: `lever` is the lever's vector (x, y and z as float bits), `share_left` the
/// share of the push this lever takes.
pub(super) fn lever_push(
    car: &mut Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    lever: [u32; 3],
    share_left: f32,
) {
    let place = turn(&car.car_inverse_matrix, bits_of(&car.spin));
    let mut turned = turn(&car.crash_matrix, cross(lever, place));
    for factor in [share_left, tables.lever_share] {
        turned = scale(turned, extended(factor));
    }
    for (from, axis) in [(turned[0], 0), (turned[2], 2)] {
        let push = mul2_to_int(from, numbers.k65536, numbers.step_time.get()) as u32;
        car.position
            .set_bits_at(axis, car.position.bits_at(axis).wrapping_add(push));
    }
}
