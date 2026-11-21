//! The physics' tables and work variables: the ground kinds' records, the
//! handling constants, the car's working figures, the force and contact records, the gearbox and
//! the spec's figures (the race clock and the recorded input words are in `physics::run_store`). Most names are readings from the code's uses.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFloat, PlainInt};
use crate::physics::figures;

/// A kind of ground (0x30 bytes, `PhysicsTables.surfaces[0].table`): what `physics::wheel_surfaces`
/// blends between for each point, and `physics::apply_patches` patches (readings).
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct Surface {
    /// Where its table starts in `figures::SURFACE_TABLES`, as a dword's number (a multiple of 13).
    pub table: PlainDword,
    /// Slide grip (`patchslidegrip`).
    pub slide_grip: PlainFloat,
    /// Acceleration grip (`patchaccelgrip`).
    pub accel_grip: PlainFloat,
    /// Brake grip (`patchbrakegrip`).
    pub brake_grip: PlainFloat,
    /// How much mud the surface lets the wheels throw. Blended per point into `surface_blends`; the
    /// centre point's go to `mud_scale_a` ([0] -> throws, scaled by the drive speed; [1] ->
    /// splashes, scaled by the slip; `physics::accumulate_mud`). Probable.
    pub mud_rates: [PlainFloat; 2],
    /// No code reads or writes it (data only); kept so the record keeps its 0x30 bytes, as the
    /// recorded vectors lay it out.
    pub reserved_18: [PlainDword; 2],
    /// The drag (10.0 on two grasses, `patchstickygrass`).
    pub drag: PlainFloat,
    /// The speed over the ground above which the rolling drag applies (the centre point's blend is
    /// `wheel_drag[1]`, compared in `drive_wheels_step::speed_over_ground`).
    pub rolling_drag_min_speed: PlainFloat,
    /// The grip blended into `point_grips`.
    pub grip: PlainFloat,
    /// How much a hit on this ground wears the car: blended per point into the car's
    /// `point_wear_factor`; the centre point's multiplies the hit figures of the damage model
    /// (`damage_wear::hit_figures`). Probable.
    pub wear_factor: PlainFloat,
}

/// A force on the car (0x20 bytes): `physics::apply_force` takes it, the step clears its
/// first six dwords (`physics::clear_records`).
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ForceRecord {
    /// The force.
    pub force: [PlainFloat; 3],
    /// Where it acts.
    pub point: [PlainFloat; 3],
}

