//! The damage model's figures, out of RAL.EXE's image (IDA 0x75B340 .. 0x75B940; phase 7,
//! typed): shares, thresholds, the work floats, the tables of four (x0, y0, x1, y1)
//! `physics::flow::table_lerp` reads, and the rates. `#[repr(C)]` struct of atomic cells
//! with the image's layout, one static; `moved::MOVED` points the range at it. What each
//! figure is, is a reading from how `physics::damage_factors`, `damage_wear`, `damage_gauges`
//! and `rate_damage` use it (`physics/damage_model.rs`); the doc of each says what.

use crate::session::{Bytes, Dword, Float};

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
    pub chance_scale: Float,
    /// `flt_75B344`: 6.0, the number of lights: the six light gauges' sum is divided by it for their mean (`lights_damage`).
    pub light_count_divisor: Float,
    /// `flt_75B348`: the working grades are held up to it.
    pub grade_high: Float,
    /// `flt_75B34C`: the gauges and gears are held 0 ..= it; a gear equal to it is worn out.
    pub gauge_top: Float,
    /// `flt_75B350`: the gauges and grades are held down to it.
    pub gauge_low: Float,
    /// `flt_75B354`: 8.0, the highest landing dive figure counted (`physics::damage_factors` holds the dive 0 ..= it before squaring).
    pub dive_top: Float,
    /// Unreferenced bytes (IDA 0X75B358 .. 0X75B35C).
    pub _gap_75b358: Bytes<4>,
    /// `flt_75B35C`: the divisor (256) that turns the car block's whole-number figures into shares.
    pub fixed_point_scale: Float,
    /// `flt_75B360`: 0.5: the average of two (brake wear, tyre wear, gearbox, the wheel loads' blend).
    pub half: Float,
    /// `flt_75B364`: 0.25: the average of four (the wheel stiffness / ride-height means, the wheel loads).
    pub quarter: Float,
    /// `flt_75B368`: 0.1: the body damage gauge is the sum of its parts times it.
    pub tenth: Float,
    /// `flt_75B36C`: the work cell a `fistp` rounds into.
    pub scratch: Float,
    /// `flt_75B370`: 0.01: the percent factor the wear and hit terms are multiplied by.
    pub hundredth: Float,
    /// Unreferenced bytes (IDA 0X75B374 .. 0X75B378).
    pub _gap_75b374: Bytes<4>,
    /// `flt_75B378`: 70.0: the forward speed (and speed gaps) are held to it and divided by it, then scaled to `gauge_top`, so a speed share runs 0 ..= 100.
    pub speed_range: Float,
    /// `flt_75B37C`: 0.5: the engine counts as over its limit when its speed is above `engine_speed_limit` times it.
    pub engine_limit_factor: Float,
    /// `flt_75B380`: 1609.34, the metres in a mile: the tyre durability (`wear_figure`) is turned to this unit before the speed share is divided by it (probably, the durability figures being miles).
    pub metres_per_mile: Float,
    /// Unreferenced bytes (IDA 0X75B384 .. 0X75B388).
    pub _gap_75b384: Bytes<4>,
    /// `flt_75B388`: the factor `physics::rate_damage` multiplies by (80 bits).
    pub grade_k: Float,
    /// `flt_75B38C`: About 1/6000 (0.000167): the gauge rates are multiplied by it, so a gauge rises by its rate times this per step.
    pub gauge_step: Float,
    /// `flt_75B390`: the step the wear rates are multiplied by (`physics::damage_wear`).
    pub wear_step: Float,
    /// `flt_75B394`: 45.0: the hit size is the contact depth squared over it and over `speed_range`.
    pub hit_divisor: Float,
    /// `flt_75B398`: the engine speed over its limit, scaled to `gauge_top`.
    pub engine_speed_share: Float,
    /// `flt_75B39C`: the forward speed over `speed_range`, scaled to `gauge_top`.
    pub speed_share: Float,
    /// `flt_75B3A0`: the gap between the forward speed and drive speed A over `speed_range`.
    pub drive_a_slip_share: Float,
    /// `flt_75B3A4`: the gap between the forward speed and drive speed B over `speed_range`.
    pub drive_b_slip_share: Float,
    /// `flt_75B3A8`: drive speed A pushed past the forward speed, over `speed_range`.
    pub drive_a_extrapolated_share: Float,
    /// `flt_75B3AC`: drive speed B pushed past the forward speed, over `speed_range`.
    pub drive_b_extrapolated_share: Float,
    /// `flt_75B3B0`: the brake figure over `fixed_point_scale` while the car moves and the brake is 0x10 or more.
    pub brake_share: Float,
    /// `flt_75B3B4`: 1 while the car's centre point is on surface kind 5 (wet, as `wheel_surface_a` says), else 0: it wets the brakes (fade) and damages the electrics and the lights.
    pub on_surface_5: Float,
    /// `dword_75B3B8`: Copy of the car's `gear_changed`: 1 after a shift up, -1 after a shift down, else 0 (the gauges add the clutch and gear wear of a shift).
    pub gear_changed: Dword,
    /// `dword_75B3BC`: Copy of the car's `gear`: -1 (reverse), 0 (neutral), 1 ..= 7; picks the gear's wear gauge and factor.
    pub gear: Dword,
    /// `flt_75B3C0`: 1.0 when the engine speed is above `engine_speed_limit` times `engine_limit_factor`, else 0: the turbo wear while over the limit.
    pub engine_over_limit: Float,
    /// `dword_75B3C4`: Where the car was hit: 0 none, else 1 ..= 4 by the side of the contact point (1 + 1 if the point's x is above 0 + 2 if its z is 0 or below).
    pub hit_side: Dword,
    /// `flt_75B3C8`: The size of this step's hit: the contact depth squared over `hit_divisor` over `speed_range`; scaled by the damage setting (`patch_factors[1]`) in the gauges, zeroed while the hit cooldown runs.
    pub hit_size: Float,
    /// `dword_75B3CC` ..: the four wheels' loads, blended by `flt_75B360` (`physics::damage_factors`).
    pub wheel_loads: [Float; 4],
    /// `flt_75B3DC`: The mean of the four wheel loads (their sum times `quarter`).
    pub wheel_load_mean: Float,
    /// `flt_75B3E0`: Copy of the centre point's surface figure (`point_surface_2c[4]`, the surface's +0x2c value blended between its two cells): how hard the ground is on the car; it scales the roughness wear. Probably a roughness.
    pub point_surface_copy: Float,
    /// `flt_75B3E4`: Copy of the car's `drive_kind` (0 or 1, as a whole number in a float cell): picks the entry of the two kind tables, and a car with 0 has no turbo damage.
    pub drive_kind: Float,
    /// `dword_75B3E8`: Copy of the car's `tyre_kind`: the row of `wear_table` (16 dwords a row).
    pub tyre_kind: Dword,
    /// `dword_75B3EC`: Copy of the centre point's surface kind (`wheel_surface_a[4]`): its low four bits pick the wear table's columns.
    pub surface: Dword,
    /// `dword_75B3F0`: Copy of the car's `surface_blend` (a float, 0 ..= 1, held in a dword cell): the weight of the second surface's column against the first when `wear_figure` is blended.
    pub surface_blend: Dword,
    /// `flt_75B3F4`: 1 when at least one wheel is not off the ground, else 0.
    pub wheels_down: Float,
    /// `flt_75B3F8`: the car's ride-height figure over `fixed_point_scale`.
    pub ride_height_share: Float,
    /// `flt_75B3FC`: the car's suspension-stiffness figure over `fixed_point_scale`.
    pub stiffness_share: Float,
    /// `flt_75B400`: the car's brake-balance figure over `fixed_point_scale`.
    pub brake_balance_share: Float,
    /// `flt_75B404`: the chance the electrics fail this step (ramp of `electrics_damage`, table 11), `electrical_failures[0]` set by it (`physics::rate_damage`).
    pub electrics_fail_chance: Float,
    /// `flt_75B408`: the chance light 0 (`light_damage[0]`, table 11) goes out this step, `electrical_failures[1 + 0]` set by it.
    pub light_fail_chance_0: Float,
    /// `flt_75B40C`: the chance light 1 (`light_damage[1]`, table 11) goes out this step, `electrical_failures[1 + 1]` set by it.
    pub light_fail_chance_1: Float,
    /// `flt_75B410`: the chance light 2 (`light_damage[2]`, table 11) goes out this step, `electrical_failures[1 + 2]` set by it.
    pub light_fail_chance_2: Float,
    /// `flt_75B414`: the chance light 3 (`light_damage[3]`, table 11) goes out this step, `electrical_failures[1 + 3]` set by it.
    pub light_fail_chance_3: Float,
    /// `flt_75B418`: the chance light 4 (`light_damage[4]`, table 11) goes out this step, `electrical_failures[1 + 4]` set by it.
    pub light_fail_chance_4: Float,
    /// `flt_75B41C`: the chance light 5 (`light_damage[5]`, table 11) goes out this step, `electrical_failures[1 + 5]` set by it.
    pub light_fail_chance_5: Float,
    /// `flt_75B420`: the chance a failed electrics recovers this step (ramp of `electrics_damage`, table 11).
    pub electrics_recover_chance: Float,
    /// `flt_75B424`: the chance a failed light 0 comes back this step (ramp of `light_damage[0]`, table 12).
    pub light_recover_chance_0: Float,
    /// `flt_75B428`: the chance a failed light 1 comes back this step (ramp of `light_damage[1]`, table 12).
    pub light_recover_chance_1: Float,
    /// `flt_75B42C`: the chance a failed light 2 comes back this step (ramp of `light_damage[2]`, table 12).
    pub light_recover_chance_2: Float,
    /// `flt_75B430`: the chance a failed light 3 comes back this step (ramp of `light_damage[3]`, table 12).
    pub light_recover_chance_3: Float,
    /// `flt_75B434`: the chance a failed light 4 comes back this step (ramp of `light_damage[4]`, table 12).
    pub light_recover_chance_4: Float,
    /// `flt_75B438`: the chance a failed light 5 comes back this step (ramp of `light_damage[5]`, table 12).
    pub light_recover_chance_5: Float,
    /// `flt_75B43C`: 1 - `brake_balance_share`, the other axle's brake share: `brake_wear_b` rises by its rate times this. `physics::rate_damage` reuses the cell as the scratch of the drive-efficiency ramp.
    pub brake_balance_complement: Float,
    /// `flt_75B440`: the front tyres' wear, the mean of `tyre_wears` 0 and 1 (FL, FR; "Front tyres" on the developers' wear page): the input of the drive-efficiency ramp for drive speed A, 0 ..= 100.
    pub front_tyre_wear: Float,
    /// `flt_75B444`: the rear tyres' wear, the mean of `tyre_wears` 2 and 3 (RL, RR; "Rear tyres"): as `front_tyre_wear` for drive speed B.
    pub rear_tyre_wear: Float,
    /// `flt_75B448`: the stiffness mean through `lerp_tables_b[0]`, squared: scales the suspension's roughness wear (`suspension_roughness_wear`).
    pub stiffness_suspension_factor: Float,
    /// `flt_75B44C`: the stiffness mean through `lerp_tables_b[2]`, squared: scales the body roughness wear.
    pub stiffness_body_factor: Float,
    /// `flt_75B450`: surface figure times the speed share times `figures[32]` times `wheels_down`: the rough-ground wear before any part's factor; it wears the wheel hit gauges (times `wheel_roughness_rate`).
    pub roughness_wear_base: Float,
    /// `flt_75B454`: `roughness_wear_base` times `stiffness_body_factor` and `ride_height_body_factor`: the rate for the body, steering, engine, exhaust, gears and differential gauges.
    pub body_roughness_wear: Float,
    /// `flt_75B458`: `roughness_wear_base` times `stiffness_suspension_factor`: the rate for the four suspension gauges.
    pub suspension_roughness_wear: Float,
    /// `flt_75B45C`: the engine speed share through `lerp_tables[0]`: the rate of the engine and cooling gauges' wear from revving.
    pub engine_speed_wear_factor: Float,
    /// `flt_75B460`: the cooling damage through `lerp_tables[1]`: the engine speed share the cooling can still carry off.
    pub cooling_capacity: Float,
    /// `flt_75B464`: engine speed share minus `cooling_capacity`, held at 0 or more: the heat the cooling cannot carry; raises the engine heat and, when above 0, the engine and cooling damage.
    pub overheat_excess: Float,
    /// `flt_75B468`: the ride-height mean through `lerp_tables_b[1]`, squared: scales the body roughness wear.
    pub ride_height_body_factor: Float,
    /// `flt_75B46C`: the drive-A slip share through `lerp_tables[3]`: how much slip adds to the front tyres' wear.
    pub front_slip_wear_factor: Float,
    /// `flt_75B470`: the drive-B slip share through `lerp_tables[3]`: the same for tyre gauges 2 and 3.
    pub rear_slip_wear_factor: Float,
    /// `flt_75B474` ..: the four wheels' table lerps of the suspension damage
    /// (`physics::damage_factors`).
    pub wheel_stiffness_lerp: [Float; 4],
    /// `flt_75B484`: The tyre durability on this ground: the two columns of `wear_table` (whole numbers, e.g. 100, 20, 60, 5) blended by `surface_blend`, times `wear_figure_scale`; the tyre wear rates divide by it, so a bigger figure wears slower.
    pub wear_figure: Float,
    /// `flt_75B488`: the speed share over (`wear_figure` times `metres_per_mile`) times `gauge_top` times `wheels_down`, times the kind table A entry: the speed part of the front tyres' wear rate.
    pub front_tyre_wear_rate: Float,
    /// `flt_75B48C`: as `front_tyre_wear_rate`, times the kind table B entry: the speed part of the rear tyres' wear rate.
    pub rear_tyre_wear_rate: Float,
    /// `flt_75B490`: the chance gear changes are blocked this step (ramp of `clutch_damage`, table 7; sets `gear_change_blocked`).
    pub clutch_block_chance: Float,
    /// `flt_75B494`: the chance gear changes are blocked (ramp of `gearbox_damage`, table 8; sets `gear_change_blocked`).
    pub gearbox_block_chance: Float,
    /// `flt_75B498`: the chance the gearbox is forced into neutral (ramp of `gearbox_damage`, table 9; sets `forced_neutral`).
    pub gearbox_neutral_chance: Float,
    /// `flt_75B49C`: the chance the handbrake fails for good (ramp of `brake_damage`, table 10; sets `handbrake_failed`).
    pub handbrake_fail_chance: Float,
    /// `dword_75B4A0`: the clock / random figure the failures by chance compare against.
    pub clock: Dword,
    /// Unreferenced bytes (IDA 0X75B4A4 .. 0X75B4A8).
    pub _gap_75b4a4: Bytes<4>,
    /// The tables of four (x0, y0, x1, y1): 0x75B4A8 .. 0x75B578 (`physics::damage_wear`,
    /// `rate_damage`, `damage_factors`).
    pub lerp_tables: [[Float; 4]; 13],
    /// `flt_75B578` ..: the wear rates (`flt_75B584` ..) `physics::damage_wear` raises and lowers the gauges by.
    pub wear_rates: [Float; 13],
    /// `flt_75B5B0` at index 1 (0x75B5AC .. 0x75B5D0): the gear factors by gear + 1
    /// (reverse, neutral, 1 .. 7), `physics::damage_gauges`; `flt_75B5B0` itself is the neutral entry (index 1), read when the gear is 0 like the others.
    pub gear_factors: [Float; 9],
    /// `flt_75B5D0` ..: the gauges' rates (`physics::damage_gauges`), times the step's shares.
    pub gauge_rates: [Float; 24],
    /// `flt_75B630`: the first kind table (by `flt_75B3E4`'s bits).
    pub kind_table_a: [Float; 2],
    /// `flt_75B638`: the second kind table.
    pub kind_table_b: [Float; 2],
    /// Three more tables of four: 0x75B640, 0x75B650, 0x75B660.
    pub lerp_tables_b: [[Float; 4]; 3],
    /// `flt_75B670`: 0.15: the rate of the wheel hit gauges' wear by rough ground (times `roughness_wear_base`).
    pub wheel_roughness_rate: Float,
    /// Unreferenced bytes (IDA 0X75B674 .. 0X75B678).
    pub _gap_75b674: Bytes<4>,
    /// `flt_75B678` ..: more hit figures and thresholds (`physics::damage_gauges`, `rate_damage`).
    pub figures: [Float; 33],
    /// Unreferenced bytes (IDA 0X75B6FC .. 0X75B73C).
    pub _gap_75b6fc: Bytes<64>,
    /// `dword_75B73C`: the wear table (rows by `dword_75B3E8`, two columns, as the original
    /// indexes them), 0x180 bytes.
    pub wear_table: [Float; 96],
    /// `flt_75B8BC`: 2.5: `wear_figure` is the blended table figure times it.
    pub wear_figure_scale: Float,
    /// `dword_75B8C0`: the wear table's columns by surface kind, a pair of dwords a surface
    /// (first column, second column; the rest of the pairs sit in `_gap_75b8c8`): here the
    /// first of surface 0's pair; `physics::damage_wear` reads past it.
    pub wear_first_columns: Dword,
    /// `dword_75B8C4`: the second of surface 0's column pair (the original read the second
    /// column at the first column's pair, not the surface's: fixed in `physics::damage_wear`).
    pub wear_second_columns: Dword,
    /// Unreferenced bytes (IDA 0X75B8C8 .. 0X75B940).
    pub _gap_75b8c8: Bytes<120>,
}

