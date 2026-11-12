//! The damage model's figures, out of RAL.EXE's image (IDA 0x75B340 .. 0x75B940; phase 7,
//! typed): shares, thresholds, the work floats, the tables of four (x0, y0, x1, y1)
//! `physics::flow::table_lerp` reads, and the rates. `#[repr(C)]` struct of atomic cells
//! with the image's layout, one static. What each
//! figure is, is a reading from how `physics::damage_factors`, `damage_wear`, `damage_gauges`
//! and `rate_damage` use it (`physics/damage_model.rs`); the doc of each says what.

use crate::session::{PlainBytes, PlainDword, PlainFloat};

/// A gap's first bytes: `runs` are (offset, bytes) of the non-zero ones.
const fn sparse<const N: usize>(runs: &[(usize, &[u8])]) -> [u8; N] {
    let mut out = [0u8; N];
    let mut r = 0;
    while r < runs.len() {
        let (at, bytes) = runs[r];
        let mut i = 0;
        while i < bytes.len() {
            out[at + i] = bytes[i];
            i += 1;
        }
        r += 1;
    }
    out
}

/// The damage model's figures.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DamageModel {
    /// `flt_75B340`: a figure times it gives the failure chance (`physics::rate_damage`).
    pub chance_scale: PlainFloat,
    /// `flt_75B344`: 6.0, the number of lights: the six light gauges' sum is divided by it for their mean (`lights_damage`).
    pub light_count_divisor: PlainFloat,
    /// `flt_75B348`: the working grades are held up to it.
    pub grade_high: PlainFloat,
    /// `flt_75B34C`: the gauges and gears are held 0 ..= it; a gear equal to it is worn out.
    pub gauge_top: PlainFloat,
    /// `flt_75B350`: the gauges and grades are held down to it.
    pub gauge_low: PlainFloat,
    /// `flt_75B354`: 8.0, the highest landing dive figure counted (`physics::damage_factors` holds the dive 0 ..= it before squaring).
    pub dive_top: PlainFloat,
    /// Unreferenced bytes (IDA 0X75B358 .. 0X75B35C).
    pub _gap_75b358: PlainBytes<4>,
    /// `flt_75B35C`: the divisor (256) that turns the car block's whole-number figures into shares.
    pub fixed_point_scale: PlainFloat,
    /// `flt_75B360`: 0.5: the average of two (brake wear, tyre wear, gearbox, the wheel loads' blend).
    pub half: PlainFloat,
    /// `flt_75B364`: 0.25: the average of four (the wheel stiffness / ride-height means, the wheel loads).
    pub quarter: PlainFloat,
    /// `flt_75B368`: 0.1: the body damage gauge is the sum of its parts times it.
    pub tenth: PlainFloat,
    /// `flt_75B36C`: the work cell a `fistp` rounds into.
    pub scratch: PlainFloat,
    /// `flt_75B370`: 0.01: the percent factor the wear and hit terms are multiplied by.
    pub hundredth: PlainFloat,
    /// Unreferenced bytes (IDA 0X75B374 .. 0X75B378).
    pub _gap_75b374: PlainBytes<4>,
    /// `flt_75B378`: 70.0: the forward speed (and speed gaps) are held to it and divided by it, then scaled to `gauge_top`, so a speed share runs 0 ..= 100.
    pub speed_range: PlainFloat,
    /// `flt_75B37C`: 0.5: the engine counts as over its limit when its speed is above `engine_speed_limit` times it.
    pub engine_limit_factor: PlainFloat,
    /// `flt_75B380`: 1609.34, the metres in a mile: the tyre durability (`wear_figure`) is turned to this unit before the speed share is divided by it (probably, the durability figures being miles).
    pub metres_per_mile: PlainFloat,
    /// Unreferenced bytes (IDA 0X75B384 .. 0X75B388).
    pub _gap_75b384: PlainBytes<4>,
    /// `flt_75B388`: the factor `physics::rate_damage` multiplies by (80 bits).
    pub grade_k: PlainFloat,
    /// `flt_75B38C`: About 1/6000 (0.000167): the gauge rates are multiplied by it, so a gauge rises by its rate times this per step.
    pub gauge_step: PlainFloat,
    /// `flt_75B390`: the step the wear rates are multiplied by (`physics::damage_wear`).
    pub wear_step: PlainFloat,
    /// `flt_75B394`: 45.0: the hit size is the contact depth squared over it and over `speed_range`.
    pub hit_divisor: PlainFloat,
    /// `flt_75B398`: the engine speed over its limit, scaled to `gauge_top`.
    pub engine_speed_share: PlainFloat,
    /// `flt_75B39C`: the forward speed over `speed_range`, scaled to `gauge_top`.
    pub speed_share: PlainFloat,
    /// `flt_75B3A0`: the gap between the forward speed and drive speed A over `speed_range`.
    pub drive_a_slip_share: PlainFloat,
    /// `flt_75B3A4`: the gap between the forward speed and drive speed B over `speed_range`.
    pub drive_b_slip_share: PlainFloat,
    /// `flt_75B3A8`: drive speed A pushed past the forward speed, over `speed_range`.
    pub drive_a_extrapolated_share: PlainFloat,
    /// `flt_75B3AC`: drive speed B pushed past the forward speed, over `speed_range`.
    pub drive_b_extrapolated_share: PlainFloat,
    /// `flt_75B3B0`: the brake figure over `fixed_point_scale` while the car moves and the brake is 0x10 or more.
    pub brake_share: PlainFloat,
    /// `flt_75B3B4`: 1 while the car's centre point is on surface kind 5 (wet, as `wheel_surface_a` says), else 0: it wets the brakes (fade) and damages the electrics and the lights.
    pub on_surface_5: PlainFloat,
    /// `dword_75B3B8`: Copy of the car's `gear_changed`: 1 after a shift up, -1 after a shift down, else 0 (the gauges add the clutch and gear wear of a shift).
    pub gear_changed: PlainDword,
    /// `dword_75B3BC`: Copy of the car's `gear`: -1 (reverse), 0 (neutral), 1 ..= 7; picks the gear's wear gauge and factor.
    pub gear: PlainDword,
    /// `flt_75B3C0`: 1.0 when the engine speed is above `engine_speed_limit` times `engine_limit_factor`, else 0: the turbo wear while over the limit.
    pub engine_over_limit: PlainFloat,
    /// `dword_75B3C4`: Where the car was hit: 0 none, else 1 ..= 4 by the side of the contact point (1 + 1 if the point's x is above 0 + 2 if its z is 0 or below).
    pub hit_side: PlainDword,
    /// `flt_75B3C8`: The size of this step's hit: the contact depth squared over `hit_divisor` over `speed_range`; scaled by the damage setting (`patch_factors[1]`) in the gauges, zeroed while the hit cooldown runs.
    pub hit_size: PlainFloat,
    /// `dword_75B3CC` ..: the four wheels' loads, blended by `DamageModel.half` (`physics::damage_factors`).
    pub wheel_loads: [PlainFloat; 4],
    /// `flt_75B3DC`: The mean of the four wheel loads (their sum times `quarter`).
    pub wheel_load_mean: PlainFloat,
    /// `flt_75B3E0`: Copy of the centre point's surface figure (`point_surface_2c[4]`, the surface's +0x2c value blended between its two cells): how hard the ground is on the car; it scales the roughness wear. Probably a roughness.
    pub point_surface_copy: PlainFloat,
    /// `flt_75B3E4`: Copy of the car's `drive_kind` (0 or 1, as a whole number in a float cell): picks the entry of the two kind tables, and a car with 0 has no turbo damage.
    pub drive_kind: PlainFloat,
    /// `dword_75B3E8`: Copy of the car's `tyre_kind`: the row of `wear_table` (16 dwords a row).
    pub tyre_kind: PlainDword,
    /// `dword_75B3EC`: Copy of the centre point's surface kind (`wheel_surface_a[4]`): its low four bits pick the wear table's columns.
    pub surface: PlainDword,
    /// `dword_75B3F0`: Copy of the car's `surface_blend` (a float, 0 ..= 1, held in a dword cell): the weight of the second surface's column against the first when `wear_figure` is blended.
    pub surface_blend: PlainFloat,
    /// `flt_75B3F4`: 1 when at least one wheel is not off the ground, else 0.
    pub wheels_down: PlainFloat,
    /// `flt_75B3F8`: the car's ride-height figure over `fixed_point_scale`.
    pub ride_height_share: PlainFloat,
    /// `flt_75B3FC`: the car's suspension-stiffness figure over `fixed_point_scale`.
    pub stiffness_share: PlainFloat,
    /// `flt_75B400`: the car's brake-balance figure over `fixed_point_scale`.
    pub brake_balance_share: PlainFloat,
    /// `flt_75B404`: the chance the electrics fail this step (ramp of `electrics_damage`, table 11), `electrical_failures[0]` set by it (`physics::rate_damage`).
    pub electrics_fail_chance: PlainFloat,
    /// `flt_75B408`: the chance light 0 (`light_damage[0]`, table 11) goes out this step, `electrical_failures[1 + 0]` set by it.
    pub light_fail_chance_0: PlainFloat,
    /// `flt_75B40C`: the chance light 1 (`light_damage[1]`, table 11) goes out this step, `electrical_failures[1 + 1]` set by it.
    pub light_fail_chance_1: PlainFloat,
    /// `flt_75B410`: the chance light 2 (`light_damage[2]`, table 11) goes out this step, `electrical_failures[1 + 2]` set by it.
    pub light_fail_chance_2: PlainFloat,
    /// `flt_75B414`: the chance light 3 (`light_damage[3]`, table 11) goes out this step, `electrical_failures[1 + 3]` set by it.
    pub light_fail_chance_3: PlainFloat,
    /// `flt_75B418`: the chance light 4 (`light_damage[4]`, table 11) goes out this step, `electrical_failures[1 + 4]` set by it.
    pub light_fail_chance_4: PlainFloat,
    /// `flt_75B41C`: the chance light 5 (`light_damage[5]`, table 11) goes out this step, `electrical_failures[1 + 5]` set by it.
    pub light_fail_chance_5: PlainFloat,
    /// `flt_75B420`: the chance a failed electrics recovers this step (ramp of `electrics_damage`, table 11).
    pub electrics_recover_chance: PlainFloat,
    /// `flt_75B424`: the chance a failed light 0 comes back this step (ramp of `light_damage[0]`, table 12).
    pub light_recover_chance_0: PlainFloat,
    /// `flt_75B428`: the chance a failed light 1 comes back this step (ramp of `light_damage[1]`, table 12).
    pub light_recover_chance_1: PlainFloat,
    /// `flt_75B42C`: the chance a failed light 2 comes back this step (ramp of `light_damage[2]`, table 12).
    pub light_recover_chance_2: PlainFloat,
    /// `flt_75B430`: the chance a failed light 3 comes back this step (ramp of `light_damage[3]`, table 12).
    pub light_recover_chance_3: PlainFloat,
    /// `flt_75B434`: the chance a failed light 4 comes back this step (ramp of `light_damage[4]`, table 12).
    pub light_recover_chance_4: PlainFloat,
    /// `flt_75B438`: the chance a failed light 5 comes back this step (ramp of `light_damage[5]`, table 12).
    pub light_recover_chance_5: PlainFloat,
    /// `flt_75B43C`: 1 - `brake_balance_share`, the other axle's brake share: `brake_wear_b` rises by its rate times this. `physics::rate_damage` reuses the cell as the scratch of the drive-efficiency ramp.
    pub brake_balance_complement: PlainFloat,
    /// `flt_75B440`: the front tyres' wear, the mean of `tyre_wears` 0 and 1 (FL, FR; "Front tyres" on the developers' wear page): the input of the drive-efficiency ramp for drive speed A, 0 ..= 100.
    pub front_tyre_wear: PlainFloat,
    /// `flt_75B444`: the rear tyres' wear, the mean of `tyre_wears` 2 and 3 (RL, RR; "Rear tyres"): as `front_tyre_wear` for drive speed B.
    pub rear_tyre_wear: PlainFloat,
    /// `flt_75B448`: the stiffness mean through `lerp_tables_b[0]`, squared: scales the suspension's roughness wear (`suspension_roughness_wear`).
    pub stiffness_suspension_factor: PlainFloat,
    /// `flt_75B44C`: the stiffness mean through `lerp_tables_b[2]`, squared: scales the body roughness wear.
    pub stiffness_body_factor: PlainFloat,
    /// `flt_75B450`: surface figure times the speed share times `figures[32]` times `wheels_down`: the rough-ground wear before any part's factor; it wears the wheel hit gauges (times `wheel_roughness_rate`).
    pub roughness_wear_base: PlainFloat,
    /// `flt_75B454`: `roughness_wear_base` times `stiffness_body_factor` and `ride_height_body_factor`: the rate for the body, steering, engine, exhaust, gears and differential gauges.
    pub body_roughness_wear: PlainFloat,
    /// `flt_75B458`: `roughness_wear_base` times `stiffness_suspension_factor`: the rate for the four suspension gauges.
    pub suspension_roughness_wear: PlainFloat,
    /// `flt_75B45C`: the engine speed share through `lerp_tables[0]`: the rate of the engine and cooling gauges' wear from revving.
    pub engine_speed_wear_factor: PlainFloat,
    /// `flt_75B460`: the cooling damage through `lerp_tables[1]`: the engine speed share the cooling can still carry off.
    pub cooling_capacity: PlainFloat,
    /// `flt_75B464`: engine speed share minus `cooling_capacity`, held at 0 or more: the heat the cooling cannot carry; raises the engine heat and, when above 0, the engine and cooling damage.
    pub overheat_excess: PlainFloat,
    /// `flt_75B468`: the ride-height mean through `lerp_tables_b[1]`, squared: scales the body roughness wear.
    pub ride_height_body_factor: PlainFloat,
    /// `flt_75B46C`: the drive-A slip share through `lerp_tables[3]`: how much slip adds to the front tyres' wear.
    pub front_slip_wear_factor: PlainFloat,
    /// `flt_75B470`: the drive-B slip share through `lerp_tables[3]`: the same for tyre gauges 2 and 3.
    pub rear_slip_wear_factor: PlainFloat,
    /// `flt_75B474` ..: the four wheels' table lerps of the suspension damage
    /// (`physics::damage_factors`).
    pub wheel_stiffness_lerp: [PlainFloat; 4],
    /// `flt_75B484`: The tyre durability on this ground: the two columns of `wear_table` (whole numbers, e.g. 100, 20, 60, 5) blended by `surface_blend`, times `wear_figure_scale`; the tyre wear rates divide by it, so a bigger figure wears slower.
    pub wear_figure: PlainFloat,
    /// `flt_75B488`: the speed share over (`wear_figure` times `metres_per_mile`) times `gauge_top` times `wheels_down`, times the kind table A entry: the speed part of the front tyres' wear rate.
    pub front_tyre_wear_rate: PlainFloat,
    /// `flt_75B48C`: as `front_tyre_wear_rate`, times the kind table B entry: the speed part of the rear tyres' wear rate.
    pub rear_tyre_wear_rate: PlainFloat,
    /// `flt_75B490`: the chance gear changes are blocked this step (ramp of `clutch_damage`, table 7; sets `gear_change_blocked`).
    pub clutch_block_chance: PlainFloat,
    /// `flt_75B494`: the chance gear changes are blocked (ramp of `gearbox_damage`, table 8; sets `gear_change_blocked`).
    pub gearbox_block_chance: PlainFloat,
    /// `flt_75B498`: the chance the gearbox is forced into neutral (ramp of `gearbox_damage`, table 9; sets `forced_neutral`).
    pub gearbox_neutral_chance: PlainFloat,
    /// `flt_75B49C`: the chance the handbrake fails for good (ramp of `brake_damage`, table 10; sets `handbrake_failed`).
    pub handbrake_fail_chance: PlainFloat,
    /// `dword_75B4A0`: the clock / random figure the failures by chance compare against.
    pub clock: PlainDword,
    /// Unreferenced bytes (IDA 0X75B4A4 .. 0X75B4A8).
    pub _gap_75b4a4: PlainBytes<4>,
    /// The tables of four (x0, y0, x1, y1): 0x75B4A8 .. 0x75B578 (`physics::damage_wear`,
    /// `rate_damage`, `damage_factors`).
    pub lerp_tables: [[PlainFloat; 4]; 13],
    /// `flt_75B578` ..: the wear rates (`flt_75B584` ..) `physics::damage_wear` raises and lowers the gauges by.
    pub wear_rates: [PlainFloat; 13],
    /// `flt_75B5B0` at index 1 (0x75B5AC .. 0x75B5D0): the gear factors by gear + 1
    /// (reverse, neutral, 1 .. 7), `physics::damage_gauges`; `flt_75B5B0` itself is the neutral entry (index 1), read when the gear is 0 like the others.
    pub gear_factors: [PlainFloat; 9],
    /// `flt_75B5D0` ..: the gauges' rates (`physics::damage_gauges`), times the step's shares.
    pub gauge_rates: [PlainFloat; 24],
    /// `flt_75B630`: the first kind table (by `DamageModel.drive_kind`'s bits).
    pub kind_table_a: [PlainFloat; 2],
    /// `flt_75B638`: the second kind table.
    pub kind_table_b: [PlainFloat; 2],
    /// Three more tables of four: 0x75B640, 0x75B650, 0x75B660.
    pub lerp_tables_b: [[PlainFloat; 4]; 3],
    /// `flt_75B670`: 0.15: the rate of the wheel hit gauges' wear by rough ground (times `roughness_wear_base`).
    pub wheel_roughness_rate: PlainFloat,
    /// Unreferenced bytes (IDA 0X75B674 .. 0X75B678).
    pub _gap_75b674: PlainBytes<4>,
    /// `flt_75B678` ..: more hit figures and thresholds (`physics::damage_gauges`, `rate_damage`).
    pub figures: [PlainFloat; 33],
    /// Unreferenced bytes (IDA 0X75B6FC .. 0X75B73C).
    pub _gap_75b6fc: PlainBytes<64>,
    /// `dword_75B73C`: the wear table (rows by `DamageModel.tyre_kind`, two columns, as the original
    /// indexes them), 0x180 bytes.
    pub wear_table: [PlainFloat; 96],
    /// `flt_75B8BC`: 2.5: `wear_figure` is the blended table figure times it.
    pub wear_figure_scale: PlainFloat,
    /// `dword_75B8C0`: the wear table's columns by surface kind, a pair of dwords a surface
    /// (first column, second column; the rest of the pairs sit in `_gap_75b8c8`): here the
    /// first of surface 0's pair; `physics::damage_wear` reads past it.
    pub wear_first_columns: PlainDword,
    /// `dword_75B8C4`: the second of surface 0's column pair (the original read the second
    /// column at the first column's pair, not the surface's: fixed in `physics::damage_wear`).
    pub wear_second_columns: PlainDword,
    /// Unreferenced bytes (IDA 0X75B8C8 .. 0X75B940).
    pub _gap_75b8c8: PlainBytes<120>,
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
            _gap_75b358: PlainBytes::new(sparse::<4>(&[(0x2, &[0x20, 0x41])])),
            fixed_point_scale: PlainFloat::new(256.0),
            half: PlainFloat::new(0.5),
            quarter: PlainFloat::new(0.25),
            tenth: PlainFloat::new(0.1),
            scratch: PlainFloat::new(0.0),
            hundredth: PlainFloat::new(0.01),
            _gap_75b374: PlainBytes::new(sparse::<4>(&[(0x2, &[0x7a, 0x44])])),
            speed_range: PlainFloat::new(70.0),
            engine_limit_factor: PlainFloat::new(0.5),
            metres_per_mile: PlainFloat::new(1609.3457),
            _gap_75b384: PlainBytes::new(sparse::<4>(&[(0x0, &[0x6f, 0x12, 0x83, 0x3a])])),
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
            gear_changed: PlainDword::new(0),
            gear: PlainDword::new(0),
            engine_over_limit: PlainFloat::new(0.0),
            hit_side: PlainDword::new(0),
            hit_size: PlainFloat::new(0.0),
            wheel_loads: [const { PlainFloat::new(0.0) }; 4],
            wheel_load_mean: PlainFloat::new(0.0),
            point_surface_copy: PlainFloat::new(0.0),
            drive_kind: PlainFloat::new(0.0),
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
            _gap_75b4a4: PlainBytes::new(sparse::<4>(&[(0x0, &[0xcd, 0xcc, 0x4c, 0x3f])])),
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
            _gap_75b674: PlainBytes::new(sparse::<4>(&[(0x0, &[0xcd, 0xcc, 0xcc, 0x3d])])),
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
            _gap_75b6fc: PlainBytes::new(sparse::<64>(&[(
                0x0,
                &[
                    0x64, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x14,
                    0x00, 0x00, 0x00, 0x23, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x1e, 0x00,
                    0x00, 0x00, 0x3c, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00,
                    0x00, 0x64, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00,
                    0x64, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x64,
                ],
            )])),
            wear_table: [
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x14),
                PlainFloat::from_bits(0x28),
                PlainFloat::from_bits(0xa),
                PlainFloat::from_bits(0xf),
                PlainFloat::from_bits(0xf),
                PlainFloat::from_bits(0x1e),
                PlainFloat::from_bits(0x3c),
                PlainFloat::from_bits(5),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x14),
                PlainFloat::from_bits(0x23),
                PlainFloat::from_bits(8),
                PlainFloat::from_bits(0xd),
                PlainFloat::from_bits(0xc),
                PlainFloat::from_bits(0x19),
                PlainFloat::from_bits(0x32),
                PlainFloat::from_bits(4),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x12),
                PlainFloat::from_bits(0x28),
                PlainFloat::from_bits(6),
                PlainFloat::from_bits(0xa),
                PlainFloat::from_bits(0xa),
                PlainFloat::from_bits(0x14),
                PlainFloat::from_bits(0x28),
                PlainFloat::from_bits(3),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0xf),
                PlainFloat::from_bits(0x23),
                PlainFloat::from_bits(0x1c),
                PlainFloat::from_bits(0x23),
                PlainFloat::from_bits(0xf),
                PlainFloat::from_bits(0x1e),
                PlainFloat::from_bits(0x3c),
                PlainFloat::from_bits(8),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0xc),
                PlainFloat::from_bits(0x1e),
                PlainFloat::from_bits(0x16),
                PlainFloat::from_bits(0x1c),
                PlainFloat::from_bits(0xf),
                PlainFloat::from_bits(0x19),
                PlainFloat::from_bits(0x32),
                PlainFloat::from_bits(7),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0xc),
                PlainFloat::from_bits(0x12),
                PlainFloat::from_bits(0xa),
                PlainFloat::from_bits(0xc),
                PlainFloat::from_bits(0xa),
                PlainFloat::from_bits(0xf),
                PlainFloat::from_bits(0x28),
                PlainFloat::from_bits(3),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
                PlainFloat::from_bits(0x64),
            ],
            wear_figure_scale: PlainFloat::new(2.5),
            wear_first_columns: PlainDword::new(0),
            wear_second_columns: PlainDword::new(0),
            _gap_75b8c8: PlainBytes::new(sparse::<120>(&[(
                0x0,
                &[
                    0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04,
                    0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x07, 0x00,
                    0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00,
                    0x00, 0x05, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
                    0x06, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x07,
                    0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00,
                    0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00,
                    0x00, 0x07,
                ],
            )])),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: usize = 0x75B340;
    use crate::race::owner::race;
    use core::mem::offset_of;

    #[test]
    fn damage_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            race().physics.damage(),
        );
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(DamageModel, lerp_tables), 0x75B4A8);
        at(offset_of!(DamageModel, gear_factors), 0x75B5AC);
        at(offset_of!(DamageModel, wear_table), 0x75B73C);
    }
}