#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct PhysicsTables {
    /// 7.5 (`physics::drive_grip_forces`).
    pub grip_slide_scale: PlainFloat,
    /// 5.0 (`physics::drive_grip_forces`).
    pub grip_slip_limit: PlainFloat,
    /// 10.0, held both ways (`physics::drive_grip_forces`).
    pub grip_force_limit: PlainFloat,
    /// 0.9 (`physics::load_car_specs`).
    pub spec_scale_a4: PlainFloat,
    /// 8500.0 (`physics::load_car_specs`).
    pub rev_limit_scale: PlainFloat,
    /// 2.5 (`physics::drive_wheels_step`).
    pub drive_speed_scale: PlainFloat,
    /// 0.05 (`physics::drive_grip_forces`).
    pub side_force_scale: PlainFloat,
    /// 0.5 (`physics::drive_wheels_step`).
    pub slip_blend: PlainFloat,
    /// 0.5 (`physics::drive_wheels_step`).
    pub slip_share: PlainFloat,
    /// 1.5 (`physics::drive_wheels_step`).
    pub wheel_speed_margin: PlainFloat,
    /// 0.1 (`physics::drive_wheels_step`).
    pub wheel_speed_gap: PlainFloat,
    /// 1.0 (`physics::car_sound_state`).
    pub sound_speed_scale: PlainFloat,
    /// 0.85 (`physics::load_car_specs`).
    pub spec_scale_1b8: PlainFloat,
    /// 2.0 (`physics::drive_grip_forces`).
    pub grip_ramp: PlainFloat,
    /// 0.25 (`physics::drive_grip_forces`).
    pub grip_ramp_share: PlainFloat,
    /// 0.7 (`physics::update_camera`, the car's shadow).
    pub camera_drop: PlainFloat,
    /// 5.0 (`physics::air_drag`, the test drive).
    pub air_drag: PlainFloat,
    /// 0.75 (the test drive).
    pub rolling_drag: PlainFloat,
    /// 5.0 (`physics::free_rev`).
    pub free_rev_add: PlainFloat,
    /// 0.005 (`physics::free_rev`).
    pub free_rev_scale: PlainFloat,
    /// 1.2 (`physics::load_car_specs`).
    pub rev_top_scale: PlainFloat,
    /// 0.0025 (`physics::wheel_surfaces`).
    pub wheel_load_scale: PlainFloat,
    /// 0.05 (`physics::camera_frame`).
    pub camera_frame_scale: PlainFloat,
    /// 0.01: the physics step's length in seconds (`figures::STEP_TIME`, the same value; the
    /// contact routines still read the cell).
    pub step_time: PlainFloat,
    /// 0x1000 (`physics::crash_contact`; `figures::CRASH_DEPTH`, the same value).
    pub crash_depth: PlainDword,
    /// 10: the contact steps' limit (`figures::CONTACT_STEPS`, the same value).
    pub contact_steps: PlainDword,
    /// The ground kinds' records (`physics::wheel_surfaces`; twelve patched by
    /// `physics::apply_patches`).
    pub surfaces: [Surface; 16],
    /// 2.0 (`physics::hold_spin_on_ground`).
    pub ground_spin_limit: PlainFloat,
    /// 0x76C, 0x640, 0x384, scaled into `spec_ints` (`physics::load_car_specs`).
    pub spec_default_bc0: [PlainDword; 3],
    /// 1.0, 0.9, 0.85, 1.0 (`physics::wheel_surfaces`).
    pub surface_blend: [PlainFloat; 4],
    /// 0.85, 0.75, 2.5: the car's turning inertia by axis (`physics::apply_torque`).
    pub inertia: [PlainFloat; 3],
    /// 3000.0 then one more, by spec (`physics::load_car_specs`, by drive kind 0 .. 2: kind 2 reads
    /// the next field, `camera_drop`: a slip kept as is).
    pub rev_limits: [PlainFloat; 2],
    /// 9.8 (`physics::apply_patches`).
    pub gravity: PlainFloat,
    /// The motion kept by `physics::keep_motion`.
    pub kept_motion: [PlainFloat; 3],
    /// `physics::stall_when_still`'s.
    pub stall_figure: PlainFloat,
    /// Two shares held to 0 .. 1 and their blend (`physics::drive_wheels_step`).
    pub drive_shares: [PlainFloat; 3],
    /// The gear ratios from gear -1 (reverse) to 7 (`physics::make_gear_ratios`).
    pub gear_ratios: [PlainFloat; 9],
    /// The engine's lowest speed (`physics::load_car_specs`).
    pub idle_engine_speed: PlainFloat,
    /// The idle speed + a step, the drop to neutral below it.
    pub stall_engine_speed: PlainFloat,
    /// The motion's limit (`physics::limit_motion`).
    pub top_speed: PlainFloat,
    /// The engine's highest speed.
    pub rev_limit: PlainFloat,
    /// The face `physics::find_clear_place` logs.
    pub clear_place_face: PlainDword,
    /// The two surfaces' scales by tyre kind (`physics::wheel_surfaces`).
    pub surface_scales: [PlainFloat; 2],
    /// The car's vertical speed kept (`physics::car_physics_step`).
    pub vertical_speed: PlainFloat,
    /// `physics::pick_start_gear`'s: 100, 0, the engine speed it began with.
    pub start_gear_figures: [PlainFloat; 3],
    /// Steps since a contact (`physics::car_physics_step`, `physics::apply_contact_torques`).
    pub contact_step: PlainDword,
    /// 1 for each point on hard ground (`physics::wheel_surfaces`).
    pub hard_ground: [PlainDword; 5],
    /// `physics::drive_grip_forces`' integer.
    pub grip_drive_word: PlainInt,
    /// (`physics::cap_drive_speed`).
    pub drive_speed_figure: PlainFloat,
    /// The `noautoneutral` setting.
    pub no_auto_neutral: PlainDword,
    /// The car's weight factor (`physics::race_timers`, `physics::set_wheel_weights`).
    pub weight: PlainFloat,
    /// (`physics::set_downforce`).
    pub downforce: PlainFloat,
    /// The two axles' pushes (`physics::drive_wheels_step`, `physics::wheel_springs`).
    pub wheel_spin_push: [PlainFloat; 2],
    /// The car's place saved (`physics::restore_saved_car`).
    pub saved_place: [PlainDword; 3],
    /// The car's matrix saved.
    pub saved_matrix: [PlainFloat; 16],
    /// (`physics::drive_wheels_step`).
    pub wheel_speed_step: PlainFloat,
    /// (`physics::wheel_surfaces`).
    pub surface_softness: PlainFloat,
    /// (`physics::drive_wheels_step`).
    pub drive_limits: [PlainFloat; 3],
    /// (`physics::drive_grip_forces`).
    pub grip_limit: PlainFloat,
    /// (`physics::drive_wheels_step`).
    pub slip_limit: PlainFloat,
    /// The four wheel loads' sum x `wheel_load_scale`.
    pub wheel_load: PlainFloat,
    /// The pitch and roll steps of a car settling (`race::car_starts`).
    pub settle_steps: [PlainDword; 2],
    /// The five points' loads (`physics::wheel_surfaces` writes all five; `physics::crash_contact`
    /// reads the fifth).
    pub wheel_loads: [PlainFloat; 5],
    /// The five points' grips.
    pub wheel_grips: [PlainFloat; 5],
    /// The five points' second figures.
    pub wheel_grips_b: [PlainFloat; 5],
    /// The five points' grips blended (`physics::wheel_surfaces`).
    pub point_grips: [PlainFloat; 5],
    /// Four of the surface records' figures blended for the five points.
    pub surface_blends: [PlainFloat; 20],
    /// The first surface of each point (`physics::wheel_surfaces`).
    pub surface_firsts: [PlainDword; 5],
    /// The second surface of each point.
    pub surface_seconds: [PlainDword; 5],
    /// `physics::find_clear_place`'s count, 0 when clear.
    pub clear_place_count: PlainDword,
    /// (`physics::drive_force`, `physics::free_rev`).
    pub drive_ratio: PlainFloat,
    /// (`physics::engine_torque`).
    pub engine_torque: PlainFloat,
    /// The engine's torque, 0 when cut.
    pub engine_force: PlainFloat,
    /// (`physics::drive_force`).
    pub drive_torque: PlainFloat,
    /// (`physics::drive_torque_share`).
    pub torque_share: PlainFloat,
    /// (`physics::ease_low_speed_forces`).
    pub low_speed_share: PlainFloat,
    /// (`physics::drive_wheels_step`).
    pub wheel_drag: [PlainFloat; 2],
    /// Held to 0 .. 1 (`physics::drive_grip_forces`, `physics::lever_push`).
    pub lever_share: PlainFloat,
    /// The spec's downforce figure (`physics::load_car_specs`).
    pub downforce_spec: PlainDword,
    /// Force records (0x20: a force, a point): 0..3 the wheels' weights, then springs, drive and
    /// grip (`physics::clear_forces`).
    pub forces: [ForceRecord; 23],
    /// The contacts' force records (`physics::clear_contacts`).
    pub contacts: [ForceRecord; 7],
    /// The push `physics::apply_force` takes.
    pub push_record: [PlainDword; 4],
    /// (`physics::drive_grip_forces`).
    pub grip_scale: PlainFloat,
    /// (`physics::drive_grip_forces`).
    pub side_grip_scale: PlainFloat,
    /// The spec's torque curve (`physics::load_car_specs`, `physics::engine_torque`).
    pub spec_torque: [PlainDword; 9],
    /// The spec's grip (`physics::drive_grip_forces`).
    pub spec_grip: PlainFloat,
    /// The spec's wheel figure (`physics::drive_wheels_step`).
    pub spec_wheel: PlainFloat,
    /// (`physics::drive_wheels_step`).
    pub spec_drive: [PlainFloat; 2],
    /// The spec's surface figures (`physics::wheel_surfaces`).
    pub spec_surface: [PlainFloat; 3],
    /// The spec's whole figures, scaled (`physics::load_car_specs`).
    pub spec_ints: [PlainDword; 3],
    /// The motion and spin kept (`physics::keep_motion_vectors`).
    pub kept_vectors: [PlainFloat; 8],
    /// `physics::set_handling`'s figures, by the set-up.
    pub handling: [PlainFloat; 9],
    /// Forward gears.
    pub gear_count: PlainDword,
    /// The spec's ride-height scale for the handling.
    pub ride_height_handling_scale: PlainDword,
    /// (`physics::drive_wheels_step`).
    pub axle_slip_limits: [PlainFloat; 2],
    /// The gear asked for (`physics::take_asked_gear`).
    pub gear_asked: PlainInt,
    /// (`physics::reset_car`).
    pub reset_offset: PlainDword,
    /// The checkpoint tried (`physics::reset_point_blocked`).
    pub reset_candidate: PlainDword,
    /// The distance to the nearest checkpoint.
    pub nearest_checkpoint_distance: PlainFloat,
    /// The distance to the split point.
    pub split_point_distance: PlainFloat,
    /// The slot 0 .. 7.
    pub pace_slot: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The starting values of `spec_default_bc0`.
