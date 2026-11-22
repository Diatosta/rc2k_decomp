//! Resets a car block (0xFE0 bytes, the work copy's layout): `damage_failed` and the run of fields
//! after it are zeroed; `engine_power_factor` and the run after it = 1.0; the wheel stiffness
//! fields = `suspension_stiffness` / 256 x 100 and the wheel ride heights (`wheel_ride_height`
//! ..) = `ride_height` / 256 x 100 (both read from the work copy, whatever block is reset: the
//! setup's percentages, by the look of it); `steering_efficiency` .. `lost_gear_bits` and one more
//! run zeroed; `parts_present` = -1. A reading: the block's grades and values.

use crate::memory::plain_place::WordMut;
use crate::physics::car_store::Car;
use crate::race::numbers_store::Numbers;
use rc2k_core::physics::grades::{Field, percentage, reset_writes};

/// The block's fields written as `reset_writes` lists them.
fn write_all(car: &mut Car, writes: Vec<(Field, u32)>) {
    // The damage gauges in the order they lie in the block, `damage_failed` first and
    // `tyre_wear` last.
    let mut gauges: Vec<WordMut<'_>> = Vec::with_capacity(51);
    gauges.push((&mut car.damage_failed).into());
    gauges.push((&mut car.brake_fade).into());
    gauges.extend(car.damper_wear.iter_mut().map(WordMut::from));
    gauges.extend(car.brake_heat.iter_mut().map(WordMut::from));
    gauges.push((&mut car.engine_heat).into());
    gauges.push((&mut car.engine_damage).into());
    gauges.push((&mut car.exhaust_damage).into());
    gauges.push((&mut car.turbo_damage).into());
    gauges.push((&mut car.cooling_damage).into());
    gauges.push((&mut car.clutch_damage).into());
    gauges.push((&mut car.differential_damage).into());
    gauges.extend(car.gear_wear.iter_mut().map(WordMut::from));
    gauges.push((&mut car.gearbox_damage).into());
    gauges.extend(car.wheel_hit_damage.iter_mut().map(WordMut::from));
    gauges.push((&mut car.body_hit_damage).into());
    gauges.push((&mut car.body_damage).into());
    gauges.push((&mut car.electrics_damage).into());
    gauges.extend(car.light_damage.iter_mut().map(WordMut::from));
    gauges.push((&mut car.lights_damage).into());
    gauges.push((&mut car.steering_damage).into());
    gauges.extend(car.suspension_damage.iter_mut().map(WordMut::from));
    gauges.push((&mut car.overall_suspension_damage).into());
    gauges.push((&mut car.brake_wear_a).into());
    gauges.push((&mut car.brake_wear_b).into());
    gauges.push((&mut car.brake_damage).into());
    gauges.extend(car.tyre_wears.iter_mut().map(WordMut::from));
    gauges.push((&mut car.tyre_wear).into());
    for &(field, v) in &writes {
        if let Field::Gauge(k) = field {
            gauges[k as usize].set_bits(v);
        }
    }
    drop(gauges);
    for (field, v) in writes {
        let mut word: WordMut<'_> = match field {
            Field::Gauge(_) => continue,
            Field::Efficiency(0) => (&mut car.engine_power_factor).into(),
            Field::Efficiency(k @ 1..=2) => (&mut car.brake_efficiency[k as usize - 1]).into(),
            Field::Efficiency(k) => (&mut car.drive_efficiency[k as usize - 3]).into(),
            Field::StiffnessMean => (&mut car.stiffness_mean).into(),
            Field::WheelStiffness(k) => (&mut car.wheel_stiffness[k as usize]).into(),
            Field::RideHeightMean => (&mut car.ride_height_mean).into(),
            Field::WheelRideHeight(k) => (&mut car.wheel_ride_height[k as usize]).into(),
            Field::Failure(0) => (&mut car.steering_efficiency).into(),
            Field::Failure(k @ 1..=7) => (&mut car.electrical_failures[k as usize - 1]).into(),
            Field::Failure(8) => (&mut car.handbrake_failed).into(),
            Field::Failure(9) => (&mut car.gear_change_blocked).into(),
            Field::Failure(10) => (&mut car.forced_neutral).into(),
            Field::Failure(_) => (&mut car.lost_gear_bits).into(),
            Field::ImpactMark(k) => (&mut car.impact_octant_marks[k as usize]).into(),
            Field::PartsPresent => (&mut car.parts_present).into(),
        };
        word.set_bits(v);
    }
}

/// The block `car`, the two settings read at `s0c`, `s08` (the same at any precision).
// Core: `rc2k_core::physics::grades::{percentage, reset_writes}`.
pub(super) fn reset_grades_at(numbers: &Numbers, car: &mut Car, s0c: u32, s08: u32) {
    let (k256, k100) = (numbers.k256.get(), numbers.hundred.get());
    let a = percentage(s0c as i32, k256, k100);
    let c = percentage(s08 as i32, k256, k100);
    write_all(car, reset_writes(numbers.one.get(), a, c));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The writes as the byte-offset version made them.
    fn old_offsets(field: Field) -> usize {
        match field {
            Field::Gauge(k) => 0x29C + 4 * k as usize,
            Field::Efficiency(k) => 0x3F4 + 4 * k as usize,
            Field::StiffnessMean => 0x458,
            Field::WheelStiffness(k) => 0x448 + 4 * k as usize,
            Field::RideHeightMean => 0x45C,
            Field::WheelRideHeight(k) => 0x438 + 4 * k as usize,
            Field::Failure(k) => 0x408 + 4 * k as usize,
            Field::ImpactMark(k) => 0x64 + 4 * k as usize,
            Field::PartsPresent => 0x298,
        }
    }

    #[test]
    fn the_fields_are_the_offsets_they_replace() {
        use crate::physics::car_offsets_1999::{bytes_1999, car_with_1999_bytes, store_1999};
        let writes = reset_writes(1.0, 50.0, 25.0);
        let pattern = |n: usize| (n * 13 + 5) as u8;
        let mut named = car_with_1999_bytes(pattern);
        let mut by_offset = car_with_1999_bytes(pattern);
        let before = bytes_1999(&named);
        for &(field, v) in &writes {
            store_1999(&mut by_offset, old_offsets(field), v);
        }
        write_all(&mut named, writes);
        assert!(
            bytes_1999(&named) == bytes_1999(&by_offset),
            "the named fields wrote other bytes than the offsets"
        );
        assert_ne!(bytes_1999(&named), before);
    }
}
