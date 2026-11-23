//! The physics' tables and work variables: the ground kinds' records, the handling figures, the
//! car's working figures, the force and contact records and the spec's figures (the fixed
//! constants are `race::numbers_store::Numbers`; the race clock and the recorded input words are
//! in `physics::run_store`). Most names are readings from the code's uses.

use crate::physics::figures;

/// A kind of ground (0x30 bytes, `PhysicsTables.surfaces[0].table`): what `physics::wheel_surfaces`
/// blends between for each point, and `physics::apply_patches` patches (readings).
#[cfg_attr(test, repr(C))]
#[derive(Clone, Default, rc2k_derive::PlainRecord)]
pub struct Surface {
    /// Where its table starts in `figures::SURFACE_TABLES`, as a dword's number (a multiple of 13).
    pub table: u32,
    /// Slide grip (`patchslidegrip`).
    pub slide_grip: f32,
    /// Acceleration grip (`patchaccelgrip`).
    pub accel_grip: f32,
    /// Brake grip (`patchbrakegrip`).
    pub brake_grip: f32,
    /// How much mud the surface lets the wheels throw. Blended per point into `surface_blends`; the
    /// centre point's go to `mud_scale_a` ([0] -> throws, scaled by the drive speed; [1] ->
    /// splashes, scaled by the slip; `physics::accumulate_mud`). Probable.
    pub mud_rates: [f32; 2],
    /// No code reads or writes it (data only); kept so the record keeps its 0x30 bytes, as the
    /// recorded vectors lay it out.
    pub reserved_18: [u32; 2],
    /// The drag (10.0 on two grasses, `patchstickygrass`).
    pub drag: f32,
    /// The speed over the ground above which the rolling drag applies (the centre point's blend is
    /// `wheel_drag[1]`, compared in `drive_wheels_step::speed_over_ground`).
    pub rolling_drag_min_speed: f32,
    /// The grip blended into `point_grips`.
    pub grip: f32,
    /// How much a hit on this ground wears the car: blended per point into the car's
    /// `point_wear_factor`; the centre point's multiplies the hit figures of the damage model
    /// (`damage_wear::hit_figures`). Probable.
    pub wear_factor: f32,
}

/// A force on the car (0x20 bytes): `physics::apply_force` takes it, the step clears its
/// first six dwords (`physics::clear_records`).
#[cfg_attr(test, repr(C))]
#[derive(Clone, Default, rc2k_derive::PlainRecord)]
pub struct ForceRecord {
    /// The force.
    pub force: [f32; 3],
    /// Where it acts.
    pub point: [f32; 3],
}

