//! The damage model's figures: shares, thresholds, the work floats, the tables of four (x0, y0, x1,
//! y1) `physics::flow::table_lerp` reads, and the rates. Atomic cells, one static. What each figure
//! is, is a reading from how `physics::damage_factors`, `damage_wear`, `damage_gauges` and
//! `rate_damage` use it (`physics/damage_model.rs`); the doc of each says what.

use crate::errors::BadInput;
use crate::memory::session::{PlainDword, PlainFloat, PlainInt};

/// The damage model's figures.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct DamageModel {
    /// A figure times it gives the failure chance (`physics::rate_damage`).
    pub chance_scale: PlainFloat,
    /// 6.0, the number of lights: the six light gauges' sum is divided by it for their mean
    /// (`lights_damage`).
    pub light_count_divisor: PlainFloat,
    /// The working grades are held up to it.
    pub grade_high: PlainFloat,
    /// The gauges and gears are held 0 ..= it; a gear equal to it is worn out.
    pub gauge_top: PlainFloat,
    /// The gauges and grades are held down to it.
    pub gauge_low: PlainFloat,
    /// 8.0, the highest landing dive figure counted (`physics::damage_factors` holds the dive 0 ..=
    /// it before squaring).
    pub dive_top: PlainFloat,
    /// The divisor (256) that turns the car block's whole-number figures into shares.
    pub fixed_point_scale: PlainFloat,
    /// 0.5: the average of two (brake wear, tyre wear, gearbox, the wheel loads' blend).
    pub half: PlainFloat,
    /// 0.25: the average of four (the wheel stiffness / ride-height means, the wheel loads).
    pub quarter: PlainFloat,
    /// 0.1: the body damage gauge is the sum of its parts times it.
    pub tenth: PlainFloat,
    /// The work cell a rounding to a whole number goes through.
    pub scratch: PlainFloat,
    /// 0.01: the percent factor the wear and hit terms are multiplied by.
    pub hundredth: PlainFloat,
    /// 70.0: the forward speed (and speed gaps) are held to it and divided by it, then scaled to
    /// `gauge_top`, so a speed share runs 0 ..= 100.
    pub speed_range: PlainFloat,
    /// 0.5: the engine counts as over its limit when its speed is above `engine_speed_limit` times
    /// it.
    pub engine_limit_factor: PlainFloat,
    /// 1609.34, the metres in a mile: the tyre durability (`wear_figure`) is turned to this unit
    /// before the speed share is divided by it (probably, the durability figures being miles).
    pub metres_per_mile: PlainFloat,
    /// The factor `physics::rate_damage` multiplies by (80 bits).
    pub grade_k: PlainFloat,
    /// About 1/6000 (0.000167): the gauge rates are multiplied by it, so a gauge rises by its rate
    /// times this per step.
    pub gauge_step: PlainFloat,
    /// The step the wear rates are multiplied by (`physics::damage_wear`).
    pub wear_step: PlainFloat,
    /// 45.0: the hit size is the contact depth squared over it and over `speed_range`.
    pub hit_divisor: PlainFloat,
    /// The engine speed over its limit, scaled to `gauge_top`.
    pub engine_speed_share: PlainFloat,
    /// The forward speed over `speed_range`, scaled to `gauge_top`.
    pub speed_share: PlainFloat,
    /// The gap between the forward speed and drive speed A over `speed_range`.
    pub drive_a_slip_share: PlainFloat,
    /// The gap between the forward speed and drive speed B over `speed_range`.
    pub drive_b_slip_share: PlainFloat,
    /// Drive speed A pushed past the forward speed, over `speed_range`.
    pub drive_a_extrapolated_share: PlainFloat,
    /// Drive speed B pushed past the forward speed, over `speed_range`.
    pub drive_b_extrapolated_share: PlainFloat,
    /// The brake figure over `fixed_point_scale` while the car moves and the brake is 0x10 or more.
    pub brake_share: PlainFloat,
    /// 1 while the car's centre point is on surface kind 5 (wet, as `wheel_surface_a` says), else
    /// 0: it wets the brakes (fade) and damages the electrics and the lights.
    pub on_surface_5: PlainFloat,
    /// Copy of the car's `gear_changed`: 1 after a shift up, -1 after a shift down, else 0 (the
    /// gauges add the clutch and gear wear of a shift).
    pub gear_changed: PlainInt,
    /// Copy of the car's `gear`: -1 (reverse), 0 (neutral), 1 ..= 7; picks the gear's wear gauge
    /// and factor.
    pub gear: PlainInt,
    /// 1.0 when the engine speed is above `engine_speed_limit` times `engine_limit_factor`, else 0:
    /// the turbo wear while over the limit.
    pub engine_over_limit: PlainFloat,
    /// Where the car was hit: 0 none, else 1 ..= 4 by the side of the contact point (1 + 1 if the
    /// point's x is above 0 + 2 if its z is 0 or below).
    pub hit_side: PlainDword,
    /// The size of this step's hit: the contact depth squared over `hit_divisor` over
    /// `speed_range`; scaled by the damage setting (`patch_factors[1]`) in the gauges, zeroed while
    /// the hit cooldown runs.
    pub hit_size: PlainFloat,
    /// The four wheels' loads, blended by `DamageModel.half` (`physics::damage_factors`).
    pub wheel_loads: [PlainFloat; 4],
    /// The mean of the four wheel loads (their sum times `quarter`).
    pub wheel_load_mean: PlainFloat,
    /// Copy of the centre point's surface figure (the surface's wear figure blended between its two
    /// cells): how hard the ground is on the car; it scales the roughness wear. Probably a
    /// roughness.
    pub point_surface_copy: PlainFloat,
    /// Copy of the car's `drive_kind` (a whole number): picks the entry of the two kind tables (the
    /// kind must be 0 or 1 there), and a car with 0 has no turbo damage.
    pub drive_kind: PlainDword,
    /// Copy of the car's `tyre_kind`: the row of `wear_table` (16 dwords a row).
    pub tyre_kind: PlainDword,
    /// Copy of the centre point's surface kind (`wheel_surface_a[4]`): its low four bits pick the
    /// wear table's columns.
    pub surface: PlainDword,
    /// Copy of the car's `surface_blend` (a float, 0 ..= 1, held in a dword cell): the weight of
    /// the second surface's column against the first when `wear_figure` is blended.
    pub surface_blend: PlainFloat,
    /// 1 when at least one wheel is not off the ground, else 0.
    pub wheels_down: PlainFloat,
    /// The car's ride-height figure over `fixed_point_scale`.
    pub ride_height_share: PlainFloat,
    /// The car's suspension-stiffness figure over `fixed_point_scale`.
    pub stiffness_share: PlainFloat,
    /// The car's brake-balance figure over `fixed_point_scale`.
    pub brake_balance_share: PlainFloat,
    /// The chance the electrics fail this step (ramp of `electrics_damage`, table 11),
    /// `electrical_failures[0]` set by it (`physics::rate_damage`).
    pub electrics_fail_chance: PlainFloat,
    /// The chance light 0 (`light_damage[0]`, table 11) goes out this step,
    /// `electrical_failures[1 + 0]` set by it.
    pub light_fail_chance_0: PlainFloat,
    /// The chance light 1 (`light_damage[1]`, table 11) goes out this step,
    /// `electrical_failures[1 + 1]` set by it.
    pub light_fail_chance_1: PlainFloat,
    /// The chance light 2 (`light_damage[2]`, table 11) goes out this step,
    /// `electrical_failures[1 + 2]` set by it.
    pub light_fail_chance_2: PlainFloat,
    /// The chance light 3 (`light_damage[3]`, table 11) goes out this step,
    /// `electrical_failures[1 + 3]` set by it.
    pub light_fail_chance_3: PlainFloat,
    /// The chance light 4 (`light_damage[4]`, table 11) goes out this step,
    /// `electrical_failures[1 + 4]` set by it.
    pub light_fail_chance_4: PlainFloat,
    /// The chance light 5 (`light_damage[5]`, table 11) goes out this step,
    /// `electrical_failures[1 + 5]` set by it.
    pub light_fail_chance_5: PlainFloat,
    /// The chance a failed electrics recovers this step (ramp of `electrics_damage`, table 11).
    pub electrics_recover_chance: PlainFloat,
    /// The chance a failed light 0 comes back this step (ramp of `light_damage[0]`, table 12).
    pub light_recover_chance_0: PlainFloat,
    /// The chance a failed light 1 comes back this step (ramp of `light_damage[1]`, table 12).
    pub light_recover_chance_1: PlainFloat,
    /// The chance a failed light 2 comes back this step (ramp of `light_damage[2]`, table 12).
    pub light_recover_chance_2: PlainFloat,
    /// The chance a failed light 3 comes back this step (ramp of `light_damage[3]`, table 12).
    pub light_recover_chance_3: PlainFloat,
    /// The chance a failed light 4 comes back this step (ramp of `light_damage[4]`, table 12).
    pub light_recover_chance_4: PlainFloat,
    /// The chance a failed light 5 comes back this step (ramp of `light_damage[5]`, table 12).
    pub light_recover_chance_5: PlainFloat,
    /// 1 - `brake_balance_share`, the other axle's brake share: `brake_wear_b` rises by its rate
    /// times this. `physics::rate_damage` reuses the cell as the scratch of the drive-efficiency
    /// ramp.
    pub brake_balance_complement: PlainFloat,
    /// The front tyres' wear, the mean of `tyre_wears` 0 and 1 (FL, FR; "Front tyres" on the
    /// developers' wear page): the input of the drive-efficiency ramp for drive speed A, 0 ..= 100.
    pub front_tyre_wear: PlainFloat,
    /// The rear tyres' wear, the mean of `tyre_wears` 2 and 3 (RL, RR; "Rear tyres"): as
    /// `front_tyre_wear` for drive speed B.
    pub rear_tyre_wear: PlainFloat,
    /// The stiffness mean through `lerp_tables_b[0]`, squared: scales the suspension's roughness
    /// wear (`suspension_roughness_wear`).
    pub stiffness_suspension_factor: PlainFloat,
    /// The stiffness mean through `lerp_tables_b[2]`, squared: scales the body roughness wear.
    pub stiffness_body_factor: PlainFloat,
    /// Surface figure times the speed share times `figures[32]` times `wheels_down`: the
    /// rough-ground wear before any part's factor; it wears the wheel hit gauges (times
    /// `wheel_roughness_rate`).
    pub roughness_wear_base: PlainFloat,
    /// `roughness_wear_base` times `stiffness_body_factor` and `ride_height_body_factor`: the rate
    /// for the body, steering, engine, exhaust, gears and differential gauges.
    pub body_roughness_wear: PlainFloat,
    /// `roughness_wear_base` times `stiffness_suspension_factor`: the rate for the four suspension
    /// gauges.
    pub suspension_roughness_wear: PlainFloat,
    /// The engine speed share through `lerp_tables[0]`: the rate of the engine and cooling gauges'
    /// wear from revving.
    pub engine_speed_wear_factor: PlainFloat,
    /// The cooling damage through `lerp_tables[1]`: the engine speed share the cooling can still
    /// carry off.
    pub cooling_capacity: PlainFloat,
    /// Engine speed share minus `cooling_capacity`, held at 0 or more: the heat the cooling cannot
    /// carry; raises the engine heat and, when above 0, the engine and cooling damage.
    pub overheat_excess: PlainFloat,
    /// The ride-height mean through `lerp_tables_b[1]`, squared: scales the body roughness wear.
    pub ride_height_body_factor: PlainFloat,
    /// The drive-A slip share through `lerp_tables[3]`: how much slip adds to the front tyres'
    /// wear.
    pub front_slip_wear_factor: PlainFloat,
    /// The drive-B slip share through `lerp_tables[3]`: the same for tyre gauges 2 and 3.
    pub rear_slip_wear_factor: PlainFloat,
    /// The four wheels' table lerps of the suspension damage (`physics::damage_factors`).
    pub wheel_stiffness_lerp: [PlainFloat; 4],
    /// The tyre durability on this ground: the two columns of `wear_table` (whole numbers, e.g.
    /// 100, 20, 60, 5) blended by `surface_blend`, times `wear_figure_scale`; the tyre wear rates
    /// divide by it, so a bigger figure wears slower.
    pub wear_figure: PlainFloat,
    /// The speed share over (`wear_figure` times `metres_per_mile`) times `gauge_top` times
    /// `wheels_down`, times the kind table A entry: the speed part of the front tyres' wear rate.
    pub front_tyre_wear_rate: PlainFloat,
    /// As `front_tyre_wear_rate`, times the kind table B entry: the speed part of the rear tyres'
    /// wear rate.
    pub rear_tyre_wear_rate: PlainFloat,
    /// The chance gear changes are blocked this step (ramp of `clutch_damage`, table 7; sets
    /// `gear_change_blocked`).
    pub clutch_block_chance: PlainFloat,
    /// The chance gear changes are blocked (ramp of `gearbox_damage`, table 8; sets
    /// `gear_change_blocked`).
    pub gearbox_block_chance: PlainFloat,
    /// The chance the gearbox is forced into neutral (ramp of `gearbox_damage`, table 9; sets
    /// `forced_neutral`).
    pub gearbox_neutral_chance: PlainFloat,
    /// The chance the handbrake fails for good (ramp of `brake_damage`, table 10; sets
    /// `handbrake_failed`).
    pub handbrake_fail_chance: PlainFloat,
    /// The clock / random figure the failures by chance compare against.
    pub clock: PlainDword,
    /// The tables of four (x0, y0, x1, y1) (`physics::damage_wear`, `rate_damage`,
    /// `damage_factors`).
    pub lerp_tables: [[PlainFloat; 4]; 13],
    /// The wear rates `physics::damage_wear` raises and lowers the gauges by.
    pub wear_rates: [PlainFloat; 13],
    /// The gear factors by gear + 1 (reverse, neutral, 1 .. 7), `physics::damage_gauges`; index 1
    /// is the neutral entry, read when the gear is 0 like the others.
    pub gear_factors: [PlainFloat; 9],
    /// The gauges' rates (`physics::damage_gauges`), times the step's shares.
    pub gauge_rates: [PlainFloat; 24],
    /// The first kind table (by `DamageModel.drive_kind`'s bits).
    pub kind_table_a: [PlainFloat; 2],
    /// The second kind table.
    pub kind_table_b: [PlainFloat; 2],
    /// Three more tables of four.
    pub lerp_tables_b: [[PlainFloat; 4]; 3],
    /// 0.15: the rate of the wheel hit gauges' wear by rough ground (times `roughness_wear_base`).
    pub wheel_roughness_rate: PlainFloat,
    /// More hit figures and thresholds (`physics::damage_gauges`, `rate_damage`).
    pub figures: [PlainFloat; 33],
    /// The row of figures a car with no tyre kind set (`tyre_kind` -1) gets; one row before
    /// `wear_table`, as the index runs.
    pub wear_unset_row: [PlainDword; 16],
    /// The wear table, a row of 16 whole numbers per `DamageModel.tyre_kind` 0 ..= 5.
    pub wear_table: [[PlainDword; 16]; 6],
    /// 2.5: `wear_figure` is the blended table figure times it.
    pub wear_figure_scale: PlainFloat,
    /// The wear table's two columns by surface kind (its low four bits): the first and the second
    /// column of the pair, whole numbers.
    pub wear_columns: [[PlainDword; 2]; 16],
}

