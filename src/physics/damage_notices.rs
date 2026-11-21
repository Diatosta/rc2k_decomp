//! The work car's damage warnings. Each damage gauge (the floats from `brake_heat` to
//! `tyre_wear`) has a level (`damage_levels`) moved by thresholds, in the order of 82 steps
//! (`STEPS`). A step for level `from` puts the threshold (a whole number) into the threshold
//! scratch float and compares the gauge with it (below or unordered counts as below): rising
//! steps, when the gauge is not below, set the level to `to` and queue a notice
//! (`physics::queue_notice` with the notice, the gauge and the rank); falling steps (the
//! hysteresis of some gauges), when below, set it back.
//!
//! Refuses what `queue_notice` refuses (everything written is put back).

use crate::errors::Refused;
use crate::math::x87::from_int;
use crate::memory::session::Run;
use crate::physics::car_store::Car;
use crate::physics::queue_notice::queue_notice;
use crate::physics::store::DamageModel;
use rc2k_core::physics::damage_notices::{LEVELS, damage_notices as damage_notices_core};

/// `brake_heat` .. `tyre_wear`: the gauges, 47 dwords in a row in the car's record.
fn gauge(car: &Car, mut k: usize) -> u32 {
    let groups: [&dyn Run; 24] = [
        &car.brake_heat,
        &car.engine_heat,
        &car.engine_damage,
        &car.exhaust_damage,
        &car.turbo_damage,
        &car.cooling_damage,
        &car.clutch_damage,
        &car.differential_damage,
        &car.gear_wear,
        &car.gearbox_damage,
        &car.wheel_hit_damage,
        &car.body_hit_damage,
        &car.body_damage,
        &car.electrics_damage,
        &car.light_damage,
        &car.lights_damage,
        &car.steering_damage,
        &car.suspension_damage,
        &car.overall_suspension_damage,
        &car.brake_wear_a,
        &car.brake_wear_b,
        &car.brake_damage,
        &car.tyre_wears,
        &car.tyre_wear,
    ];
    for group in groups {
        if k < group.count() {
            return group.bits(k);
        }
        k -= group.count();
    }
    crate::fatal::raise::end(
        "physics::damage_notices",
        crate::errors::BadInput("a gauge past the car's gauges"),
    )
}

// Core: `rc2k_core::physics::damage_notices::damage_notices`.
fn steps(car: &Car, damage: &mut DamageModel) -> Result<(), Refused> {
    let mut levels: [u32; LEVELS] = std::array::from_fn(|k| car.damage_levels[k].get());
    let result = damage_notices_core(
        &mut levels,
        |k| f32::from_bits(gauge(car, k)),
        |t| {
            damage.scratch = from_int(t);
        },
        |q| queue_notice(car, q.notice, q.rank, gauge(car, q.gauge)),
    );
    for (k, &v) in levels.iter().enumerate() {
        car.damage_levels[k].set(v);
    }
    result
}

pub fn damage_notices(car: &Car, damage: &mut DamageModel) -> Result<(), Refused> {
    steps(car, damage)
}

#[cfg(test)]
mod gauge_tests {
    /// Each gauge is the dword its number says, from `brake_heat` on.
    #[test]
    fn the_gauges_are_in_a_row() {
        let shared_store = &crate::test_support::own::race().shared;

        let car = shared_store.car();
        let at = |cell: *const u8| crate::physics::car_offsets_1999::offset_1999(car, cell.addr());
        let first = at(car.brake_heat.cell_at(0).addr() as *const u8);
        assert_eq!(at((&raw const car.tyre_wear).cast()), first + 46 * 4);
    }
}