#[cfg_attr(test, repr(C))]
#[derive(Clone, Default, rc2k_derive::PlainRecord)]
pub struct PhysicsTables {
    /// 0.7 (`physics::update_camera`, the car's shadow).
    pub camera_drop: f32,
    /// 10: the contact steps' limit (`figures::CONTACT_STEPS`, the same value).
    pub contact_steps: u32,
    /// The ground kinds' records (`physics::wheel_surfaces`; twelve patched by
    /// `physics::apply_patches`).
    pub surfaces: [Surface; 16],
    /// 2.0 (`physics::hold_spin_on_ground`).
    pub ground_spin_limit: f32,
    /// 0x76C, 0x640, 0x384, scaled into `spec_ints` (`physics::load_car_specs`).
    pub spec_default_bc0: [u32; 3],
    /// 3000.0 then one more, by spec (`physics::load_car_specs`, by drive kind 0 .. 2: kind 2 reads
    /// the next field, `camera_drop`: a slip kept as is).
    pub rev_limits: [f32; 2],
    /// 9.8 (`physics::apply_patches`).
    pub gravity: f32,
    /// The motion kept by `physics::keep_motion`.
    pub kept_motion: [f32; 3],
    /// `physics::stall_when_still`'s.
    pub stall_figure: f32,
    /// Two shares held to 0 .. 1 and their blend (`physics::drive_wheels_step`).
    pub drive_shares: [f32; 3],
    /// The gear ratios from gear -1 (reverse) to 7 (`physics::make_gear_ratios`).
    pub gear_ratios: [f32; 9],
    /// The engine's lowest speed (`physics::load_car_specs`).
    pub idle_engine_speed: f32,
    /// The idle speed + a step, the drop to neutral below it.
    pub stall_engine_speed: f32,
    /// The motion's limit (`physics::limit_motion`).
    pub top_speed: f32,
    /// The engine's highest speed.
    pub rev_limit: f32,
    /// The face `physics::find_clear_place` logs.
    pub clear_place_face: u32,
    /// The two surfaces' scales by tyre kind (`physics::wheel_surfaces`).
    pub surface_scales: [f32; 2],
    /// The car's vertical speed kept (`physics::car_physics_step`).
    pub vertical_speed: f32,
    /// `physics::pick_start_gear`'s: 100, 0, the engine speed it began with.
    pub start_gear_figures: [f32; 3],
    /// Steps since a contact (`physics::car_physics_step`, `physics::apply_contact_torques`).
    pub contact_step: u32,
    /// 1 for each point on hard ground (`physics::wheel_surfaces`).
    pub hard_ground: [u32; 5],
    /// `physics::drive_grip_forces`' integer.
    pub grip_drive_word: i32,
    /// (`physics::cap_drive_speed`).
    pub drive_speed_figure: f32,
    /// The `noautoneutral` setting.
    pub no_auto_neutral: u32,
    /// The car's weight factor (`physics::race_timers`, `physics::set_wheel_weights`).
    pub weight: f32,
    /// (`physics::set_downforce`).
    pub downforce: f32,
    /// The two axles' pushes (`physics::drive_wheels_step`, `physics::wheel_springs`).
    pub wheel_spin_push: [f32; 2],
    /// The car's place saved (`physics::restore_saved_car`).
    pub saved_place: [u32; 3],
    /// The car's matrix saved.
    pub saved_matrix: [f32; 16],
    /// (`physics::drive_wheels_step`).
    pub wheel_speed_step: f32,
    /// (`physics::wheel_surfaces`).
    pub surface_softness: f32,
    /// (`physics::drive_wheels_step`).
    pub drive_limits: [f32; 3],
    /// (`physics::drive_grip_forces`).
    pub grip_limit: f32,
    /// (`physics::drive_wheels_step`).
    pub slip_limit: f32,
    /// The four wheel loads' sum x `wheel_load_scale`.
    pub wheel_load: f32,
    /// The pitch and roll steps of a car settling (`race::car_starts`).
    pub settle_steps: [u32; 2],
    /// The five points' loads (`physics::wheel_surfaces` writes all five; `physics::crash_contact`
    /// reads the fifth).
    pub wheel_loads: [f32; 5],
    /// The five points' grips.
    pub wheel_grips: [f32; 5],
    /// The five points' second figures.
    pub wheel_grips_b: [f32; 5],
    /// The five points' grips blended (`physics::wheel_surfaces`).
    pub point_grips: [f32; 5],
    /// Four of the surface records' figures blended for the five points.
    pub surface_blends: [f32; 20],
    /// The first surface of each point (`physics::wheel_surfaces`).
    pub surface_firsts: [u32; 5],
    /// The second surface of each point.
    pub surface_seconds: [u32; 5],
    /// `physics::find_clear_place`'s count, 0 when clear.
    pub clear_place_count: u32,
    /// (`physics::drive_force`, `physics::free_rev`).
    pub drive_ratio: f32,
    /// (`physics::engine_torque`).
    pub engine_torque: f32,
    /// The engine's torque, 0 when cut.
    pub engine_force: f32,
    /// (`physics::drive_force`).
    pub drive_torque: f32,
    /// (`physics::drive_torque_share`).
    pub torque_share: f32,
    /// (`physics::ease_low_speed_forces`).
    pub low_speed_share: f32,
    /// (`physics::drive_wheels_step`).
    pub wheel_drag: [f32; 2],
    /// Held to 0 .. 1 (`physics::drive_grip_forces`, `physics::lever_push`).
    pub lever_share: f32,
    /// The spec's downforce figure (`physics::load_car_specs`).
    pub downforce_spec: u32,
    /// Force records (0x20: a force, a point): 0..3 the wheels' weights, then springs, drive and
    /// grip (`physics::clear_forces`).
    pub forces: [ForceRecord; 23],
    /// The contacts' force records (`physics::clear_contacts`).
    pub contacts: [ForceRecord; 7],
    /// (`physics::drive_grip_forces`).
    pub grip_scale: f32,
    /// (`physics::drive_grip_forces`).
    pub side_grip_scale: f32,
    /// The spec's torque curve (`physics::load_car_specs`, `physics::engine_torque`).
    pub spec_torque: [u32; 9],
    /// The spec's grip (`physics::drive_grip_forces`).
    pub spec_grip: f32,
    /// The spec's wheel figure (`physics::drive_wheels_step`).
    pub spec_wheel: f32,
    /// (`physics::drive_wheels_step`).
    pub spec_drive: [f32; 2],
    /// The spec's surface figures (`physics::wheel_surfaces`).
    pub spec_surface: [f32; 3],
    /// The spec's whole figures, scaled (`physics::load_car_specs`).
    pub spec_ints: [u32; 3],
    /// The motion and spin kept (`physics::keep_motion_vectors`).
    pub kept_vectors: [f32; 8],
    /// `physics::set_handling`'s figures, by the set-up.
    pub handling: [f32; 9],
    /// Forward gears.
    pub gear_count: u32,
    /// The spec's ride-height scale for the handling.
    pub ride_height_handling_scale: u32,
    /// (`physics::drive_wheels_step`).
    pub axle_slip_limits: [f32; 2],
    /// The gear asked for (`physics::take_asked_gear`).
    pub gear_asked: i32,
    /// (`physics::reset_car`).
    pub reset_offset: u32,
    /// The checkpoint tried (`physics::reset_point_blocked`).
    pub reset_candidate: u32,
    /// The distance to the nearest checkpoint.
    pub nearest_checkpoint_distance: f32,
    /// The distance to the split point.
    pub split_point_distance: f32,
    /// The slot 0 .. 7.
    pub pace_slot: u32,
}