impl DamageModel {
    /// The wear table's figure in `column` of the row for `tyre_kind` (-1 is the row of a car with
    /// none set). The table is indexed by dwords from the unset row on, so a column past 15 reads
    /// the next row; past the last row it reads whatever follows the table, which is refused.
    pub(super) fn wear_figure_entry(&self, tyre_kind: u32, column: u32) -> Result<u32, BadInput> {
        let rows = (tyre_kind as i32).wrapping_add(1);
        if !(0..=6).contains(&rows) {
            return Err(BadInput("a tyre kind outside the wear table"));
        }
        let at = rows as u32 * 16 + column;
        let figure = match at {
            0..16 => self.wear_unset_row.get(at as usize),
            16..112 => self.wear_table[(at as usize - 16) / 16].get(at as usize % 16),
            _ => None,
        };
        figure
            .map(PlainDword::get)
            .ok_or(BadInput("a wear table column past the table"))
    }
}

/// First values the image's.
impl DamageModel {
    pub const fn new() -> Self {
        DamageModel {
            chance_scale: PlainFloat::new(65536.0),
            light_count_divisor: PlainFloat::new(6.0),
            grade_high: PlainFloat::new(1.0),
            gauge_top: PlainFloat::new(100.0),
            gauge_low: PlainFloat::new(0.0),
            dive_top: PlainFloat::new(8.0),
            fixed_point_scale: PlainFloat::new(256.0),
            half: PlainFloat::new(0.5),
            quarter: PlainFloat::new(0.25),
            tenth: PlainFloat::new(0.1),
            scratch: PlainFloat::new(0.0),
            hundredth: PlainFloat::new(0.01),
            speed_range: PlainFloat::new(70.0),
            engine_limit_factor: PlainFloat::new(0.5),
            metres_per_mile: PlainFloat::new(1609.3457),
            grade_k: PlainFloat::new(0.01),
            gauge_step: PlainFloat::new(0.000167),
            wear_step: PlainFloat::new(0.01),
            hit_divisor: PlainFloat::new(45.0),
            engine_speed_share: PlainFloat::new(0.0),
            speed_share: PlainFloat::new(0.0),
            drive_a_slip_share: PlainFloat::new(0.0),
            drive_b_slip_share: PlainFloat::new(0.0),
            drive_a_extrapolated_share: PlainFloat::new(0.0),
            drive_b_extrapolated_share: PlainFloat::new(0.0),
            brake_share: PlainFloat::new(0.0),
            on_surface_5: PlainFloat::new(0.0),
            gear_changed: PlainInt::new(0),
            gear: PlainInt::new(0),
            engine_over_limit: PlainFloat::new(0.0),
            hit_side: PlainDword::new(0),
            hit_size: PlainFloat::new(0.0),
            wheel_loads: [const { PlainFloat::new(0.0) }; 4],
            wheel_load_mean: PlainFloat::new(0.0),
            point_surface_copy: PlainFloat::new(0.0),
            drive_kind: PlainDword::new(0),
            tyre_kind: PlainDword::new(0),
            surface: PlainDword::new(0),
            surface_blend: PlainFloat::new(0.0),
            wheels_down: PlainFloat::new(0.0),
            ride_height_share: PlainFloat::new(50.0),
            stiffness_share: PlainFloat::new(50.0),
            brake_balance_share: PlainFloat::new(50.0),
            electrics_fail_chance: PlainFloat::new(0.0),
            light_fail_chance_0: PlainFloat::new(0.0),
            light_fail_chance_1: PlainFloat::new(0.0),
            light_fail_chance_2: PlainFloat::new(0.0),
            light_fail_chance_3: PlainFloat::new(0.0),
            light_fail_chance_4: PlainFloat::new(0.0),
            light_fail_chance_5: PlainFloat::new(0.0),
            electrics_recover_chance: PlainFloat::new(0.0),
            light_recover_chance_0: PlainFloat::new(0.0),
            light_recover_chance_1: PlainFloat::new(0.0),
            light_recover_chance_2: PlainFloat::new(0.0),
            light_recover_chance_3: PlainFloat::new(0.0),
            light_recover_chance_4: PlainFloat::new(0.0),
            light_recover_chance_5: PlainFloat::new(0.0),
            brake_balance_complement: PlainFloat::new(0.0),
            front_tyre_wear: PlainFloat::new(0.0),
            rear_tyre_wear: PlainFloat::new(0.0),
            stiffness_suspension_factor: PlainFloat::new(0.0),
            stiffness_body_factor: PlainFloat::new(0.0),
            roughness_wear_base: PlainFloat::new(0.0),
            body_roughness_wear: PlainFloat::new(0.0),
            suspension_roughness_wear: PlainFloat::new(0.0),
            engine_speed_wear_factor: PlainFloat::new(0.0),
            cooling_capacity: PlainFloat::new(0.0),
            overheat_excess: PlainFloat::new(0.0),
            ride_height_body_factor: PlainFloat::new(0.0),
            front_slip_wear_factor: PlainFloat::new(0.0),
            rear_slip_wear_factor: PlainFloat::new(0.0),
            wheel_stiffness_lerp: [const { PlainFloat::new(0.0) }; 4],
            wear_figure: PlainFloat::new(0.0),
            front_tyre_wear_rate: PlainFloat::new(0.0),
            rear_tyre_wear_rate: PlainFloat::new(0.0),
            clutch_block_chance: PlainFloat::new(0.0),
            gearbox_block_chance: PlainFloat::new(0.0),
            gearbox_neutral_chance: PlainFloat::new(0.0),
            handbrake_fail_chance: PlainFloat::new(0.0),
            clock: PlainDword::new(0),
            lerp_tables: [
                [
                    PlainFloat::new(80.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(100.0),
                ],
                [
                    PlainFloat::new(50.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(50.0),
                ],
                [
                    PlainFloat::new(50.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(100.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(1.0),
                    PlainFloat::new(0.2),
                    PlainFloat::new(3.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(95.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(100.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(80.0),
                ],
                [
                    PlainFloat::new(50.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(0.3),
                ],
                [
                    PlainFloat::new(45.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(0.01),
                ],
                [
                    PlainFloat::new(70.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(0.005),
                ],
                [
                    PlainFloat::new(50.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(0.003),
                ],
                [
                    PlainFloat::new(85.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(0.005),
                ],
                [
                    PlainFloat::new(35.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(0.05),
                ],
                [
                    PlainFloat::new(35.0),
                    PlainFloat::new(0.05),
                    PlainFloat::new(100.0),
                    PlainFloat::new(0.0),
                ],
            ],
            wear_rates: [
                PlainFloat::new(0.2),
                PlainFloat::new(1.0),
                PlainFloat::new(0.02),
                PlainFloat::new(2.0),
                PlainFloat::new(0.8),
                PlainFloat::new(5.0),
                PlainFloat::new(0.01),
                PlainFloat::new(100.0),
                PlainFloat::new(3.0),
                PlainFloat::new(7.0),
                PlainFloat::new(0.6),
                PlainFloat::new(0.009),
                PlainFloat::new(0.001),
            ],
            gear_factors: [
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.8),
                PlainFloat::new(1.2),
                PlainFloat::new(0.6),
                PlainFloat::new(0.3),
                PlainFloat::new(0.15),
                PlainFloat::new(0.08),
                PlainFloat::new(0.06),
            ],
            gauge_rates: [
                PlainFloat::new(0.075),
                PlainFloat::new(0.15),
                PlainFloat::new(0.075),
                PlainFloat::new(0.075),
                PlainFloat::new(0.125),
                PlainFloat::new(0.5),
                PlainFloat::new(0.25),
                PlainFloat::new(0.25),
                PlainFloat::new(0.1),
                PlainFloat::new(35.0),
                PlainFloat::new(50.0),
                PlainFloat::new(25.0),
                PlainFloat::new(0.0003),
                PlainFloat::new(0.0002),
                PlainFloat::new(0.0005),
                PlainFloat::new(0.005),
                PlainFloat::new(0.02),
                PlainFloat::new(0.2),
                PlainFloat::new(0.008),
                PlainFloat::new(0.15),
                PlainFloat::new(4.0),
                PlainFloat::new(0.05),
                PlainFloat::new(0.02),
                PlainFloat::new(0.005),
            ],
            kind_table_a: [PlainFloat::new(1.2), PlainFloat::new(1.0)],
            kind_table_b: [PlainFloat::new(1.0), PlainFloat::new(1.0)],
            lerp_tables_b: [
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.5),
                    PlainFloat::new(100.0),
                    PlainFloat::new(1.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(1.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(0.5),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(1.0),
                    PlainFloat::new(100.0),
                    PlainFloat::new(0.5),
                ],
            ],
            wheel_roughness_rate: PlainFloat::new(0.15),
            figures: [
                PlainFloat::new(0.125),
                PlainFloat::new(0.05),
                PlainFloat::new(0.08),
                PlainFloat::new(0.06),
                PlainFloat::new(0.3),
                PlainFloat::new(0.08),
                PlainFloat::new(0.32),
                PlainFloat::new(0.6),
                PlainFloat::new(0.2),
                PlainFloat::new(0.3),
                PlainFloat::new(1.0),
                PlainFloat::new(0.4),
                PlainFloat::new(0.2),
                PlainFloat::new(8.0),
                PlainFloat::new(6.0),
                PlainFloat::new(7.0),
                PlainFloat::new(3.0),
                PlainFloat::new(12.0),
                PlainFloat::new(15.0),
                PlainFloat::new(10.0),
                PlainFloat::new(10.0),
                PlainFloat::new(25.0),
                PlainFloat::new(15.0),
                PlainFloat::new(16.0),
                PlainFloat::new(80.0),
                PlainFloat::new(40.0),
                PlainFloat::new(30.0),
                PlainFloat::new(50.0),
                PlainFloat::new(40.0),
                PlainFloat::new(20.0),
                PlainFloat::new(10.0),
                PlainFloat::new(5.0),
                PlainFloat::new(0.015),
            ],
            wear_unset_row: [
                PlainDword::new(100),
                PlainDword::new(20),
                PlainDword::new(40),
                PlainDword::new(20),
                PlainDword::new(35),
                PlainDword::new(15),
                PlainDword::new(30),
                PlainDword::new(60),
                PlainDword::new(5),
                PlainDword::new(100),
                PlainDword::new(100),
                PlainDword::new(100),
                PlainDword::new(100),
                PlainDword::new(100),
                PlainDword::new(100),
                PlainDword::new(100),
            ],
            wear_table: [
                [
                    PlainDword::new(100),
                    PlainDword::new(20),
                    PlainDword::new(40),
                    PlainDword::new(10),
                    PlainDword::new(15),
                    PlainDword::new(15),
                    PlainDword::new(30),
                    PlainDword::new(60),
                    PlainDword::new(5),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                ],
                [
                    PlainDword::new(100),
                    PlainDword::new(20),
                    PlainDword::new(35),
                    PlainDword::new(8),
                    PlainDword::new(13),
                    PlainDword::new(12),
                    PlainDword::new(25),
                    PlainDword::new(50),
                    PlainDword::new(4),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                ],
                [
                    PlainDword::new(100),
                    PlainDword::new(18),
                    PlainDword::new(40),
                    PlainDword::new(6),
                    PlainDword::new(10),
                    PlainDword::new(10),
                    PlainDword::new(20),
                    PlainDword::new(40),
                    PlainDword::new(3),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                ],
                [
                    PlainDword::new(100),
                    PlainDword::new(15),
                    PlainDword::new(35),
                    PlainDword::new(28),
                    PlainDword::new(35),
                    PlainDword::new(15),
                    PlainDword::new(30),
                    PlainDword::new(60),
                    PlainDword::new(8),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                ],
                [
                    PlainDword::new(100),
                    PlainDword::new(12),
                    PlainDword::new(30),
                    PlainDword::new(22),
                    PlainDword::new(28),
                    PlainDword::new(15),
                    PlainDword::new(25),
                    PlainDword::new(50),
                    PlainDword::new(7),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                ],
                [
                    PlainDword::new(100),
                    PlainDword::new(12),
                    PlainDword::new(18),
                    PlainDword::new(10),
                    PlainDword::new(12),
                    PlainDword::new(10),
                    PlainDword::new(15),
                    PlainDword::new(40),
                    PlainDword::new(3),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                    PlainDword::new(100),
                ],
            ],
            wear_figure_scale: PlainFloat::new(2.5),
            wear_columns: [
                [PlainDword::new(0), PlainDword::new(0)],
                [PlainDword::new(1), PlainDword::new(2)],
                [PlainDword::new(3), PlainDword::new(4)],
                [PlainDword::new(3), PlainDword::new(4)],
                [PlainDword::new(7), PlainDword::new(7)],
                [PlainDword::new(7), PlainDword::new(7)],
                [PlainDword::new(5), PlainDword::new(6)],
                [PlainDword::new(5), PlainDword::new(6)],
                [PlainDword::new(8), PlainDword::new(8)],
                [PlainDword::new(7), PlainDword::new(7)],
                [PlainDword::new(3), PlainDword::new(4)],
                [PlainDword::new(7), PlainDword::new(7)],
                [PlainDword::new(7), PlainDword::new(7)],
                [PlainDword::new(0), PlainDword::new(0)],
                [PlainDword::new(0), PlainDword::new(0)],
                [PlainDword::new(0), PlainDword::new(0)],
            ],
        }
    }
}

/// The offset (from the image's start) at which the recorded vectors of the damage tests place a
/// figure: the recorded layout's, less the four unreferenced bytes the struct has dropped before
/// it. The vectors' buffers hold the struct, so the places move with it.
#[cfg(test)]
pub const fn laid_offset(image_offset: usize) -> usize {
    /// The image offsets of the dropped single-dword gaps.
    const DROPPED: [usize; 5] = [0x35B358, 0x35B374, 0x35B384, 0x35B4A4, 0x35B674];
    let mut dropped = 0;
    let mut i = 0;
    while i < DROPPED.len() {
        assert!(image_offset != DROPPED[i], "a dropped gap has no place");
        if image_offset > DROPPED[i] {
            dropped += 4;
        }
        i += 1;
    }
    image_offset - dropped
}

/// The damage model as a block vector's test buffer holds it: the buffer at `base` holds the
/// image's bytes from its start, the model at its place in the image. Buffer operands only (a race
/// block operand's `base` is 0).
#[cfg(test)]
pub(crate) fn model_in_test_buffer(base: usize) -> &'static DamageModel {
    use crate::test_support::vectors::block_vectors::{LAID_ROOM, at};
    const { assert!(size_of::<DamageModel>() <= LAID_ROOM) };
    assert!(base != 0, "the damage model in a race block operand");
    let model = at(base + 0x35B340).cast::<DamageModel>();
    assert!(model.is_aligned(), "the damage model on an odd place");
    // SAFETY: the buffer holds every place the block lists and `LAID_ROOM` bytes past the last
    // (`block_vectors`), and the model starts at or below its first listed place, so all of its
    // `size_of` bytes (at most `LAID_ROOM`, asserted above) lie in the buffer; aligned (checked);
    // cells over integers only, so any bytes are a valid model. The buffer lives through the
    // block's run, the one user of the reference (it is not kept past it).
    unsafe { &*model }
}

/// `laid_offset(..)` as a constant, for the vectors' `&'static` place lists.
#[cfg(test)]
macro_rules! laid {
    ($image_offset:expr) => {
        const { $crate::physics::store::laid_offset($image_offset) }
    };
}
#[cfg(test)]
pub(crate) use laid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wear_table_reads_on_as_the_original_indexed_it() {
        let m = DamageModel::new();
        // The unset row, then the six rows, as one run of 112 dwords.
        let flat: Vec<u32> = m
            .wear_unset_row
            .iter()
            .chain(m.wear_table.iter().flatten())
            .map(PlainDword::get)
            .collect();
        assert_eq!(flat.len(), 112);
        for kind in [u32::MAX, 0, 1, 2, 3, 4, 5] {
            for column in 0..40u32 {
                let at = (kind.wrapping_add(1) * 16 + column) as usize;
                match flat.get(at) {
                    Some(&figure) => assert_eq!(m.wear_figure_entry(kind, column), Ok(figure)),
                    None => assert!(m.wear_figure_entry(kind, column).is_err()),
                }
            }
        }
        assert!(m.wear_figure_entry(6, 0).is_err());
        assert!(m.wear_figure_entry(u32::MAX - 1, 0).is_err());
        // The shipped columns all stay within the row.
        assert!(m.wear_columns.iter().flatten().all(|c| c.get() < 16));
    }
}
