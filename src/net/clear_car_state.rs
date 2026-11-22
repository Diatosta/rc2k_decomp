//! Makes a car's block ready for a network race: 51 of its dwords are cleared and one is set
//! to -1.

use crate::memory::plain_place::WordMut;
use crate::physics::car_store::Car;

/// A car block's dwords cleared (`clear_car_state`) and what goes in them: 51 of them 0,
/// `parts_present` -1.
type Field = for<'a> fn(&'a mut Car) -> WordMut<'a>;

const CLEARED: [(Field, u32); 52] = [
    (|c| (&mut c.engine_damage).into(), 0),
    (|c| (&mut c.exhaust_damage).into(), 0),
    (|c| (&mut c.electrics_damage).into(), 0),
    (|c| (&mut c.turbo_damage).into(), 0),
    (|c| (&mut c.cooling_damage).into(), 0),
    (|c| (&mut c.steering_damage).into(), 0),
    (|c| (&mut c.clutch_damage).into(), 0),
    (|c| (&mut c.differential_damage).into(), 0),
    (|c| (&mut c.overall_suspension_damage).into(), 0),
    (|c| (&mut c.tyre_wear).into(), 0),
    (|c| (&mut c.brake_damage).into(), 0),
    (|c| (&mut c.lights_damage).into(), 0),
    (|c| (&mut c.gearbox_damage).into(), 0),
    (|c| (&mut c.body_damage).into(), 0),
    (|c| (&mut c.gear_wear[0]).into(), 0),
    (|c| (&mut c.gear_wear[2]).into(), 0),
    (|c| (&mut c.gear_wear[3]).into(), 0),
    (|c| (&mut c.gear_wear[4]).into(), 0),
    (|c| (&mut c.gear_wear[5]).into(), 0),
    (|c| (&mut c.gear_wear[6]).into(), 0),
    (|c| (&mut c.gear_wear[7]).into(), 0),
    (|c| (&mut c.gear_wear[8]).into(), 0),
    (|c| (&mut c.wheel_hit_damage[0]).into(), 0),
    (|c| (&mut c.wheel_hit_damage[1]).into(), 0),
    (|c| (&mut c.wheel_hit_damage[2]).into(), 0),
    (|c| (&mut c.wheel_hit_damage[3]).into(), 0),
    (|c| (&mut c.body_hit_damage).into(), 0),
    (|c| (&mut c.light_damage[0]).into(), 0),
    (|c| (&mut c.light_damage[1]).into(), 0),
    (|c| (&mut c.light_damage[2]).into(), 0),
    (|c| (&mut c.light_damage[3]).into(), 0),
    (|c| (&mut c.light_damage[4]).into(), 0),
    (|c| (&mut c.light_damage[5]).into(), 0),
    (|c| (&mut c.suspension_damage[0]).into(), 0),
    (|c| (&mut c.suspension_damage[1]).into(), 0),
    (|c| (&mut c.suspension_damage[2]).into(), 0),
    (|c| (&mut c.suspension_damage[3]).into(), 0),
    (|c| (&mut c.brake_wear_a).into(), 0),
    (|c| (&mut c.brake_wear_b).into(), 0),
    (|c| (&mut c.tyre_wears[0]).into(), 0),
    (|c| (&mut c.tyre_wears[1]).into(), 0),
    (|c| (&mut c.tyre_wears[2]).into(), 0),
    (|c| (&mut c.tyre_wears[3]).into(), 0),
    (|c| (&mut c.impact_octant_marks[0]).into(), 0),
    (|c| (&mut c.impact_octant_marks[1]).into(), 0),
    (|c| (&mut c.impact_octant_marks[2]).into(), 0),
    (|c| (&mut c.impact_octant_marks[3]).into(), 0),
    (|c| (&mut c.impact_octant_marks[4]).into(), 0),
    (|c| (&mut c.impact_octant_marks[5]).into(), 0),
    (|c| (&mut c.impact_octant_marks[6]).into(), 0),
    (|c| (&mut c.impact_octant_marks[7]).into(), 0),
    (|c| (&mut c.parts_present).into(), u32::MAX),
];
pub fn clear_car_state(block: &mut Car) -> Result<(), crate::errors::BadInput> {
    for (field, v) in CLEARED {
        field(block).set_bits(v);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every selector sits where the core's old table of byte positions had it.
    #[test]
    fn cleared_sit_at_their_old_offsets() {
        let old: [usize; 52] = [
            0x2B8, 0x2BC, 0x310, 0x2C0, 0x2C4, 0x330, 0x2C8, 0x2CC, 0x344, 0x364, 0x350, 0x32C,
            0x2F4, 0x30C, 0x2D0, 0x2D8, 0x2DC, 0x2E0, 0x2E4, 0x2E8, 0x2EC, 0x2F0, 0x2F8, 0x2FC,
            0x300, 0x304, 0x308, 0x314, 0x318, 0x31C, 0x320, 0x324, 0x328, 0x334, 0x338, 0x33C,
            0x340, 0x348, 0x34C, 0x354, 0x358, 0x35C, 0x360, 0x64, 0x68, 0x6C, 0x70, 0x74, 0x78,
            0x7C, 0x80, 0x298,
        ];
        for ((field, v), old) in CLEARED.iter().zip(old) {
            assert_eq!(
                crate::physics::car_offsets_1999::offset_changed_by(|c| field(c).set_bits(1)),
                old
            );
            assert_eq!(*v, if old == 0x298 { u32::MAX } else { 0 });
        }
    }
}
