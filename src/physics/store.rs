//! The damage model's figures: shares, thresholds, the work floats, the tables of four (x0, y0, x1,
//! y1) `physics::flow::table_lerp` reads, and the rates. Plain fields, one record in the physics owner. What each figure
//! is, is a reading from how `physics::damage_factors`, `damage_wear`, `damage_gauges` and
//! `rate_damage` use it (`physics/damage_model.rs`); the doc of each says what.

use crate::errors::BadInput;

/// The damage model's figures.
#[cfg_attr(test, repr(C))]
pub struct DamageModel {
    /// A figure times it gives the failure chance (`physics::rate_damage`).
    pub chance_scale: f32,
    /// 6.0, the number of lights: the six light gauges' sum is divided by it for their mean
    /// (`lights_damage`).
    pub light_count_divisor: f32,
    /// The working grades are held up to it.
    pub grade_high: f32,
    /// The gauges and gears are held 0 ..= it; a gear equal to it is worn out.
    pub gauge_top: f32,
    /// The gauges and grades are held down to it.
    pub gauge_low: f32,
    /// 8.0, the highest landing dive figure counted (`physics::damage_factors` holds the dive 0 ..=
    /// it before squaring).
    pub dive_top: f32,
    /// The divisor (256) that turns the car block's whole-number figures into shares.
    pub fixed_point_scale: f32,
    /// 0.5: the average of two (brake wear, tyre wear, gearbox, the wheel loads' blend).
    pub half: f32,
    /// 0.25: the average of four (the wheel stiffness / ride-height means, the wheel loads).
    pub quarter: f32,
    /// 0.1: the body damage gauge is the sum of its parts times it.
    pub tenth: f32,
    /// The work cell a rounding to a whole number goes through.
    pub scratch: f32,
    /// 0.01: the percent factor the wear and hit terms are multiplied by.
    pub hundredth: f32,
    /// 70.0: the forward speed (and speed gaps) are held to it and divided by it, then scaled to
    /// `gauge_top`, so a speed share runs 0 ..= 100.
    pub speed_range: f32,
    /// 0.5: the engine counts as over its limit when its speed is above `engine_speed_limit` times
    /// it.
    pub engine_limit_factor: f32,
    /// 1609.34, the metres in a mile: the tyre durability (`wear_figure`) is turned to this unit
    /// before the speed share is divided by it (probably, the durability figures being miles).
    pub metres_per_mile: f32,
    /// The factor `physics::rate_damage` multiplies by (80 bits).
    pub grade_k: f32,
    /// About 1/6000 (0.000167): the gauge rates are multiplied by it, so a gauge rises by its rate
    /// times this per step.
    pub gauge_step: f32,
    /// The step the wear rates are multiplied by (`physics::damage_wear`).
    pub wear_step: f32,
    /// 45.0: the hit size is the contact depth squared over it and over `speed_range`.
    pub hit_divisor: f32,
    /// The engine speed over its limit, scaled to `gauge_top`.
    pub engine_speed_share: f32,
    /// The forward speed over `speed_range`, scaled to `gauge_top`.
    pub speed_share: f32,
    /// The gap between the forward speed and drive speed A over `speed_range`.
    pub drive_a_slip_share: f32,
    /// The gap between the forward speed and drive speed B over `speed_range`.
    pub drive_b_slip_share: f32,
    /// Drive speed A pushed past the forward speed, over `speed_range`.
    pub drive_a_extrapolated_share: f32,
    /// Drive speed B pushed past the forward speed, over `speed_range`.
    pub drive_b_extrapolated_share: f32,
    /// The brake figure over `fixed_point_scale` while the car moves and the brake is 0x10 or more.
    pub brake_share: f32,
    /// 1 while the car's centre point is on surface kind 5 (wet, as `wheel_surface_a` says), else
    /// 0: it wets the brakes (fade) and damages the electrics and the lights.
    pub on_surface_5: f32,
    /// Copy of the car's `gear_changed`: 1 after a shift up, -1 after a shift down, else 0 (the
    /// gauges add the clutch and gear wear of a shift).
    pub gear_changed: i32,
    /// Copy of the car's `gear`: -1 (reverse), 0 (neutral), 1 ..= 7; picks the gear's wear gauge
    /// and factor.
    pub gear: i32,
    /// 1.0 when the engine speed is above `engine_speed_limit` times `engine_limit_factor`, else 0:
    /// the turbo wear while over the limit.
    pub engine_over_limit: f32,
    /// Where the car was hit: 0 none, else 1 ..= 4 by the side of the contact point (1 + 1 if the
    /// point's x is above 0 + 2 if its z is 0 or below).
    pub hit_side: u32,
    /// The size of this step's hit: the contact depth squared over `hit_divisor` over
    /// `speed_range`; scaled by the damage setting (`patch_factors[1]`) in the gauges, zeroed while
    /// the hit cooldown runs.
    pub hit_size: f32,
    /// The four wheels' loads, blended by `DamageModel.half` (`physics::damage_factors`).
    pub wheel_loads: [f32; 4],
    /// The mean of the four wheel loads (their sum times `quarter`).
    pub wheel_load_mean: f32,
    /// Copy of the centre point's surface figure (the surface's wear figure blended between its two
    /// cells): how hard the ground is on the car; it scales the roughness wear. Probably a
    /// roughness.
    pub point_surface_copy: f32,
    /// Copy of the car's `drive_kind` (a whole number): picks the entry of the two kind tables (the
    /// kind must be 0 or 1 there), and a car with 0 has no turbo damage.
    pub drive_kind: u32,
    /// Copy of the car's `tyre_kind`: the row of `wear_table` (16 dwords a row).
    pub tyre_kind: u32,
    /// Copy of the centre point's surface kind (`wheel_surface_a[4]`): its low four bits pick the
    /// wear table's columns.
    pub surface: u32,
    /// Copy of the car's `surface_blend` (a float, 0 ..= 1, held in a dword cell): the weight of
    /// the second surface's column against the first when `wear_figure` is blended.
    pub surface_blend: f32,
    /// 1 when at least one wheel is not off the ground, else 0.
    pub wheels_down: f32,
    /// The car's ride-height figure over `fixed_point_scale`.
    pub ride_height_share: f32,
    /// The car's suspension-stiffness figure over `fixed_point_scale`.
    pub stiffness_share: f32,
    /// The car's brake-balance figure over `fixed_point_scale`.
    pub brake_balance_share: f32,
    /// The chance the electrics fail this step (ramp of `electrics_damage`, table 11),
    /// `electrical_failures[0]` set by it (`physics::rate_damage`).
    pub electrics_fail_chance: f32,
    /// The chance light 0 (`light_damage[0]`, table 11) goes out this step,
    /// `electrical_failures[1 + 0]` set by it.
    pub light_fail_chance_0: f32,
    /// The chance light 1 (`light_damage[1]`, table 11) goes out this step,
    /// `electrical_failures[1 + 1]` set by it.
    pub light_fail_chance_1: f32,
    /// The chance light 2 (`light_damage[2]`, table 11) goes out this step,
    /// `electrical_failures[1 + 2]` set by it.
    pub light_fail_chance_2: f32,
    /// The chance light 3 (`light_damage[3]`, table 11) goes out this step,
    /// `electrical_failures[1 + 3]` set by it.
    pub light_fail_chance_3: f32,
    /// The chance light 4 (`light_damage[4]`, table 11) goes out this step,
    /// `electrical_failures[1 + 4]` set by it.
    pub light_fail_chance_4: f32,
    /// The chance light 5 (`light_damage[5]`, table 11) goes out this step,
    /// `electrical_failures[1 + 5]` set by it.
    pub light_fail_chance_5: f32,
    /// The chance a failed electrics recovers this step (ramp of `electrics_damage`, table 11).
    pub electrics_recover_chance: f32,
    /// The chance a failed light 0 comes back this step (ramp of `light_damage[0]`, table 12).
    pub light_recover_chance_0: f32,
    /// The chance a failed light 1 comes back this step (ramp of `light_damage[1]`, table 12).
    pub light_recover_chance_1: f32,
    /// The chance a failed light 2 comes back this step (ramp of `light_damage[2]`, table 12).
    pub light_recover_chance_2: f32,
    /// The chance a failed light 3 comes back this step (ramp of `light_damage[3]`, table 12).
    pub light_recover_chance_3: f32,
    /// The chance a failed light 4 comes back this step (ramp of `light_damage[4]`, table 12).
    pub light_recover_chance_4: f32,
    /// The chance a failed light 5 comes back this step (ramp of `light_damage[5]`, table 12).
    pub light_recover_chance_5: f32,
    /// 1 - `brake_balance_share`, the other axle's brake share: `brake_wear_b` rises by its rate
    /// times this. `physics::rate_damage` reuses the cell as the scratch of the drive-efficiency
    /// ramp.
    pub brake_balance_complement: f32,
    /// The front tyres' wear, the mean of `tyre_wears` 0 and 1 (FL, FR; "Front tyres" on the
    /// developers' wear page): the input of the drive-efficiency ramp for drive speed A, 0 ..= 100.
    pub front_tyre_wear: f32,
    /// The rear tyres' wear, the mean of `tyre_wears` 2 and 3 (RL, RR; "Rear tyres"): as
    /// `front_tyre_wear` for drive speed B.
    pub rear_tyre_wear: f32,
    /// The stiffness mean through `lerp_tables_b[0]`, squared: scales the suspension's roughness
    /// wear (`suspension_roughness_wear`).
    pub stiffness_suspension_factor: f32,
    /// The stiffness mean through `lerp_tables_b[2]`, squared: scales the body roughness wear.
    pub stiffness_body_factor: f32,
    /// Surface figure times the speed share times `figures[32]` times `wheels_down`: the
    /// rough-ground wear before any part's factor; it wears the wheel hit gauges (times
    /// `wheel_roughness_rate`).
    pub roughness_wear_base: f32,
    /// `roughness_wear_base` times `stiffness_body_factor` and `ride_height_body_factor`: the rate
    /// for the body, steering, engine, exhaust, gears and differential gauges.
    pub body_roughness_wear: f32,
    /// `roughness_wear_base` times `stiffness_suspension_factor`: the rate for the four suspension
    /// gauges.
    pub suspension_roughness_wear: f32,
    /// The engine speed share through `lerp_tables[0]`: the rate of the engine and cooling gauges'
    /// wear from revving.
    pub engine_speed_wear_factor: f32,
    /// The cooling damage through `lerp_tables[1]`: the engine speed share the cooling can still
    /// carry off.
    pub cooling_capacity: f32,
    /// Engine speed share minus `cooling_capacity`, held at 0 or more: the heat the cooling cannot
    /// carry; raises the engine heat and, when above 0, the engine and cooling damage.
    pub overheat_excess: f32,
    /// The ride-height mean through `lerp_tables_b[1]`, squared: scales the body roughness wear.
    pub ride_height_body_factor: f32,
    /// The drive-A slip share through `lerp_tables[3]`: how much slip adds to the front tyres'
    /// wear.
    pub front_slip_wear_factor: f32,
    /// The drive-B slip share through `lerp_tables[3]`: the same for tyre gauges 2 and 3.
    pub rear_slip_wear_factor: f32,
    /// The four wheels' table lerps of the suspension damage (`physics::damage_factors`).
    pub wheel_stiffness_lerp: [f32; 4],
    /// The tyre durability on this ground: the two columns of `wear_table` (whole numbers, e.g.
    /// 100, 20, 60, 5) blended by `surface_blend`, times `wear_figure_scale`; the tyre wear rates
    /// divide by it, so a bigger figure wears slower.
    pub wear_figure: f32,
    /// The speed share over (`wear_figure` times `metres_per_mile`) times `gauge_top` times
    /// `wheels_down`, times the kind table A entry: the speed part of the front tyres' wear rate.
    pub front_tyre_wear_rate: f32,
    /// As `front_tyre_wear_rate`, times the kind table B entry: the speed part of the rear tyres'
    /// wear rate.
    pub rear_tyre_wear_rate: f32,
    /// The chance gear changes are blocked this step (ramp of `clutch_damage`, table 7; sets
    /// `gear_change_blocked`).
    pub clutch_block_chance: f32,
    /// The chance gear changes are blocked (ramp of `gearbox_damage`, table 8; sets
    /// `gear_change_blocked`).
    pub gearbox_block_chance: f32,
    /// The chance the gearbox is forced into neutral (ramp of `gearbox_damage`, table 9; sets
    /// `forced_neutral`).
    pub gearbox_neutral_chance: f32,
    /// The chance the handbrake fails for good (ramp of `brake_damage`, table 10; sets
    /// `handbrake_failed`).
    pub handbrake_fail_chance: f32,
    /// The clock / random figure the failures by chance compare against.
    pub clock: u32,
    /// The tables of four (x0, y0, x1, y1) (`physics::damage_wear`, `rate_damage`,
    /// `damage_factors`).
    pub lerp_tables: [[f32; 4]; 13],
    /// The wear rates `physics::damage_wear` raises and lowers the gauges by.
    pub wear_rates: [f32; 13],
    /// The gear factors by gear + 1 (reverse, neutral, 1 .. 7), `physics::damage_gauges`; index 1
    /// is the neutral entry, read when the gear is 0 like the others.
    pub gear_factors: [f32; 9],
    /// The gauges' rates (`physics::damage_gauges`), times the step's shares.
    pub gauge_rates: [f32; 24],
    /// The first kind table (by `DamageModel.drive_kind`'s bits).
    pub kind_table_a: [f32; 2],
    /// The second kind table.
    pub kind_table_b: [f32; 2],
    /// Three more tables of four.
    pub lerp_tables_b: [[f32; 4]; 3],
    /// 0.15: the rate of the wheel hit gauges' wear by rough ground (times `roughness_wear_base`).
    pub wheel_roughness_rate: f32,
    /// More hit figures and thresholds (`physics::damage_gauges`, `rate_damage`).
    pub figures: [f32; 33],
    /// The row of figures a car with no tyre kind set (`tyre_kind` -1) gets; one row before
    /// `wear_table`, as the index runs.
    pub wear_unset_row: [u32; 16],
    /// The wear table, a row of 16 whole numbers per `DamageModel.tyre_kind` 0 ..= 5.
    pub wear_table: [[u32; 16]; 6],
    /// 2.5: `wear_figure` is the blended table figure times it.
    pub wear_figure_scale: f32,
    /// The wear table's two columns by surface kind (its low four bits): the first and the second
    /// column of the pair, whole numbers.
    pub wear_columns: [[u32; 2]; 16],
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
            .copied()
            .ok_or(BadInput("a wear table column past the table"))
    }
}