#[rustfmt::skip]
const SPEC_DEFAULT_BC0: [u32; 3] = [
    0x76C, 0x640, 0x384,
];

/// The starting values of `surface_blend`.
#[rustfmt::skip]
const SURFACE_BLEND: [f32; 4] = [
    1.0, 0.9, 0.85, 1.0,
];

/// The starting values of `inertia`.
#[rustfmt::skip]
const INERTIA: [f32; 3] = [
    0.85, 0.75, 2.5,
];

/// The starting values of `rev_limits`.
#[rustfmt::skip]
const REV_LIMITS: [f32; 2] = [
    3000.0, 2000.0,
];

/// The 16 ground kinds' starting values: the table start, then the eleven figures after it,
/// in the order of `Surface`'s fields.
#[rustfmt::skip]
const SURFACE_ROWS: [(u32, [f32; 11]); 16] = [
    (0, [1.0, 0.9, 1.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
    (13, [1.0, 0.9, 1.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.07]),
    (26, [1.0, 0.9, 1.7, 0.0, 2.0, 0.0, 2.0, 0.0, 0.0, 1.0, 0.2]),
    (39, [1.0, 0.9, 1.7, 2e+01, 2.0, 2e+01, 2.0, 5.0, 3e+01, 0.5, 0.05]),
    (78, [1.0, 0.9, 1.2, 0.5, 2.0, 0.5, 2.0, 5.0, 4e+01, 0.0, 0.05]),
    (78, [1.0, 0.9, 1.7, 0.0, 0.0, 0.0, 0.0, 1e+01, 5.0, 0.0, 0.0]),
    (39, [1.0, 0.9, 1.7, 0.5, 5.0, 0.5, 5.0, 0.0, 0.0, 1.0, 0.05]),
    (65, [1.0, 0.9, 1.7, 1.0, 2.0, 1.0, 2.0, 1e+01, 25.0, 3.0, 0.1]),
    (65, [1.0, 0.9, 1.7, 1.0, 2.0, 1.0, 2.0, 1e+01, 15.0, 5.0, 1.0]),
    (91, [1.0, 0.9, 1.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.05]),
    (65, [1.0, 0.9, 1.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.05]),
    (78, [1.0, 0.9, 1.2, 0.5, 2.0, 0.5, 2.0, 5.0, 3e+01, 0.0, 0.1]),
    (52, [1.0, 0.9, 1.7, 0.5, 2.0, 0.5, 2.0, 0.0, 0.0, 0.75, 0.15]),
    (0, [1.0, 0.9, 1.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
    (0, [1.0, 0.9, 1.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
    (0, [1.0, 0.9, 1.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
];

impl Surface {
    /// A ground kind from its table start and the eleven figures after it (`SURFACE_ROWS`); the
    /// two unknown dwords hold floats.
    const fn from_row(table: u32, f: &[f32; 11]) -> Self {
        Self {
            table: PlainDword::new(table),
            slide_grip: PlainFloat::new(f[0]),
            accel_grip: PlainFloat::new(f[1]),
            brake_grip: PlainFloat::new(f[2]),
            mud_rates: [PlainFloat::new(f[3]), PlainFloat::new(f[4])],
            reserved_18: [
                PlainDword::new(f[5].to_bits()),
                PlainDword::new(f[6].to_bits()),
            ],
            drag: PlainFloat::new(f[7]),
            rolling_drag_min_speed: PlainFloat::new(f[8]),
            grip: PlainFloat::new(f[9]),
            wear_factor: PlainFloat::new(f[10]),
        }
    }

    /// The ground kinds of `rows`, in order.
    const fn rows(rows: &[(u32, [f32; 11]); 16]) -> [Self; 16] {
        let mut surfaces: [Self; 16] = ZeroCells::ZERO;
        let mut n = 0;
        while n < 16 {
            surfaces[n] = Self::from_row(rows[n].0, &rows[n].1);
            n += 1;
        }
        surfaces
    }
}

impl PhysicsTables {
    /// The piece, with the image's first values.
    pub const fn new() -> Self {
        Self {
            grip_slide_scale: PlainFloat::new(figures::GRIP_SLIDE_SCALE),
            grip_slip_limit: PlainFloat::new(figures::GRIP_SLIP_LIMIT),
            grip_force_limit: PlainFloat::new(figures::GRIP_FORCE_LIMIT),
            spec_scale_a4: PlainFloat::new(figures::SPEC_SCALE_A4),
            rev_limit_scale: PlainFloat::new(figures::REV_LIMIT_SCALE),
            drive_speed_scale: PlainFloat::new(figures::DRIVE_SPEED_SCALE),
            side_force_scale: PlainFloat::new(figures::SIDE_FORCE_SCALE),
            slip_blend: PlainFloat::new(figures::SLIP_BLEND),
            slip_share: PlainFloat::new(figures::SLIP_SHARE),
            wheel_speed_margin: PlainFloat::new(figures::WHEEL_SPEED_MARGIN),
            wheel_speed_gap: PlainFloat::new(figures::WHEEL_SPEED_GAP),
            sound_speed_scale: PlainFloat::new(figures::SOUND_SPEED_SCALE),
            spec_scale_1b8: PlainFloat::new(figures::SPEC_SCALE_1B8),
            grip_ramp: PlainFloat::new(figures::GRIP_RAMP),
            grip_ramp_share: PlainFloat::new(figures::GRIP_RAMP_SHARE),
            camera_drop: PlainFloat::new(figures::CAMERA_DROP),
            air_drag: PlainFloat::new(figures::AIR_DRAG),
            rolling_drag: PlainFloat::new(figures::ROLLING_DRAG),
            free_rev_add: PlainFloat::new(figures::FREE_REV_ADD),
            free_rev_scale: PlainFloat::new(figures::FREE_REV_SCALE),
            rev_top_scale: PlainFloat::new(figures::REV_TOP_SCALE),
            wheel_load_scale: PlainFloat::new(figures::WHEEL_LOAD_SCALE),
            camera_frame_scale: PlainFloat::new(figures::CAMERA_FRAME_SCALE),
            step_time: PlainFloat::new(figures::STEP_TIME),
            crash_depth: PlainDword::new(figures::CRASH_DEPTH),
            contact_steps: PlainDword::new(figures::CONTACT_STEPS),
            surfaces: Surface::rows(&SURFACE_ROWS),
            ground_spin_limit: PlainFloat::new(2.0),
            spec_default_bc0: PlainDword::array(SPEC_DEFAULT_BC0),
            surface_blend: PlainFloat::array(SURFACE_BLEND),
            inertia: PlainFloat::array(INERTIA),
            rev_limits: PlainFloat::array(REV_LIMITS),
            gravity: PlainFloat::new(9.8),
            ..ZeroCells::ZERO
        }
    }
}

impl PhysicsTables {
    /// The dwords of `figures::SURFACE_TABLES`.
    const TABLES_LEN: usize = figures::SURFACE_TABLES.len();

    /// Where ground kind `n`'s table starts in `figures::SURFACE_TABLES`, as a dword's number. A kind past
    /// the 16, or a start past the tables, is refused.
    pub(super) fn surface_table_index(&self, n: u32) -> Result<usize, crate::errors::BadInput> {
        let record = self
            .surfaces
            .get(n as usize)
            .ok_or(crate::errors::BadInput("a ground kind past the 16"))?;
        let at = record.table.get() as usize;
        if at >= Self::TABLES_LEN {
            return Err(crate::errors::BadInput(
                "a ground record's table off the tables",
            ));
        }
        Ok(at)
    }
}

#[cfg(test)]
mod tests {
    /// The tables' whole bytes, as a checksum: a wrong dword in a row shows here.
    #[test]
    fn the_physics_tables_keep_the_bytes_of_the_image() {
        let bytes = crate::memory::cells::flatten(&super::PhysicsTables::new());
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in &bytes {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
        assert_eq!(bytes.len(), 2376);
        assert_eq!(hash, 9_046_352_354_328_914_456);
    }

    #[test]
    fn the_cells_kept_for_the_contact_routines_hold_the_figures_constants() {
        let tables = super::PhysicsTables::new();
        assert_eq!(tables.step_time.get(), crate::physics::figures::STEP_TIME);
        assert_eq!(
            tables.crash_depth.get(),
            crate::physics::figures::CRASH_DEPTH
        );
        assert_eq!(
            tables.contact_steps.get(),
            crate::physics::figures::CONTACT_STEPS
        );
    }

    #[test]
    fn the_ground_records_name_whole_tables() {
        let shared = &crate::test_support::own::race().shared;
        for n in 0..16u32 {
            let slot = shared.physics_tables.get().surface_table_index(n).unwrap();
            assert!(slot.is_multiple_of(13) && slot < 104);
        }
        assert!(shared.physics_tables.get().surface_table_index(16).is_err());
    }
}
