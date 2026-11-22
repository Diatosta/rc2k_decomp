//! Only with `levers_on` 1 and the wheels on the ground: a share A / (B + A) of
//! A = |grip_forces[0]| + |grip_forces[3]| and B = |grip_forces[1]| + |grip_forces[2]| (a reading:
//! the wheels' loads) is held to `zero` (0) ..= `one` (1). If B + A is 0 (or no number) the
//! share is A itself and it ends there. Otherwise the lever at `probe_point_z[3]` pushes the car
//! by that share (`physics::lever_push`), then the lever at `probe_point_z[0]` by 1 less it.

use crate::math::x87;
use crate::math::x87::{lever_share, one_less};
use crate::physics::car_store::Car;
use crate::physics::lever_push::lever_push;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use std::cmp::Ordering;

/// Whether `share` is below `limit`, or unordered.
fn below(share: f32, limit: f32) -> bool {
    !matches!(
        share.partial_cmp(&limit),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

pub fn lever_pushes(
    numbers: &Numbers,
    switches: crate::race::switches_store::FrameSwitchesRef<'_>,
    physics_tables: &PhysicsTables,
    car: &mut Car,
) {
    if switches.levers_on() != 1 || car.all_wheels_off != 0 {
        return;
    }
    let (share, zero) = lever_share(
        car.grip_forces[0],
        car.grip_forces[3],
        car.grip_forces[1],
        car.grip_forces[2],
        0.0,
    );
    if zero {
        return;
    }
    let mut share_start = share;
    if below(share_start, 0.0) {
        share_start = x87::loaded(0.0f32);
    }
    if !below(share_start, numbers.one.get()) {
        share_start = x87::loaded(numbers.one);
    }
    let share_left = x87::loaded(share_start);
    let rear = x87::loaded(car.probe_point_z[3]).to_bits();
    lever_push(car, physics_tables, numbers, [0, 0, rear], share_left);
    let front = x87::loaded(car.probe_point_z[0]).to_bits();
    lever_push(
        car,
        physics_tables,
        numbers,
        [0, 0, front],
        one_less(share_left),
    );
}
