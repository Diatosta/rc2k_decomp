//! Race init's last step (`race::grade_all_cars`, IDA 2026-10-17): each car block's values (floats,
//! 0 .. 100 by their use) graded against thresholds into whole numbers: `race::grade_car_values`
//! for one block, `race::grade_all_cars` for the eight.

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::race::owner_shared::RaceShared;
use crate::session::{Dword, Float};

/// A car's grade: the block's value (a float), where its grade goes, the thresholds, in the
/// original's order (`race::grade_car_values`).
pub type Grade = (fn(&Car) -> &Float, fn(&Car) -> &Dword, &'static [u32]);

/// The grades `race::grade_car_values` works out.
pub const GRADES: [Grade; 34] = [
    (
        |c| &c.engine_heat,
        |c| &c.damage_levels[22],
        &[50, 75, 90, 100],
    ),
    (|c| &c.brake_heat[0], |c| &c.damage_levels[23], &[75]),
    (
        |c| &c.engine_damage,
        |c| &c.damage_levels[0],
        &[50, 75, 90, 100],
    ),
    (
        |c| &c.exhaust_damage,
        |c| &c.damage_levels[1],
        &[50, 75, 90, 100],
    ),
    (
        |c| &c.turbo_damage,
        |c| &c.damage_levels[2],
        &[50, 75, 90, 100],
    ),
    (
        |c| &c.cooling_damage,
        |c| &c.damage_levels[3],
        &[50, 75, 90, 100],
    ),
    (
        |c| &c.clutch_damage,
        |c| &c.damage_levels[4],
        &[50, 75, 90, 100],
    ),
    (
        |c| &c.differential_damage,
        |c| &c.damage_levels[5],
        &[50, 75, 90, 100],
    ),
    (
        |c| &c.gearbox_damage,
        |c| &c.damage_levels[6],
        &[50, 75, 90, 100],
    ),
    (
        |c| &c.body_damage,
        |c| &c.damage_levels[7],
        &[50, 75, 90, 100],
    ),
    (
        |c| &c.electrics_damage,
        |c| &c.damage_levels[8],
        &[50, 75, 90, 100],
    ),
    (
        |c| &c.steering_damage,
        |c| &c.damage_levels[9],
        &[50, 75, 90, 100],
    ),
    (
        |c| &c.brake_damage,
        |c| &c.damage_levels[10],
        &[50, 75, 90, 100],
    ),
    (
        |c| &c.overall_suspension_damage,
        |c| &c.damage_levels[13],
        &[50, 75, 90, 100],
    ),
    (
        |c| &c.lights_damage,
        |c| &c.damage_levels[11],
        &[50, 75, 90],
    ),
    (|c| &c.light_damage[0], |c| &c.damage_levels[25], &[100]),
    (|c| &c.light_damage[1], |c| &c.damage_levels[26], &[100]),
    (|c| &c.light_damage[2], |c| &c.damage_levels[27], &[100]),
    (|c| &c.light_damage[3], |c| &c.damage_levels[28], &[100]),
    (|c| &c.light_damage[4], |c| &c.damage_levels[29], &[100]),
    (|c| &c.light_damage[5], |c| &c.damage_levels[30], &[100]),
    (|c| &c.wheel_hit_damage[0], |c| &c.damage_levels[31], &[100]),
    (|c| &c.wheel_hit_damage[1], |c| &c.damage_levels[32], &[100]),
    (|c| &c.wheel_hit_damage[2], |c| &c.damage_levels[33], &[100]),
    (|c| &c.wheel_hit_damage[3], |c| &c.damage_levels[34], &[100]),
    (|c| &c.tyre_wear, |c| &c.damage_levels[12], &[50, 75, 90]),
    (|c| &c.gear_wear[0], |c| &c.damage_levels[14], &[100]),
    (|c| &c.gear_wear[2], |c| &c.damage_levels[15], &[100]),
    (|c| &c.gear_wear[3], |c| &c.damage_levels[16], &[100]),
    (|c| &c.gear_wear[4], |c| &c.damage_levels[17], &[100]),
    (|c| &c.gear_wear[5], |c| &c.damage_levels[18], &[100]),
    (|c| &c.gear_wear[6], |c| &c.damage_levels[19], &[100]),
    (|c| &c.gear_wear[7], |c| &c.damage_levels[20], &[100]),
    (|c| &c.gear_wear[8], |c| &c.damage_levels[21], &[100]),
];

/// `race::grade_car_values` (0x43FE24): for each of `GRADES`, the grade (+the second offset) the
/// number of its thresholds, taken in order, that lie below the block's value (+the first; a NaN
/// passes them all); each threshold tried goes through `dword_62400C`. its one caller reads nothing
/// of it.
pub fn grade_car_values(block: u32, scene: &crate::scene::owner::Scene) -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::cars::grade`; `dword_62400C` gets the last threshold
    // tried (the ones before it are written over at once).
    let car = crate::race::trap::car_at(block)?;
    for &(from, to, thresholds) in &GRADES {
        let x = f32::from_bits(from(car).bits());
        let (grade, tried) = rc2k_core::race::cars::grade(x, thresholds);
        scene.drawing.scene.scratch.set_bits(tried);
        to(car).set_bits(grade);
    }
    Ok(())
}

/// `race::grade_all_cars` (0x43FDFE): `grade_car_values` of the eight blocks (`g_carBlocks`).
pub fn grade_all_cars(
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    for n in 0..8usize {
        grade_car_values(shared.work.car_block(n), scene)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::race::blocks_store::CAR_BLOCKS;

    /// Every selector sits where the core's old table of byte offsets had it.
    #[test]
    fn grades_sit_at_their_old_offsets() {
        let old: [(usize, usize); 34] = [
            (0x2B4, 0x3C0),
            (0x2AC, 0x3C4),
            (0x2B8, 0x368),
            (0x2BC, 0x36C),
            (0x2C0, 0x370),
            (0x2C4, 0x374),
            (0x2C8, 0x378),
            (0x2CC, 0x37C),
            (0x2F4, 0x380),
            (0x30C, 0x384),
            (0x310, 0x388),
            (0x330, 0x38C),
            (0x350, 0x390),
            (0x344, 0x39C),
            (0x32C, 0x394),
            (0x314, 0x3CC),
            (0x318, 0x3D0),
            (0x31C, 0x3D4),
            (0x320, 0x3D8),
            (0x324, 0x3DC),
            (0x328, 0x3E0),
            (0x2F8, 0x3E4),
            (0x2FC, 0x3E8),
            (0x300, 0x3EC),
            (0x304, 0x3F0),
            (0x364, 0x398),
            (0x2D0, 0x3A0),
            (0x2D8, 0x3A4),
            (0x2DC, 0x3A8),
            (0x2E0, 0x3AC),
            (0x2E4, 0x3B0),
            (0x2E8, 0x3B4),
            (0x2EC, 0x3B8),
            (0x2F0, 0x3BC),
        ];
        let car = &CAR_BLOCKS.cars[0];
        let base = core::ptr::from_ref(car) as usize;
        for ((value, grade, _), (from, to)) in GRADES.iter().zip(old) {
            assert_eq!(core::ptr::from_ref(value(car)) as usize - base, from);
            assert_eq!(core::ptr::from_ref(grade(car)) as usize - base, to);
        }
    }
}
