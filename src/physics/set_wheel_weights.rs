//! The four wheels' load forces (four of the force records `physics::clear_forces` clears): a share
//! `wheel_share` held to 1 ..= `five` (through `scratch`), ((it - 1) `quarter` + 1) `gravity`
//! `weight`, negated, times `quarter` again goes into each record's force, turned into the car's
//! frame (`physics::turn_by_inverse`); then each record's second vector is set to (-1 / 1, 0, +-1)
//! by its corner and turned the same way. A reading: the weight on each wheel, the corners' places.

use crate::math::x87;
use crate::math::x87::{compare_full, weight_force};
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::turn_by_inverse::turn_by_inverse;
use crate::race::numbers_store::Numbers;
use std::cell::RefCell;

pub(super) fn set_wheel_weights(
    car: &Car,
    tables: &PhysicsTables,
    contact: &RefCell<ContactWork>,
    numbers: &Numbers,
) {
    let one = numbers.one.get();
    let high = numbers.five.get();
    let k = numbers.quarter.get();
    let mut share = x87::loaded(contact.borrow().wheel_share).to_bits();
    if compare_full(share, one) & 0x100 != 0 {
        share = x87::loaded(one).to_bits();
    }
    if compare_full(share, high) & 0x100 == 0 {
        share = x87::loaded(high).to_bits();
    }
    let f = weight_force(share, one, k, &tables.gravity, &tables.weight);
    let wheels = &tables.forces[..4];
    for w in wheels {
        w.force[1].set(f);
    }
    for w in wheels {
        turn_by_inverse(car, &w.force);
    }
    let plus = x87::loaded(1.0f32);
    let minus = x87::negated(plus);
    // The wheels' points: x and z of +1 or -1 (0x2C, 0x6C, 0x14, 0x34 plus; 0x0C, 0x4C,
    // 0x54, 0x74 minus).
    for cell in [
        &wheels[1].point[0],
        &wheels[3].point[0],
        &wheels[0].point[2],
        &wheels[1].point[2],
    ] {
        cell.set(plus);
    }
    for cell in [
        &wheels[0].point[0],
        &wheels[2].point[0],
        &wheels[2].point[2],
        &wheels[3].point[2],
    ] {
        cell.set(minus);
    }
    for w in wheels {
        turn_by_inverse(car, &w.point);
    }
}
