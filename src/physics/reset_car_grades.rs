//! Resets a car block (0xFE0 bytes, the work copy's layout): `damage_failed` and the run of fields
//! after it are zeroed; `engine_power_factor` and the run after it = 1.0; the wheel stiffness
//! fields = `suspension_stiffness` / 256 x 100 and the wheel ride heights (`wheel_ride_height`
//! ..) = `ride_height` / 256 x 100 (both read from the work copy, whatever block is reset: the
//! setup's percentages, by the look of it); `steering_efficiency` .. `lost_gear_bits` and one more
//! run zeroed; `parts_present` = -1. A reading: the block's grades and values.

use core::slice::from_ref;

use crate::memory::plain_floats::{PlainFloats, WordRef};
use crate::memory::session::{Cell32, PlainDword};
use crate::physics::car_store::Car;
use crate::race::numbers_store::Numbers;
use rc2k_core::physics::grades::{Field, percentage, reset_writes};

/// The damage gauges in the order they lie in the block, `damage_failed` first and `tyre_wear`
/// last.
fn gauges(car: &Car) -> Vec<WordRef<'_>> {
    fn run<'a, C: Cell32>(list: &mut Vec<WordRef<'a>>, cells: &'a [C]) {
        list.extend(cells.iter().map(|c| WordRef::Cell(c)));
    }
    fn row<'a, const N: usize>(list: &mut Vec<WordRef<'a>>, floats: &'a PlainFloats<N>) {
        list.extend((0..N).map(|k| WordRef::Element(floats.cell_at(k))));
    }
    let mut g: Vec<WordRef<'_>> = Vec::with_capacity(51);
    run(&mut g, from_ref(&car.damage_failed));
    run(&mut g, from_ref(&car.brake_fade));
    row(&mut g, &car.damper_wear);
    row(&mut g, &car.brake_heat);
    run(&mut g, from_ref(&car.engine_heat));
    run(&mut g, from_ref(&car.engine_damage));
    run(&mut g, from_ref(&car.exhaust_damage));
    run(&mut g, from_ref(&car.turbo_damage));
    run(&mut g, from_ref(&car.cooling_damage));
    run(&mut g, from_ref(&car.clutch_damage));
    run(&mut g, from_ref(&car.differential_damage));
    row(&mut g, &car.gear_wear);
    run(&mut g, from_ref(&car.gearbox_damage));
    row(&mut g, &car.wheel_hit_damage);
    run(&mut g, from_ref(&car.body_hit_damage));
    run(&mut g, from_ref(&car.body_damage));
    run(&mut g, from_ref(&car.electrics_damage));
    row(&mut g, &car.light_damage);
    run(&mut g, from_ref(&car.lights_damage));
    run(&mut g, from_ref(&car.steering_damage));
    row(&mut g, &car.suspension_damage);
    run(&mut g, from_ref(&car.overall_suspension_damage));
    run(&mut g, from_ref(&car.brake_wear_a));
    run(&mut g, from_ref(&car.brake_wear_b));
    run(&mut g, from_ref(&car.brake_damage));
    row(&mut g, &car.tyre_wears);
    run(&mut g, from_ref(&car.tyre_wear));
    g
}

/// The cell the block keeps `field` in (`gauges` are the block's, from `gauges`).
fn cell<'a>(car: &'a Car, gauges: &[WordRef<'a>], field: Field) -> WordRef<'a> {
    match field {
        Field::Gauge(k) => gauges[k as usize],
        Field::Efficiency(0) => WordRef::Cell(&car.engine_power_factor),
        Field::Efficiency(k @ 1..=2) => {
            WordRef::Element(car.brake_efficiency.cell_at(k as usize - 1))
        }
        Field::Efficiency(k) => WordRef::Element(car.drive_efficiency.cell_at(k as usize - 3)),
        Field::StiffnessMean => WordRef::Cell(&car.stiffness_mean),
        Field::WheelStiffness(k) => WordRef::Element(car.wheel_stiffness.cell_at(k as usize)),
        Field::RideHeightMean => WordRef::Cell(&car.ride_height_mean),
        Field::WheelRideHeight(k) => WordRef::Element(car.wheel_ride_height.cell_at(k as usize)),
        Field::Failure(0) => WordRef::Cell(&car.steering_efficiency),
        Field::Failure(k @ 1..=7) => WordRef::Cell(&car.electrical_failures[k as usize - 1]),
        Field::Failure(8) => WordRef::Cell(&car.handbrake_failed),
        Field::Failure(9) => WordRef::Cell(&car.gear_change_blocked),
        Field::Failure(10) => WordRef::Cell(&car.forced_neutral),
        Field::Failure(_) => WordRef::Cell(&car.lost_gear_bits),
        Field::ImpactMark(k) => WordRef::Element(car.impact_octant_marks.cell_at(k as usize)),
        Field::PartsPresent => WordRef::Cell(&car.parts_present),
    }
}

/// The block's fields written as `reset_writes` lists them.
fn write_all(car: &Car, writes: Vec<(Field, u32)>) {
    let gauges = gauges(car);
    for (field, v) in writes {
        cell(car, &gauges, field).set_bits(v);
    }
}

/// The block `car`, the two settings read at `s0c`, `s08` (the same at any precision).
// Core: `rc2k_core::physics::grades::{percentage, reset_writes}`.
pub(super) fn reset_grades_at(numbers: &Numbers, car: &Car, s0c: &PlainDword, s08: &PlainDword) {
    let (k256, k100) = (numbers.k256.get(), numbers.hundred.get());
    let a = percentage(s0c.get() as i32, k256, k100);
    let c = percentage(s08.get() as i32, k256, k100);
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
        let named = car_with_1999_bytes(pattern);
        let by_offset = car_with_1999_bytes(pattern);
        let before = bytes_1999(&named);
        for &(field, v) in &writes {
            store_1999(&by_offset, old_offsets(field), v);
        }
        write_all(&named, writes);
        assert!(
            bytes_1999(&named) == bytes_1999(&by_offset),
            "the named fields wrote other bytes than the offsets"
        );
        assert_ne!(bytes_1999(&named), before);
    }
}
