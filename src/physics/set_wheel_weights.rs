//! `physics::set_wheel_weights` (0x43F3E7): the four wheels' load forces (the records at
//! 0x614770 + 0x20 k, `physics::clear_forces`'): a share `wheel_share` held to 1 ..
//! `five` (through `scratch`), ((it - 1) `quarter` + 1) gravity
//! (`gravity`) `weight`, negated, times `quarter` again into each record's +4,
//! turned into the car's frame (`physics::turn_by_inverse`); then each record's second
//! vector (+0xC ..) set to (-1 / 1, 0, ±1) by its corner and turned the same way. A
//! reading: the weight on each wheel, the corners' places.
//!
//! Caller: `physics::car_physics_step` (0x43B723), every physics step.

use crate::math::x87;
use crate::math::x87::{compare_full, weight_force};
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::turn_by_inverse::turn_by_inverse;
use crate::race::numbers_store::Numbers;

pub fn set_wheel_weights(
    car: &Car,
    tables: &PhysicsTables,
    contact: &ContactWork,
    numbers: &Numbers,
    scene: &crate::scene::owner::Scene,
) {
    let scratch = &scene.drawing.scene.scratch;
    let one: u32 = numbers.one.bits();
    let high: u32 = numbers.five.bits();
    let k: u32 = numbers.quarter.bits();
    scratch.set(x87::loaded(&contact.wheel_share));
    if compare_full(scratch.get(), one) & 0x100 != 0 {
        scratch.set(x87::loaded(one));
    }
    if compare_full(scratch.get(), high) & 0x100 == 0 {
        scratch.set(x87::loaded(high));
    }
    let f = weight_force(
        scratch.get(),
        one,
        k,
        tables.gravity.bits(),
        tables.weight.bits(),
    );
    let wheels = &tables.forces[..4];
    for w in wheels {
        w.force[1].set_bits(f);
    }
    for w in wheels {
        turn_by_inverse(car, &w.force);
    }
    let plus = x87::loaded(1.0f32.to_bits());
    let minus = x87::negated(plus);
    // The wheels' points: x and z of +1 or -1 (0x2C, 0x6C, 0x14, 0x34 plus; 0x0C, 0x4C,
    // 0x54, 0x74 minus).
    for cell in [
        &wheels[1].point[0],
        &wheels[3].point[0],
        &wheels[0].point[2],
        &wheels[1].point[2],
    ] {
        cell.set_bits(plus);
    }
    for cell in [
        &wheels[0].point[0],
        &wheels[2].point[0],
        &wheels[2].point[2],
        &wheels[3].point[2],
    ] {
        cell.set_bits(minus);
    }
    for w in wheels {
        turn_by_inverse(car, &w.point);
    }
}