/// The starting values of `wear_rates`.
#[rustfmt::skip]
const WEAR_RATES: [f32; 13] = [
    0.2, 1.0, 0.02, 2.0, 0.8, 5.0, 0.01, 100.0, 3.0, 7.0, 0.6, 0.009, 0.001,
];

/// The starting values of `gear_factors`.
#[rustfmt::skip]
const GEAR_FACTORS: [f32; 9] = [
    1.0, 0.0, 1.8, 1.2, 0.6, 0.3, 0.15, 0.08, 0.06,
];

/// The starting values of `gauge_rates`.
#[rustfmt::skip]
const GAUGE_RATES: [f32; 24] = [
    0.075, 0.15, 0.075, 0.075, 0.125, 0.5, 0.25, 0.25, 0.1, 35.0, 50.0, 25.0, 0.0003, 0.0002,
    0.0005, 0.005, 0.02, 0.2, 0.008, 0.15, 4.0, 0.05, 0.02, 0.005,
];

/// The starting values of `kind_table_a`.
#[rustfmt::skip]
const KIND_TABLE_A: [f32; 2] = [
    1.2, 1.0,
];

/// The starting values of `kind_table_b`.
#[rustfmt::skip]
const KIND_TABLE_B: [f32; 2] = [
    1.0, 1.0,
];

