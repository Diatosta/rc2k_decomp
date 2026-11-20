//! `lever_pushes` and `PhysicsTables.lever_pushes[1]` = 0; then, only with `levers_on` 1 and
//! the wheels on the ground: the lever `lever_push_vector` and `lever_share_start` = 0; a share
//! A / (B + A) of A = |grip_forces[0]| + |grip_forces[3]| and B = |grip_forces[1]| +
//! |grip_forces[2]| (a reading: the wheels' loads) goes into `lever_share_start`, held to
//! `zero` (0) ..= `one` (1). If B + A is 0 (or no number) the share is A itself and it ends
//! there. Otherwise `PhysicsTables.lever_push_vector[2]` = `Car.probe_point_z[3]` and
//! `lever_share_left` is the share for `physics::lever_push`; then
//! `PhysicsTables.lever_push_vector[2]` = `probe_point_z` and `lever_share_left` 1 less it for
//! a second `lever_push`.

use crate::math::x87;
use crate::math::x87::{lever_share, one_less};
use crate::physics::car_store::Car;
use crate::physics::lever_push::lever_push;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::switches_store::FrameSwitches;
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
    switches: &FrameSwitches,
    physics_tables: &PhysicsTables,
    car: &Car,
) {
    physics_tables.lever_pushes[0].set(0);
    physics_tables.lever_pushes[1].set(0);
    if switches.levers_on.get() != 1 || car.all_wheels_off.get() != 0 {
        return;
    }
    physics_tables.lever_push_vector[0].set(0);
    physics_tables.lever_push_vector[1].set(0);
    physics_tables.lever_share_start.set(0.0);
    let (share, zero) = lever_share(
        &car.grip_forces[0],
        &car.grip_forces[3],
        &car.grip_forces[1],
        &car.grip_forces[2],
        0.0,
    );
    physics_tables.lever_share_start.set(share);
    if zero {
        return;
    }
    if below(physics_tables.lever_share_start.get(), 0.0) {
        physics_tables.lever_share_start.set(x87::loaded(0.0f32));
    }
    if !below(physics_tables.lever_share_start.get(), numbers.one.get()) {
        physics_tables
            .lever_share_start
            .set(x87::loaded(&numbers.one));
    }
    physics_tables.lever_push_vector[2].set(x87::loaded(&car.probe_point_z[3]).to_bits());
    physics_tables
        .lever_share_left
        .set(x87::loaded(&physics_tables.lever_share_start));
    lever_push(car, physics_tables, numbers);
    physics_tables.lever_push_vector[2].set(x87::loaded(&car.probe_point_z[0]).to_bits());
    physics_tables
        .lever_share_left
        .set(one_less(&physics_tables.lever_share_left));
    lever_push(car, physics_tables, numbers);
}
