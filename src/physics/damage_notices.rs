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
use crate::physics::queue_notice::queue_notice;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::session::Float;
use rc2k_core::physics::damage_notices::{LEVELS, damage_notices as damage_notices_core};

/// `unk_71A284` .. `unk_71A33C`: the gauges, 47 dwords in a row in the car's record.
fn gauge(shared: &Shared, mut k: usize) -> u32 {
    use std::slice::from_ref as one;
    let groups: [&[Float]; 24] = [
        &shared.car.brake_heat,
        one(&shared.car.engine_heat),
        one(&shared.car.engine_damage),
        one(&shared.car.exhaust_damage),
        one(&shared.car.turbo_damage),
        one(&shared.car.cooling_damage),
        one(&shared.car.clutch_damage),
        one(&shared.car.differential_damage),
        &shared.car.gear_wear,
        one(&shared.car.gearbox_damage),
        &shared.car.wheel_hit_damage,
        one(&shared.car.body_hit_damage),
        one(&shared.car.body_damage),
        one(&shared.car.electrics_damage),
        &shared.car.light_damage,
        one(&shared.car.lights_damage),
        one(&shared.car.steering_damage),
        &shared.car.suspension_damage,
        one(&shared.car.overall_suspension_damage),
        one(&shared.car.brake_wear_a),
        one(&shared.car.brake_wear_b),
        one(&shared.car.brake_damage),
        &shared.car.part_hit_damage,
        one(&shared.car.tyre_wear),
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
fn steps(race: &Race) -> Result<(), Refused> {
    let mut levels: [u32; LEVELS] = std::array::from_fn(|k| race.shared.car.damage_levels[k].get());
    let result = damage_notices_core(
        &mut levels,
        |k| f32::from_bits(gauge(&race.shared, k)),
        |t| {
            race.physics.damage.scratch.set_bits(from_int(t));
        },
        |q| queue_notice(&race.shared, q.notice, q.rank, gauge(&race.shared, q.gauge)),
    );
    for (k, &v) in levels.iter().enumerate() {
        race.shared.car.damage_levels[k].set(v);
    }
    result
}

/// The Rust version. The registers the original leaves are read by none of its callers.
pub fn damage_notices(race: &Race) -> Result<(), Refused> {
    steps(race)
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
