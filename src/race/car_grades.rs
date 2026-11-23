//! Grades each car block's values (floats, 0 .. 100 by their use) against thresholds into whole
//! numbers, as the last step of race initialisation: `grade_car_values` for one block,
//! `grade_all_cars` for the eight.

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::race::owner_shared::RaceShared;

/// A car's grade: the block's value (a float), where its grade goes, and the thresholds, in the
/// order `grade_car_values` tries them.
pub type Grade = (fn(&Car) -> f32, fn(&mut Car) -> &mut u32, &'static [u32]);

/// The grades `grade_car_values` works out.
pub const GRADES: [Grade; 34] = [
    (
        |c| c.engine_heat,
        |c| &mut c.damage_levels[22],
        &[50, 75, 90, 100],
    ),
    (|c| c.brake_heat[0], |c| &mut c.damage_levels[23], &[75]),
    (
        |c| c.engine_damage,
        |c| &mut c.damage_levels[0],
        &[50, 75, 90, 100],
    ),
    (
        |c| c.exhaust_damage,
        |c| &mut c.damage_levels[1],
        &[50, 75, 90, 100],
    ),
    (
        |c| c.turbo_damage,
        |c| &mut c.damage_levels[2],
        &[50, 75, 90, 100],
    ),
    (
        |c| c.cooling_damage,
        |c| &mut c.damage_levels[3],
        &[50, 75, 90, 100],
    ),
    (
        |c| c.clutch_damage,
        |c| &mut c.damage_levels[4],
        &[50, 75, 90, 100],
    ),
    (
        |c| c.differential_damage,
        |c| &mut c.damage_levels[5],
        &[50, 75, 90, 100],
    ),
    (
        |c| c.gearbox_damage,
        |c| &mut c.damage_levels[6],
        &[50, 75, 90, 100],
    ),
    (
        |c| c.body_damage,
        |c| &mut c.damage_levels[7],
        &[50, 75, 90, 100],
    ),
    (
        |c| c.electrics_damage,
        |c| &mut c.damage_levels[8],
        &[50, 75, 90, 100],
    ),
    (
        |c| c.steering_damage,
        |c| &mut c.damage_levels[9],
        &[50, 75, 90, 100],
    ),
    (
        |c| c.brake_damage,
        |c| &mut c.damage_levels[10],
        &[50, 75, 90, 100],
    ),
    (
        |c| c.overall_suspension_damage,
        |c| &mut c.damage_levels[13],
        &[50, 75, 90, 100],
    ),
    (
        |c| c.lights_damage,
        |c| &mut c.damage_levels[11],
        &[50, 75, 90],
    ),
    (|c| c.light_damage[0], |c| &mut c.damage_levels[25], &[100]),
    (|c| c.light_damage[1], |c| &mut c.damage_levels[26], &[100]),
    (|c| c.light_damage[2], |c| &mut c.damage_levels[27], &[100]),
    (|c| c.light_damage[3], |c| &mut c.damage_levels[28], &[100]),
    (|c| c.light_damage[4], |c| &mut c.damage_levels[29], &[100]),
    (|c| c.light_damage[5], |c| &mut c.damage_levels[30], &[100]),
    (
        |c| c.wheel_hit_damage[0],
        |c| &mut c.damage_levels[31],
        &[100],
    ),
    (
        |c| c.wheel_hit_damage[1],
        |c| &mut c.damage_levels[32],
        &[100],
    ),
    (
        |c| c.wheel_hit_damage[2],
        |c| &mut c.damage_levels[33],
        &[100],
    ),
    (
        |c| c.wheel_hit_damage[3],
        |c| &mut c.damage_levels[34],
        &[100],
    ),
    (|c| c.tyre_wear, |c| &mut c.damage_levels[12], &[50, 75, 90]),
    (|c| c.gear_wear[0], |c| &mut c.damage_levels[14], &[100]),
    (|c| c.gear_wear[2], |c| &mut c.damage_levels[15], &[100]),
    (|c| c.gear_wear[3], |c| &mut c.damage_levels[16], &[100]),
    (|c| c.gear_wear[4], |c| &mut c.damage_levels[17], &[100]),
    (|c| c.gear_wear[5], |c| &mut c.damage_levels[18], &[100]),
    (|c| c.gear_wear[6], |c| &mut c.damage_levels[19], &[100]),
    (|c| c.gear_wear[7], |c| &mut c.damage_levels[20], &[100]),
    (|c| c.gear_wear[8], |c| &mut c.damage_levels[21], &[100]),
];

/// For each of `GRADES`, stores as the grade the number of its thresholds, taken in order, that lie
/// below the block's value (a NaN passes them all). Each threshold tried goes through `scratch`;
/// its one caller reads nothing of it.
fn grade_car_values(car: &mut Car, scene: &crate::scene::owner::Scene) -> Result<(), BadInput> {
    // `scratch` gets the last threshold tried (the ones before it are written over at once); the
    // core is `rc2k_core::race::cars::grade`.
    for &(from, to, thresholds) in &GRADES {
        let x = from(car);
        let (grade, tried) = rc2k_core::race::cars::grade(x, thresholds);
        scene.draw_work_mut().scratch = tried;
        *to(car) = grade;
    }
    Ok(())
}

/// Grades the eight blocks (`car_blocks`) with `grade_car_values`.
pub fn grade_all_cars(
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    for n in 0..8usize {
        grade_car_values(&mut *shared.race_car_mut(n)?, scene)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Each dword of a car holds its own 1999 offset: the selector's float names it.
        let car = crate::physics::car_offsets_1999::car_with_1999_bytes(|n| {
            ((n & !3) as u32).to_le_bytes()[n & 3]
        });
        for ((value, grade, _), (from, to)) in GRADES.iter().zip(old) {
            assert_eq!(value(&car).to_bits() as usize, from);
            assert_eq!(
                crate::physics::car_offsets_1999::offset_changed_by(|c| *grade(c) = 1),
                to
            );
        }
    }
}