/// The starting values of `spec_default_bc0`.
#[rustfmt::skip]
const SPEC_DEFAULT_BC0: [u32; 3] = [
    0x76C, 0x640, 0x384,
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
    fn from_row(table: u32, f: &[f32; 11]) -> Self {
        Self {
            table,
            slide_grip: f[0],
            accel_grip: f[1],
            brake_grip: f[2],
            mud_rates: [f[3], f[4]],
            reserved_18: [f[5].to_bits(), f[6].to_bits()],
            drag: f[7],
            rolling_drag_min_speed: f[8],
            grip: f[9],
            wear_factor: f[10],
        }
    }
}

impl PhysicsTables {
    /// The piece, with the image's first values.
    pub fn new() -> Self {
        Self {
            camera_drop: figures::CAMERA_DROP,
            contact_steps: figures::CONTACT_STEPS,
            surfaces: SURFACE_ROWS
                .each_ref()
                .map(|(table, f)| Surface::from_row(*table, f)),
            ground_spin_limit: 2.0,
            spec_default_bc0: SPEC_DEFAULT_BC0,
            rev_limits: REV_LIMITS,
            gravity: 9.8,
            ..Self::default()
        }
    }
}

/// A force record of the tables, by its place.
#[derive(Clone, Copy)]
pub enum RecordAt {
    /// `forces[n]`.
    Force(usize),
    /// `contacts[n]`.
    Contact(usize),
}

impl PhysicsTables {
    /// The record at `at`, to write.
    pub fn record_mut(&mut self, at: RecordAt) -> &mut ForceRecord {
        match at {
            RecordAt::Force(n) => &mut self.forces[n],
            RecordAt::Contact(n) => &mut self.contacts[n],
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
        let at = record.table as usize;
        if at >= Self::TABLES_LEN {
            return Err(crate::errors::BadInput(
                "a ground record's table off the tables",
            ));
        }
        Ok(at)
    }
}

#[cfg(test)]
mod old_cells;
#[cfg(test)]
mod tests;
#[cfg(test)]
pub mod words;
