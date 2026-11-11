//! `physics::damage_notices` (0x4B0241): the work car's damage warnings. For each damage
//! gauge (floats of the work copy, `unk_71A284` .. `unk_71A33C`) a level (`unk_71A398`,
//! `unk_71A340` ..) moved by thresholds, in the original's order of 82 steps (`STEPS`,
//! read from the disassembly): a step for level `from` puts the threshold (a whole number)
//! into `flt_75B36C` (`fild` / `fstp`, every step) and compares the gauge with it
//! (`fld; fcomp`, below or unordered as C0): rising steps, not below, set the level to
//! `to` and queue a notice (`physics::queue_notice`: esi the notice, eax the gauge, ebx
//! the rank); falling steps (the hysteresis of some gauges), below, set it back.
//!
//! Caller: `physics::car_physics_step` (0x43B95C), every physics step. Refuses what `queue_notice`
//! refuses (everything written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87::from_int;
use crate::physics::car_store::Car;
use crate::physics::queue_notice::queue_notice;
use crate::physics::store::DamageModel;
use crate::session::Float;
use rc2k_core::physics::damage_notices::{LEVELS, damage_notices as damage_notices_core};

/// `unk_71A284` .. `unk_71A33C`: the gauges, 47 dwords in a row in the car's record.
fn gauge(car: &Car, mut k: usize) -> u32 {
    use std::slice::from_ref as one;
    let groups: [&[Float]; 24] = [
        &car.brake_heat,
        one(&car.engine_heat),
        one(&car.engine_damage),
        one(&car.exhaust_damage),
        one(&car.turbo_damage),
        one(&car.cooling_damage),
        one(&car.clutch_damage),
        one(&car.differential_damage),
        &car.gear_wear,
        one(&car.gearbox_damage),
        &car.wheel_hit_damage,
        one(&car.body_hit_damage),
        one(&car.body_damage),
        one(&car.electrics_damage),
        &car.light_damage,
        one(&car.lights_damage),
        one(&car.steering_damage),
        &car.suspension_damage,
        one(&car.overall_suspension_damage),
        one(&car.brake_wear_a),
        one(&car.brake_wear_b),
        one(&car.brake_damage),
        &car.tyre_wears,
        one(&car.tyre_wear),
    ];
    for group in groups {
        match group.get(k) {
            Some(cell) => return cell.bits(),
            None => k -= group.len(),
        }
    }
    crate::fatal::raise::end(
        "physics::damage_notices",
        crate::errors::BadInput("a gauge past the car's gauges"),
    )
}

// Converted: `rc2k_core::physics::damage_notices::damage_notices`.
fn steps(car: &Car, damage: &DamageModel) -> Result<(), Refused> {
    let mut levels: [u32; LEVELS] = std::array::from_fn(|k| car.damage_levels[k].get());
    let result = damage_notices_core(
        &mut levels,
        |k| f32::from_bits(gauge(car, k)),
        |t| {
            damage.scratch.set_bits(from_int(t));
        },
        |q| queue_notice(car, q.notice, q.rank, gauge(car, q.gauge)),
    );
    for (k, &v) in levels.iter().enumerate() {
        car.damage_levels[k].set(v);
    }
    result
}

pub fn damage_notices(car: &Car, damage: &DamageModel) -> Result<(), Refused> {
    steps(car, damage)
}

#[cfg(test)]
mod gauge_tests {
    /// Each gauge is the dword its number says, from `brake_heat` on.
    #[test]
    fn the_gauges_are_in_a_row() {
        let shared_store = &crate::race::owner::race().shared;

        let first = (&raw const shared_store.car.brake_heat) as usize;
        assert_eq!(
            (&raw const shared_store.car.tyre_wear) as usize,
            first + 46 * 4
        );
    }
}
