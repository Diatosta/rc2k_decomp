//! `net::clear_car_state` (0x463EB6): a car's block made ready for a network race: 51 of
//! its dwords (+0x64 .. +0x80, +0x2B8 .. +0x364) 0, +0x298 -1.
//!
//! Takes `block`.
//! Caller: `net::reset_cars`.

use crate::physics::car_store::Car;
use crate::session::Cell32;

/// A car block's dwords cleared (`clear_car_state`), in the original's order, and what goes
/// in them: 51 of them 0, `parts_present` (+0x298) -1.
type Field = fn(&Car) -> &dyn Cell32;

const CLEARED: [(Field, u32); 52] = [
    (|c| &c.engine_damage, 0),
    (|c| &c.exhaust_damage, 0),
    (|c| &c.electrics_damage, 0),
    (|c| &c.turbo_damage, 0),
    (|c| &c.cooling_damage, 0),
    (|c| &c.steering_damage, 0),
    (|c| &c.clutch_damage, 0),
    (|c| &c.differential_damage, 0),
    (|c| &c.overall_suspension_damage, 0),
    (|c| &c.tyre_wear, 0),
    (|c| &c.brake_damage, 0),
    (|c| &c.lights_damage, 0),
    (|c| &c.gearbox_damage, 0),
    (|c| &c.body_damage, 0),
    (|c| &c.gear_wear[0], 0),
    (|c| &c.gear_wear[2], 0),
    (|c| &c.gear_wear[3], 0),
    (|c| &c.gear_wear[4], 0),
    (|c| &c.gear_wear[5], 0),
    (|c| &c.gear_wear[6], 0),
    (|c| &c.gear_wear[7], 0),
    (|c| &c.gear_wear[8], 0),
    (|c| &c.wheel_hit_damage[0], 0),
    (|c| &c.wheel_hit_damage[1], 0),
    (|c| &c.wheel_hit_damage[2], 0),
    (|c| &c.wheel_hit_damage[3], 0),
    (|c| &c.body_hit_damage, 0),
    (|c| &c.light_damage[0], 0),
    (|c| &c.light_damage[1], 0),
    (|c| &c.light_damage[2], 0),
    (|c| &c.light_damage[3], 0),
    (|c| &c.light_damage[4], 0),
    (|c| &c.light_damage[5], 0),
    (|c| &c.suspension_damage[0], 0),
    (|c| &c.suspension_damage[1], 0),
    (|c| &c.suspension_damage[2], 0),
    (|c| &c.suspension_damage[3], 0),
    (|c| &c.brake_wear_a, 0),
    (|c| &c.brake_wear_b, 0),
    (|c| &c.part_hit_damage[0], 0),
    (|c| &c.part_hit_damage[1], 0),
    (|c| &c.part_hit_damage[2], 0),
    (|c| &c.part_hit_damage[3], 0),
    (|c| &c.impact_octant_marks[0], 0),
    (|c| &c.impact_octant_marks[1], 0),
    (|c| &c.impact_octant_marks[2], 0),
    (|c| &c.impact_octant_marks[3], 0),
    (|c| &c.impact_octant_marks[4], 0),
    (|c| &c.impact_octant_marks[5], 0),
    (|c| &c.impact_octant_marks[6], 0),
    (|c| &c.impact_octant_marks[7], 0),
    (|c| &c.parts_present, u32::MAX),
];

/// The Rust version.
pub fn clear_car_state(block: &Car) -> Result<(), crate::errors::BadInput> {
    for (field, v) in CLEARED {
        field(block).set_bits(v);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every selector sits where the core's old table of byte offsets had it.
    #[test]
    fn cleared_sit_at_their_old_offsets() {
        let old: [usize; 52] = [
            0x2B8, 0x2BC, 0x310, 0x2C0, 0x2C4, 0x330, 0x2C8, 0x2CC, 0x344, 0x364, 0x350, 0x32C,
            0x2F4, 0x30C, 0x2D0, 0x2D8, 0x2DC, 0x2E0, 0x2E4, 0x2E8, 0x2EC, 0x2F0, 0x2F8, 0x2FC,
            0x300, 0x304, 0x308, 0x314, 0x318, 0x31C, 0x320, 0x324, 0x328, 0x334, 0x338, 0x33C,
            0x340, 0x348, 0x34C, 0x354, 0x358, 0x35C, 0x360, 0x64, 0x68, 0x6C, 0x70, 0x74, 0x78,
            0x7C, 0x80, 0x298,
        ];
        let car = &crate::race::blocks_store::CAR_BLOCKS.cars[0];
        let base = core::ptr::from_ref(car) as usize;
        for ((field, v), old) in CLEARED.iter().zip(old) {
            assert_eq!(
                core::ptr::from_ref(field(car)).cast::<u8>() as usize - base,
                old
            );
            assert_eq!(*v, if old == 0x298 { u32::MAX } else { 0 });
        }
    }
}
