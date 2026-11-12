//! `physics::lever_pushes` (0x43DDDF): `lever_pushes` and `PhysicsTables.lever_pushes[1]` 0; then, only
//! with `levers_on` 1 and `all_wheels_off` 0: the lever `lever_push_vector`, `B48` 0 and
//! `lever_share_start` 0; a share A / (B + A) of A = |+0x210| + |+0x21C| and B = |+0x214| +
//! |+0x218| (`g_car`, `grip_forces` .. `Car.grip_forces[3]`; a reading: the wheels' loads)
//! into `lever_share_start`, held to `zero` (0) ..= `one` (1). If B + A is 0 (or no
//! number) the share is A itself and it ends there. Else `PhysicsTables.lever_push_vector[2]` = `Car.probe_point_z[3]` and
//! `lever_share_left` the share, `physics::lever_push`; `PhysicsTables.lever_push_vector[2]` = `probe_point_z` and
//! `lever_share_left` 1 less it, `physics::lever_push`. It ends through the `retn` at 0x43DFB0.
//!
//! Callers: `physics::car_physics_step` (0x43B831, 0x43B8AA), every physics step.

use crate::math::x87;
use crate::math::x87::{lever_share, one_less};
use crate::physics::car_store::Car;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::lever_push::lever_push;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::switches_store::FrameSwitches;
use std::cmp::Ordering;

/// `fld limit; fld share; fcompp` and its C0: share below limit, or unordered.
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
    scene: &crate::scene::owner::Scene,
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
        ZERO,
    );
    physics_tables.lever_share_start.set(f32::from_bits(share));
    if zero {
        return;
    }
    if below(physics_tables.lever_share_start.get(), f32::from_bits(ZERO)) {
        physics_tables.lever_share_start.set(x87::loaded(ZERO));
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
    lever_push(car, physics_tables, numbers, scene);
    physics_tables.lever_push_vector[2].set(x87::loaded(&car.probe_point_z[0]).to_bits());
    physics_tables
        .lever_share_left
        .set(one_less(&physics_tables.lever_share_left));
    lever_push(car, physics_tables, numbers, scene);
}