/// The starting values of `figures`.
#[rustfmt::skip]
const FIGURES: [f32; 33] = [
    0.125, 0.05, 0.08, 0.06, 0.3, 0.08, 0.32, 0.6, 0.2, 0.3, 1.0, 0.4, 0.2, 8.0, 6.0, 7.0, 3.0,
    12.0, 15.0, 10.0, 10.0, 25.0, 15.0, 16.0, 80.0, 40.0, 30.0, 50.0, 40.0, 20.0, 10.0, 5.0, 0.015,
];

/// The starting values of `wear_unset_row`.
#[rustfmt::skip]
const WEAR_UNSET_ROW: [u32; 16] = [
    100, 20, 40, 20, 35, 15, 30, 60, 5, 100, 100, 100, 100, 100, 100, 100,
];

/// First values the image's.
impl DamageModel {
    pub const fn new() -> Self {
        DamageModel {
            chance_scale: 65536.0,
            light_count_divisor: 6.0,
            grade_high: 1.0,
            gauge_top: 100.0,
            gauge_low: 0.0,
            dive_top: 8.0,
            fixed_point_scale: 256.0,
            half: 0.5,
            quarter: 0.25,
            tenth: 0.1,
            scratch: 0.0,
            hundredth: 0.01,
            speed_range: 70.0,
            engine_limit_factor: 0.5,
            metres_per_mile: 1609.3457,
            grade_k: 0.01,
            gauge_step: 0.000167,
            wear_step: 0.01,
            hit_divisor: 45.0,
            engine_speed_share: 0.0,
            speed_share: 0.0,
            drive_a_slip_share: 0.0,
            drive_b_slip_share: 0.0,
            drive_a_extrapolated_share: 0.0,
            drive_b_extrapolated_share: 0.0,
            brake_share: 0.0,
            on_surface_5: 0.0,
            gear_changed: 0,
            gear: 0,
            engine_over_limit: 0.0,
            hit_side: 0,
            hit_size: 0.0,
            wheel_loads: [0.0; 4],
            wheel_load_mean: 0.0,
            point_surface_copy: 0.0,
            drive_kind: 0,
            tyre_kind: 0,
            surface: 0,
            surface_blend: 0.0,
            wheels_down: 0.0,
            ride_height_share: 50.0,
            stiffness_share: 50.0,
            brake_balance_share: 50.0,
            electrics_fail_chance: 0.0,
            light_fail_chance_0: 0.0,
            light_fail_chance_1: 0.0,
            light_fail_chance_2: 0.0,
            light_fail_chance_3: 0.0,
            light_fail_chance_4: 0.0,
            light_fail_chance_5: 0.0,
            electrics_recover_chance: 0.0,
            light_recover_chance_0: 0.0,
            light_recover_chance_1: 0.0,
            light_recover_chance_2: 0.0,
            light_recover_chance_3: 0.0,
            light_recover_chance_4: 0.0,
            light_recover_chance_5: 0.0,
            brake_balance_complement: 0.0,
            front_tyre_wear: 0.0,
            rear_tyre_wear: 0.0,
            stiffness_suspension_factor: 0.0,
            stiffness_body_factor: 0.0,
            roughness_wear_base: 0.0,
            body_roughness_wear: 0.0,
            suspension_roughness_wear: 0.0,
            engine_speed_wear_factor: 0.0,
            cooling_capacity: 0.0,
            overheat_excess: 0.0,
            ride_height_body_factor: 0.0,
            front_slip_wear_factor: 0.0,
            rear_slip_wear_factor: 0.0,
            wheel_stiffness_lerp: [0.0; 4],
            wear_figure: 0.0,
            front_tyre_wear_rate: 0.0,
            rear_tyre_wear_rate: 0.0,
            clutch_block_chance: 0.0,
            gearbox_block_chance: 0.0,
            gearbox_neutral_chance: 0.0,
            handbrake_fail_chance: 0.0,
            clock: 0,
            lerp_tables: [
                [80.0, 0.0, 100.0, 100.0],
                [50.0, 100.0, 100.0, 50.0],
                [50.0, 0.0, 100.0, 100.0],
                [0.0, 1.0, 0.2, 3.0],
                [0.0, 95.0, 100.0, 100.0],
                [0.0, 100.0, 100.0, 80.0],
                [50.0, 0.0, 100.0, 0.3],
                [45.0, 0.0, 100.0, 0.01],
                [70.0, 0.0, 100.0, 0.005],
                [50.0, 0.0, 100.0, 0.003],
                [85.0, 0.0, 100.0, 0.005],
                [35.0, 0.0, 100.0, 0.05],
                [35.0, 0.05, 100.0, 0.0],
            ],
            wear_rates: WEAR_RATES,
            gear_factors: GEAR_FACTORS,
            gauge_rates: GAUGE_RATES,
            kind_table_a: KIND_TABLE_A,
            kind_table_b: KIND_TABLE_B,
            lerp_tables_b: [
                [0.0, 0.5, 100.0, 1.0],
                [0.0, 1.0, 100.0, 0.5],
                [0.0, 1.0, 100.0, 0.5],
            ],
            wheel_roughness_rate: 0.15,
            figures: FIGURES,
            wear_unset_row: WEAR_UNSET_ROW,
            wear_table: [
                [
                    100, 20, 40, 10, 15, 15, 30, 60, 5, 100, 100, 100, 100, 100, 100, 100,
                ],
                [
                    100, 20, 35, 8, 13, 12, 25, 50, 4, 100, 100, 100, 100, 100, 100, 100,
                ],
                [
                    100, 18, 40, 6, 10, 10, 20, 40, 3, 100, 100, 100, 100, 100, 100, 100,
                ],
                [
                    100, 15, 35, 28, 35, 15, 30, 60, 8, 100, 100, 100, 100, 100, 100, 100,
                ],
                [
                    100, 12, 30, 22, 28, 15, 25, 50, 7, 100, 100, 100, 100, 100, 100, 100,
                ],
                [
                    100, 12, 18, 10, 12, 10, 15, 40, 3, 100, 100, 100, 100, 100, 100, 100,
                ],
            ],
            wear_figure_scale: 2.5,
            wear_columns: [
                [0, 0],
                [1, 2],
                [3, 4],
                [3, 4],
                [7, 7],
                [7, 7],
                [5, 6],
                [5, 6],
                [8, 8],
                [7, 7],
                [3, 4],
                [7, 7],
                [7, 7],
                [0, 0],
                [0, 0],
                [0, 0],
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
pub(crate) fn model_in_test_buffer(base: usize) -> &'static mut DamageModel {
    use crate::test_support::vectors::block_vectors::{LAID_ROOM, at};
    const { assert!(size_of::<DamageModel>() <= LAID_ROOM) };
    assert!(base != 0, "the damage model in a race block operand");
    let model = at(base + 0x35B340).cast::<DamageModel>();
    assert!(model.is_aligned(), "the damage model on an odd place");
    // SAFETY: the buffer holds every place the block lists and `LAID_ROOM` bytes past the last
    // (`block_vectors`), and the model starts at or below its first listed place, so all of its
    // `size_of` bytes (at most `LAID_ROOM`, asserted above) lie in the buffer; aligned (checked);
    // integers and floats only, so any bytes are a valid model. The buffer lives through the
    // block's run, the one user of the reference (it is not kept past it).
    unsafe { &mut *model }
}

/// The model's dwords in the order of its fields (the layout is `repr(C)` in tests, all of its
/// fields are four bytes).
#[cfg(test)]
pub(crate) fn words_of(model: &DamageModel) -> &[u32] {
    assert_eq!(size_of::<DamageModel>() % 4, 0);
    // SAFETY: `repr(C)` in tests, every field a four-byte integer or float or an array of them, so
    // the record is `size_of / 4` aligned dwords with no padding; the borrow keeps it alive.
    unsafe {
        core::slice::from_raw_parts(
            core::ptr::from_ref(model).cast(),
            size_of::<DamageModel>() / 4,
        )
    }
}

/// `words_of`, to write.
#[cfg(test)]
pub(crate) fn words_of_mut(model: &mut DamageModel) -> &mut [u32] {
    assert_eq!(size_of::<DamageModel>() % 4, 0);
    // SAFETY: as `words_of`; any bits are a valid integer or float, and the exclusive borrow is
    // the only access.
    unsafe {
        core::slice::from_raw_parts_mut(
            core::ptr::from_mut(model).cast(),
            size_of::<DamageModel>() / 4,
        )
    }
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
    /// The record's whole bytes, as a checksum: a wrong dword in a row shows here.
    #[test]
    fn the_damage_model_keeps_the_bytes_of_the_image() {
        let model = DamageModel::new();
        let bytes: Vec<u8> = super::words_of(&model)
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect();
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in &bytes {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
        assert_eq!(bytes.len(), 1516);
        assert_eq!(hash, 0x3064_78b9_a437_1b37);
    }

    #[test]
    fn the_wear_table_reads_on_as_the_original_indexed_it() {
        let m = DamageModel::new();
        // The unset row, then the six rows, as one run of 112 dwords.
        let flat: Vec<u32> = m
            .wear_unset_row
            .iter()
            .chain(m.wear_table.iter().flatten())
            .copied()
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
        assert!(m.wear_columns.iter().flatten().all(|&c| c < 16));
    }
}
