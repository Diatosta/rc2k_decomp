//! `physics::damage_notices` (0x4B0241): the work car's damage warnings. For each damage
//! gauge (floats of the work copy, `unk_71A284` .. `unk_71A33C`) a level (`unk_71A398`,
//! `unk_71A340` ..) moved by thresholds, in the original's order of 82 steps (`STEPS`,
//! read from the disassembly): a step for level `from` puts the threshold (a whole number)
//! into `flt_75B36C` (`fild` / `fstp`, every step) and compares the gauge with it
//! (`fld; fcomp`, below or unordered as C0): rising steps, not below, set the level to
//! `to` and queue a notice (`physics::queue_notice`: esi the notice, eax the gauge, ebx
//! the rank); falling steps (the hysteresis of some gauges), below, set it back.
//!
//! In:  nothing
//! Out: eax, ebx, esi the last notice's, edx as `queue_notice` leaves it (kept when none);
//!      the rest kept; no flags read (the caller goes on)
//!
//! Caller: `sub_43B635` (0x43B95C), every physics step. Refuses what `queue_notice` refuses
//! (everything written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87::from_int;
use crate::physics::queue_notice::queue_notice;
use crate::race::owner_physics::Physics;
use crate::session::Float;
use rc2k_core::physics::damage_notices::{LEVELS, damage_notices as damage_notices_core};

/// `unk_71A284` .. `unk_71A33C`: the gauges, 47 dwords in a row in the car's record.
fn gauge(mut k: usize) -> u32 {
    use std::slice::from_ref as one;
    let groups: [&[Float]; 24] = [
        &crate::race::owner::race().shared.car.brake_heat,
        one(&crate::race::owner::race().shared.car.engine_heat),
        one(&crate::race::owner::race().shared.car.engine_damage),
        one(&crate::race::owner::race().shared.car.exhaust_damage),
        one(&crate::race::owner::race().shared.car.turbo_damage),
        one(&crate::race::owner::race().shared.car.cooling_damage),
        one(&crate::race::owner::race().shared.car.clutch_damage),
        one(&crate::race::owner::race().shared.car.differential_damage),
        &crate::race::owner::race().shared.car.gear_wear,
        one(&crate::race::owner::race().shared.car.gearbox_damage),
        &crate::race::owner::race().shared.car.wheel_hit_damage,
        one(&crate::race::owner::race().shared.car.body_hit_damage),
        one(&crate::race::owner::race().shared.car.body_damage),
        one(&crate::race::owner::race().shared.car.electrics_damage),
        &crate::race::owner::race().shared.car.light_damage,
        one(&crate::race::owner::race().shared.car.lights_damage),
        one(&crate::race::owner::race().shared.car.steering_damage),
        &crate::race::owner::race().shared.car.suspension_damage,
        one(&crate::race::owner::race()
            .shared
            .car
            .overall_suspension_damage),
        one(&crate::race::owner::race().shared.car.brake_wear_a),
        one(&crate::race::owner::race().shared.car.brake_wear_b),
        one(&crate::race::owner::race().shared.car.brake_damage),
        &crate::race::owner::race().shared.car.part_hit_damage,
        one(&crate::race::owner::race().shared.car.tyre_wear),
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
fn steps(physics: &Physics) -> Result<(), Refused> {
    let mut levels: [u32; LEVELS] =
        std::array::from_fn(|k| crate::race::owner::race().shared.car.damage_levels[k].get());
    let result = damage_notices_core(
        &mut levels,
        |k| f32::from_bits(gauge(k)),
        |t| {
            physics.damage.scratch.set_bits(from_int(t));
        },
        |q| queue_notice(q.notice, q.rank, gauge(q.gauge)),
    );
    for (k, &v) in levels.iter().enumerate() {
        crate::race::owner::race().shared.car.damage_levels[k].set(v);
    }
    result
}

/// The Rust version. The registers the original leaves are read by none of its callers.
pub fn damage_notices(physics: &Physics) -> Result<(), Refused> {
    steps(physics)
}

#[cfg(test)]
mod gauge_tests {
    /// Each gauge is the dword its number says, from `brake_heat` on.
    #[test]
    fn the_gauges_are_in_a_row() {
        let first = (&raw const crate::race::owner::race().shared.car.brake_heat) as usize;
        assert_eq!(
            (&raw const crate::race::owner::race().shared.car.tyre_wear) as usize,
            first + 46 * 4
        );
    }
}