/// First values the image's.
impl DamageModel {
    pub const fn new() -> Self {
        DamageModel {
            chance_scale: Float::from_bits(0x47800000),
            light_count_divisor: Float::from_bits(0x40c00000),
            grade_high: Float::from_bits(0x3f800000),
            gauge_top: Float::from_bits(0x42c80000),
            gauge_low: Float::from_bits(0),
            dive_top: Float::from_bits(0x41000000),
            _gap_75b358: Bytes::new(sparse::<4>(&[(0x2, &[0x20, 0x41])])),
            fixed_point_scale: Float::from_bits(0x43800000),
            half: Float::from_bits(0x3f000000),
            quarter: Float::from_bits(0x3e800000),
            tenth: Float::from_bits(0x3dcccccd),
            scratch: Float::from_bits(0),
            hundredth: Float::from_bits(0x3c23d70a),
            _gap_75b374: Bytes::new(sparse::<4>(&[(0x2, &[0x7a, 0x44])])),
            speed_range: Float::from_bits(0x428c0000),
            engine_limit_factor: Float::from_bits(0x3f000000),
            metres_per_mile: Float::from_bits(0x44c92b10),
            _gap_75b384: Bytes::new(sparse::<4>(&[(0x0, &[0x6f, 0x12, 0x83, 0x3a])])),
            grade_k: Float::from_bits(0x3c23d70a),
            gauge_step: Float::from_bits(0x392f1cb9),
            wear_step: Float::from_bits(0x3c23d70a),
            hit_divisor: Float::from_bits(0x42340000),
            engine_speed_share: Float::from_bits(0),
            speed_share: Float::from_bits(0),
            drive_a_slip_share: Float::from_bits(0),
            drive_b_slip_share: Float::from_bits(0),
            drive_a_extrapolated_share: Float::from_bits(0),
            drive_b_extrapolated_share: Float::from_bits(0),
            brake_share: Float::from_bits(0),
            on_surface_5: Float::from_bits(0),
            gear_changed: Dword::new(0),
            gear: Dword::new(0),
            engine_over_limit: Float::from_bits(0),
            hit_side: Dword::new(0),
            hit_size: Float::from_bits(0),
            wheel_loads: [const { Float::from_bits(0) }; 4],
            wheel_load_mean: Float::from_bits(0),
            point_surface_copy: Float::from_bits(0),
            drive_kind: Float::from_bits(0),
            tyre_kind: Dword::new(0),
            surface: Dword::new(0),
            surface_blend: Dword::new(0),
            wheels_down: Float::from_bits(0),
            ride_height_share: Float::from_bits(0x42480000),
            stiffness_share: Float::from_bits(0x42480000),
            brake_balance_share: Float::from_bits(0x42480000),
            electrics_fail_chance: Float::from_bits(0),
            light_fail_chance_0: Float::from_bits(0),
            light_fail_chance_1: Float::from_bits(0),
            light_fail_chance_2: Float::from_bits(0),
            light_fail_chance_3: Float::from_bits(0),
            light_fail_chance_4: Float::from_bits(0),
            light_fail_chance_5: Float::from_bits(0),
            electrics_recover_chance: Float::from_bits(0),
            light_recover_chance_0: Float::from_bits(0),
            light_recover_chance_1: Float::from_bits(0),
            light_recover_chance_2: Float::from_bits(0),
            light_recover_chance_3: Float::from_bits(0),
            light_recover_chance_4: Float::from_bits(0),
            light_recover_chance_5: Float::from_bits(0),
            brake_balance_complement: Float::from_bits(0),
            front_tyre_wear: Float::from_bits(0),
            rear_tyre_wear: Float::from_bits(0),
            stiffness_suspension_factor: Float::from_bits(0),
            stiffness_body_factor: Float::from_bits(0),
            roughness_wear_base: Float::from_bits(0),
            body_roughness_wear: Float::from_bits(0),
            suspension_roughness_wear: Float::from_bits(0),
            engine_speed_wear_factor: Float::from_bits(0),
            cooling_capacity: Float::from_bits(0),
            overheat_excess: Float::from_bits(0),
            ride_height_body_factor: Float::from_bits(0),
            front_slip_wear_factor: Float::from_bits(0),
            rear_slip_wear_factor: Float::from_bits(0),
            wheel_stiffness_lerp: [const { Float::from_bits(0) }; 4],
            wear_figure: Float::from_bits(0),
            front_tyre_wear_rate: Float::from_bits(0),
            rear_tyre_wear_rate: Float::from_bits(0),
            clutch_block_chance: Float::from_bits(0),
            gearbox_block_chance: Float::from_bits(0),
            gearbox_neutral_chance: Float::from_bits(0),
            handbrake_fail_chance: Float::from_bits(0),
            clock: Dword::new(0),
            _gap_75b4a4: Bytes::new(sparse::<4>(&[(0x0, &[0xcd, 0xcc, 0x4c, 0x3f])])),
            lerp_tables: [
                [
                    Float::from_bits(0x42a00000),
                    Float::from_bits(0),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x42c80000),
                ],
                [
                    Float::from_bits(0x42480000),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x42480000),
                ],
                [
                    Float::from_bits(0x42480000),
                    Float::from_bits(0),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x42c80000),
                ],
                [
                    Float::from_bits(0),
                    Float::from_bits(0x3f800000),
                    Float::from_bits(0x3e4ccccd),
                    Float::from_bits(0x40400000),
                ],
                [
                    Float::from_bits(0),
                    Float::from_bits(0x42be0000),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x42c80000),
                ],
                [
                    Float::from_bits(0),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x42a00000),
                ],
                [
                    Float::from_bits(0x42480000),
                    Float::from_bits(0),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x3e99999a),
                ],
                [
                    Float::from_bits(0x42340000),
                    Float::from_bits(0),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x3c23d70a),
                ],
                [
                    Float::from_bits(0x428c0000),
                    Float::from_bits(0),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x3ba3d70a),
                ],
                [
                    Float::from_bits(0x42480000),
                    Float::from_bits(0),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x3b449ba6),
                ],
                [
                    Float::from_bits(0x42aa0000),
                    Float::from_bits(0),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x3ba3d70a),
                ],
                [
                    Float::from_bits(0x420c0000),
                    Float::from_bits(0),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x3d4ccccd),
                ],
                [
                    Float::from_bits(0x420c0000),
                    Float::from_bits(0x3d4ccccd),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0),
                ],
            ],
            wear_rates: [
                Float::from_bits(0x3e4ccccd),
                Float::from_bits(0x3f800000),
                Float::from_bits(0x3ca3d70a),
                Float::from_bits(0x40000000),
                Float::from_bits(0x3f4ccccd),
                Float::from_bits(0x40a00000),
                Float::from_bits(0x3c23d70a),
                Float::from_bits(0x42c80000),
                Float::from_bits(0x40400000),
                Float::from_bits(0x40e00000),
                Float::from_bits(0x3f19999a),
                Float::from_bits(0x3c1374bc),
                Float::from_bits(0x3a83126f),
            ],
            gear_factors: [
                Float::from_bits(0x3f800000),
                Float::from_bits(0),
                Float::from_bits(0x3fe66666),
                Float::from_bits(0x3f99999a),
                Float::from_bits(0x3f19999a),
                Float::from_bits(0x3e99999a),
                Float::from_bits(0x3e19999a),
                Float::from_bits(0x3da3d70a),
                Float::from_bits(0x3d75c28f),
            ],
            gauge_rates: [
                Float::from_bits(0x3d99999a),
                Float::from_bits(0x3e19999a),
                Float::from_bits(0x3d99999a),
                Float::from_bits(0x3d99999a),
                Float::from_bits(0x3e000000),
                Float::from_bits(0x3f000000),
                Float::from_bits(0x3e800000),
                Float::from_bits(0x3e800000),
                Float::from_bits(0x3dcccccd),
                Float::from_bits(0x420c0000),
                Float::from_bits(0x42480000),
                Float::from_bits(0x41c80000),
                Float::from_bits(0x399d4952),
                Float::from_bits(0x3951b717),
                Float::from_bits(0x3a03126f),
                Float::from_bits(0x3ba3d70a),
                Float::from_bits(0x3ca3d70a),
                Float::from_bits(0x3e4ccccd),
                Float::from_bits(0x3c03126f),
                Float::from_bits(0x3e19999a),
                Float::from_bits(0x40800000),
                Float::from_bits(0x3d4ccccd),
                Float::from_bits(0x3ca3d70a),
                Float::from_bits(0x3ba3d70a),
            ],
            kind_table_a: [Float::from_bits(0x3f99999a), Float::from_bits(0x3f800000)],
            kind_table_b: [Float::from_bits(0x3f800000), Float::from_bits(0x3f800000)],
            lerp_tables_b: [
                [
                    Float::from_bits(0),
                    Float::from_bits(0x3f000000),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x3f800000),
                ],
                [
                    Float::from_bits(0),
                    Float::from_bits(0x3f800000),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x3f000000),
                ],
                [
                    Float::from_bits(0),
                    Float::from_bits(0x3f800000),
                    Float::from_bits(0x42c80000),
                    Float::from_bits(0x3f000000),
                ],
            ],
            wheel_roughness_rate: Float::from_bits(0x3e19999a),
            _gap_75b674: Bytes::new(sparse::<4>(&[(0x0, &[0xcd, 0xcc, 0xcc, 0x3d])])),
            figures: [
                Float::from_bits(0x3e000000),
                Float::from_bits(0x3d4ccccd),
                Float::from_bits(0x3da3d70a),
                Float::from_bits(0x3d75c28f),
                Float::from_bits(0x3e99999a),
                Float::from_bits(0x3da3d70a),
                Float::from_bits(0x3ea3d70a),
                Float::from_bits(0x3f19999a),
                Float::from_bits(0x3e4ccccd),
                Float::from_bits(0x3e99999a),
                Float::from_bits(0x3f800000),
                Float::from_bits(0x3ecccccd),
                Float::from_bits(0x3e4ccccd),
                Float::from_bits(0x41000000),
                Float::from_bits(0x40c00000),
                Float::from_bits(0x40e00000),
                Float::from_bits(0x40400000),
                Float::from_bits(0x41400000),
                Float::from_bits(0x41700000),
                Float::from_bits(0x41200000),
                Float::from_bits(0x41200000),
                Float::from_bits(0x41c80000),
                Float::from_bits(0x41700000),
                Float::from_bits(0x41800000),
                Float::from_bits(0x42a00000),
                Float::from_bits(0x42200000),
                Float::from_bits(0x41f00000),
                Float::from_bits(0x42480000),
                Float::from_bits(0x42200000),
                Float::from_bits(0x41a00000),
                Float::from_bits(0x41200000),
                Float::from_bits(0x40a00000),
                Float::from_bits(0x3c75c28f),
            ],
            _gap_75b6fc: Bytes::new(sparse::<64>(&[(
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
                Float::from_bits(0x64),
                Float::from_bits(0x14),
                Float::from_bits(0x28),
                Float::from_bits(0xa),
                Float::from_bits(0xf),
                Float::from_bits(0xf),
                Float::from_bits(0x1e),
                Float::from_bits(0x3c),
                Float::from_bits(5),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x14),
                Float::from_bits(0x23),
                Float::from_bits(8),
                Float::from_bits(0xd),
                Float::from_bits(0xc),
                Float::from_bits(0x19),
                Float::from_bits(0x32),
                Float::from_bits(4),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x12),
                Float::from_bits(0x28),
                Float::from_bits(6),
                Float::from_bits(0xa),
                Float::from_bits(0xa),
                Float::from_bits(0x14),
                Float::from_bits(0x28),
                Float::from_bits(3),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0xf),
                Float::from_bits(0x23),
                Float::from_bits(0x1c),
                Float::from_bits(0x23),
                Float::from_bits(0xf),
                Float::from_bits(0x1e),
                Float::from_bits(0x3c),
                Float::from_bits(8),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0xc),
                Float::from_bits(0x1e),
                Float::from_bits(0x16),
                Float::from_bits(0x1c),
                Float::from_bits(0xf),
                Float::from_bits(0x19),
                Float::from_bits(0x32),
                Float::from_bits(7),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0xc),
                Float::from_bits(0x12),
                Float::from_bits(0xa),
                Float::from_bits(0xc),
                Float::from_bits(0xa),
                Float::from_bits(0xf),
                Float::from_bits(0x28),
                Float::from_bits(3),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
                Float::from_bits(0x64),
            ],
            wear_figure_scale: Float::from_bits(0x40200000),
            wear_first_columns: Dword::new(0),
            wear_second_columns: Dword::new(0),
            _gap_75b8c8: Bytes::new(sparse::<120>(&[(
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

/// IDA 0X75B340, the first byte of the piece.
pub const START: usize = 0x75B340;
/// IDA 0X75B940, the end of the piece.
pub const END: usize = 0x75B940;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use crate::race::owner::race;
    use core::mem::{offset_of, size_of};

    #[test]
    fn damage_starts_as_the_games() {
        let size = size_of::<DamageModel>();
        assert!(END - START <= size && size <= (END - START).next_multiple_of(4));
        let mine = unsafe {
            std::slice::from_raw_parts((&raw const race().physics.damage).cast::<u8>(), END - START)
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(DamageModel, lerp_tables), 0x75B4A8);
        at(offset_of!(DamageModel, gear_factors), 0x75B5AC);
        at(offset_of!(DamageModel, wear_table), 0x75B73C);
    }
}
