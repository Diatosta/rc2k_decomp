//! The physics' tables and work variables: the race clock, the inputs, the ground kinds' records,
//! the handling constants, the car's working figures, the force and contact records, the gearbox
//! and the spec's figures. Most names are readings from the code's uses; `reserved_*` runs are
//! places no ported code names.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFlag, PlainFloat, PlainInt};

/// A kind of ground (0x30 bytes, `PhysicsTables.surfaces[0].table`): what `physics::wheel_surfaces`
/// blends between for each point, and `physics::apply_patches` patches (readings).
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct Surface {
    /// Where its table starts in `surface_tables`, as a dword's number (a multiple of 13).
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
    /// No code reads or writes it (data only).
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
    /// Not cleared; no code reads or writes it (data only).
    pub reserved_18: [PlainDword; 2],
}

#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct PhysicsTables {
    /// 5.0: a reset is allowed below this speed (`drive::reset_request`).
    pub reset_speed_limit: PlainFloat,
    /// Unused.
    pub reserved_4: PlainDword,
    /// The scene's clock, read by everything timed.
    pub scene_clock: PlainDword,
    /// Unused.
    pub reserved_c: PlainDword,
    /// Frames counted by `race::measure_frame_rate`.
    pub frame_rate_count: PlainDword,
    /// The race clock's last reading (`drive::race_clock_tick`).
    pub race_clock_now: PlainDword,
    /// The clock's count the race's time is taken from (`drive::race_clock_tick`, the films).
    pub race_clock_base: PlainDword,
    /// The race's ticks to run (`drive::race_clock_tick` -> `drive::race_step`).
    pub race_tenths: PlainDword,
    /// Unused.
    pub reserved_20: [PlainDword; 17],
    /// 0.01: the physics step's length in seconds.
    pub step_time: PlainFloat,
    /// The steering input (`drive::read_car_controls`, `drive::steer_inputs`).
    pub input_steer: PlainDword,
    /// The throttle input.
    pub input_throttle: PlainDword,
    /// The brake input.
    pub input_brake: PlainDword,
    /// The handbrake input (500 at the start line, `physics::gear_input_rules`).
    pub input_handbrake: PlainDword,
    /// The gear-up key (`physics::manual_gearbox`).
    pub input_gear_up: PlainDword,
    /// The gear-down key.
    pub input_gear_down: PlainDword,
    /// The gear asked for, -1 none (`physics::auto_gearbox`).
    pub input_gear_wanted: PlainDword,
    /// Three of the six are printed by the developers' wear page (`debug::wear_page`); nothing else
    /// uses them.
    pub wear_page_words: [PlainDword; 6],
    /// The four input words recorded and replayed (`drive::record_inputs`, `drive::play_inputs`).
    pub input_words: [PlainDword; 4],
    /// A counter by camera view (`race::frame`).
    pub view_counters: [PlainDword; 7],
    /// Unused.
    pub reserved_c8: PlainDword,
    /// The developers' debug overlay's page (0 .. 7), and the freeze-race flag (`drive::race_step`
    /// skips the physics ticks while set): what is left of the debug keys; `input::end_frame` sets
    /// both 0 every frame (`debug::overlay`).
    pub debug: [PlainDword; 2],
    /// The developers' debug keys' values (probably): each key held copies one of these dwords into
    /// a gauge of the first car block (`engine_damage` ..); no code names them.
    pub debug_key_values: [PlainDword; 31],
    /// The ground kinds' records (`physics::wheel_surfaces`; twelve patched by
    /// `physics::apply_patches`).
    pub surfaces: [Surface; 16],
    /// Eight tables of 13 dwords the surface records point at: [0] then six figures by tyre kind
    /// and six more (`physics::wheel_surfaces`).
    pub surface_tables: [PlainDword; 104],
    /// Unused.
    pub reserved_5f0: [PlainDword; 10],
    /// 8.0: below it the car may stall (`physics::stall_when_still`).
    pub stall_speed: PlainFloat,
    /// 0.5 (`physics::stall_when_still`).
    pub stall_share: PlainFloat,
    /// 800.0 (`physics::cap_drive_speed`).
    pub drive_speed_cap: PlainFloat,
    /// 20.0 (`physics::cap_drive_speed`).
    pub drive_speed_cap_a: PlainFloat,
    /// 20.0 (`physics::cap_drive_speed`).
    pub drive_speed_cap_b: PlainFloat,
    /// 2.4 / 1.6 by drive kind 1 or not (`physics::drive_grip_forces`).
    pub grip_front_drive: [PlainFloat; 2],
    /// 0.5 / 0.4 by drive kind 1 or not (`physics::drive_grip_forces`).
    pub grip_rear_drive: [PlainFloat; 2],
    /// 20.0 (`physics::ease_low_speed_forces`).
    pub low_speed_limit: PlainFloat,
    /// 1.0 (`physics::ease_low_speed_forces`).
    pub low_speed_add: PlainFloat,
    /// 0.5 (`physics::ease_low_speed_forces`).
    pub low_speed_scale: PlainFloat,
    /// 4.0: the spin held to it (`physics::limit_motion`).
    pub spin_limit: PlainFloat,
    /// 80.0: the motion held to it (`physics::limit_motion`).
    pub velocity_limit: PlainFloat,
    /// 7.5 (`physics::drive_grip_forces`).
    pub grip_slide_scale: PlainFloat,
    /// 5.0 (`physics::drive_grip_forces`).
    pub grip_slip_limit: PlainFloat,
    /// 10.0, held both ways (`physics::drive_grip_forces`).
    pub grip_force_limit: PlainFloat,
    /// 1.0 (`physics::drive_grip_forces`).
    pub grip_slow_speed: PlainFloat,
    /// 0.9 (`physics::load_car_specs`).
    pub spec_scale_a4: PlainFloat,
    /// 0.15 (`physics::probe_ground`).
    pub probe_lift: PlainFloat,
    /// Unused.
    pub reserved_668: PlainDword,
    /// 1.0 (`physics::auto_gearbox`).
    pub auto_gear_speed_up: PlainFloat,
    /// -1.0 (`physics::auto_gearbox`).
    pub auto_gear_speed_down: PlainFloat,
    /// 0.95: the engine's speed eased by it (`physics::gearbox_step`).
    pub engine_ease: PlainFloat,
    /// 2.0 (`physics::hold_spin_on_ground`).
    pub ground_spin_limit: PlainFloat,
    /// 0.1 (`physics::drive_grip_forces`).
    pub grip_ease_step: PlainFloat,
    /// Unused.
    pub reserved_680: PlainDword,
    /// 2.0 / 2.2 by drive kind (`physics::drive_grip_forces`).
    pub grip_factor: [PlainFloat; 2],
    /// 0.5 (`physics::drive_grip_forces`).
    pub side_grip_share: PlainFloat,
    /// 8500.0 (`physics::load_car_specs`).
    pub rev_limit_scale: PlainFloat,
    /// 0.65 (`physics::spin_wheels`).
    pub wheel_spin_share: PlainFloat,
    /// 0.325 (`physics::wheel_springs`).
    pub spring_share: PlainFloat,
    /// 2.5 (`physics::drive_wheels_step`).
    pub drive_speed_scale: PlainFloat,
    /// 15.0, held both ways (`physics::car_physics_step`).
    pub side_speed_limit: PlainFloat,
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
    /// 0.0045 / 0.0025 (`physics::add_drive_forces`).
    pub drive_force_factor: [PlainFloat; 2],
    /// 8 (`physics::drive_torque_share`).
    pub torque_share_a: PlainDword,
    /// 0x20 (`physics::drive_torque_share`).
    pub torque_share_b: PlainDword,
    /// 32.0's bits, divided by as an integer (`physics::drive_torque_share`).
    pub torque_share_c: PlainInt,
    /// 0x1000 (`physics::crash_contact`).
    pub crash_depth: PlainDword,
    /// 2.4, copied into the spec's grip (`physics::load_car_specs`).
    pub spec_default_ba0: PlainFloat,
    /// 0.5, copied into the spec's second grip.
    pub spec_default_ba4: PlainFloat,
    /// 0x76C, 0x640, 0x384, scaled into `spec_ints` (`physics::load_car_specs`).
    pub spec_default_bc0: [PlainDword; 3],
    /// 0.85 (`physics::load_car_specs`).
    pub spec_scale_1b8: PlainFloat,
    /// 4.0, copied into the spec's wheel figure.
    pub spec_default_ba8: PlainFloat,
    /// 0.95 (`physics::auto_gearbox`).
    pub auto_gear_engine_share: PlainFloat,
    /// `physics::set_handling`'s (lo, hi) pairs for three of its figures.
    pub handling_ranges_a: [PlainFloat; 6],
    /// 2.0 (`physics::drive_grip_forces`).
    pub grip_ramp: PlainFloat,
    /// 0.25 (`physics::drive_grip_forces`).
    pub grip_ramp_share: PlainFloat,
    /// 1.0 (`physics::settle_drive_speeds`).
    pub settle_fast_step: PlainFloat,
    /// 0.01 (`physics::settle_drive_speeds`).
    pub settle_slow_step: PlainFloat,
    /// `physics::set_handling`'s pairs for two more figures (the second chosen by kind).
    pub handling_ranges_b: [PlainFloat; 6],
    /// 1.0, 0.9, 0.85, 1.0 (`physics::wheel_surfaces`).
    pub surface_blend: [PlainFloat; 4],
    /// `physics::set_handling`'s pairs for four more figures.
    pub handling_ranges_c: [PlainFloat; 8],
    /// 0.5 (`physics::probe_ground`, `physics::wheel_springs`).
    pub probe_share: PlainFloat,
    /// 2.0 (`physics::wheel_springs`).
    pub spring_force_scale: PlainFloat,
    /// 0.25 (`physics::engine_torque`).
    pub torque_ease: PlainFloat,
    /// Unused.
    pub reserved_770: [PlainDword; 3],
    /// 5: the steps the drive is cut after a change (`physics::gearbox`).
    pub drive_cut_steps: PlainDword,
    /// 0.85, 0.75, 2.5: the car's turning inertia by axis (`physics::apply_torque`).
    pub inertia: [PlainFloat; 3],
    /// 0.0001 (`physics::set_downforce`).
    pub downforce_factor: PlainFloat,
    /// 3000.0 then one more, by spec (`physics::load_car_specs`, by drive kind 0 .. 2: kind 2 reads
    /// the next field, `camera_drop`: a slip kept as is).
    pub rev_limits: [PlainFloat; 2],
    /// 0.7 (`physics::update_camera`, the car's shadow).
    pub camera_drop: PlainFloat,
    /// 0xB333: the car's places dropped by it (`race::start_place`, `scene::add_object_shape`).
    pub ground_drop: PlainDword,
    /// 9.8 (`physics::apply_patches`).
    pub gravity: PlainFloat,
    /// 5.0 (`physics::air_drag`, the test drive).
    pub air_drag: PlainFloat,
    /// 0.75 (the test drive).
    pub rolling_drag: PlainFloat,
    /// 5.0 (`physics::free_rev`).
    pub free_rev_add: PlainFloat,
    /// 0.005 (`physics::free_rev`).
    pub free_rev_scale: PlainFloat,
    /// 0.9 (`physics::find_clear_place`).
    pub clear_place_share: PlainFloat,
    /// 0.99 (`physics::pick_start_gear`).
    pub start_gear_share: PlainFloat,
    /// 1.2 (`physics::load_car_specs`).
    pub rev_top_scale: PlainFloat,
    /// 0.001 (`physics::make_gear_ratios`).
    pub gear_ratio_step: PlainFloat,
    /// 0.0025 (`physics::wheel_surfaces`).
    pub wheel_load_scale: PlainFloat,
    /// The engine's torque curve read in pairs (`physics::engine_torque`).
    pub torque_curve: [PlainFloat; 9],
    /// `physics::find_clear_place`'s first step table.
    pub clear_place_steps_a: [PlainDword; 16],
    /// Its second step table.
    pub clear_place_steps_b: [PlainDword; 16],
    /// 0.02, held both ways (`physics::camera_frame`).
    pub camera_frame_limit: PlainFloat,
    /// 0.05 (`physics::camera_frame`).
    pub camera_frame_scale: PlainFloat,
    /// The motion kept by `physics::keep_motion`.
    pub kept_motion: [PlainFloat; 3],
    /// Unused.
    pub reserved_880: PlainDword,
    /// `physics::stall_when_still`'s.
    pub stall_figure: PlainFloat,
    /// Two shares held to 0 .. 1 and their blend (`physics::drive_wheels_step`).
    pub drive_shares: [PlainFloat; 3],
    /// The gear ratios from gear -1 (reverse) to 7 (`physics::make_gear_ratios`).
    pub gear_ratios: [PlainFloat; 9],
    /// Unused.
    pub reserved_8b8: PlainDword,
    /// The two lever pushes (`physics::lever_push`).
    pub lever_pushes: [PlainDword; 2],
    /// Unused (the image's two time marks).
    pub reserved_8c4: [PlainDword; 2],
    /// `physics::add_drive_forces`'.
    pub drive_force_scale: PlainFloat,
    /// The engine's lowest speed (`physics::load_car_specs`).
    pub idle_engine_speed: PlainFloat,
    /// The idle speed + a step, the drop to neutral below it.
    pub stall_engine_speed: PlainFloat,
    /// The motion's limit (`physics::limit_motion`).
    pub top_speed: PlainFloat,
    /// The engine's highest speed.
    pub rev_limit: PlainFloat,
    /// Hits of scene points on the car this step.
    pub scene_point_hits: PlainDword,
    /// Hits of objects on the car this step.
    pub object_hits: PlainDword,
    /// Hits of points on the car this step.
    pub point_hits: PlainDword,
    /// The count-down's last beep (`drive::countdown_beep`).
    pub countdown_sound: PlainDword,
    /// The face `physics::find_clear_place` logs.
    pub clear_place_face: PlainDword,
    /// The two surfaces' scales by tyre kind (`physics::wheel_surfaces`).
    pub surface_scales: [PlainFloat; 2],
    /// The wheels' spring damping (`physics::wheel_springs`).
    pub spring_damping: [PlainFloat; 4],
    /// 0 / 1 (`physics::drive_wheels_step`).
    pub wheels_driven: PlainDword,
    /// The car's vertical speed kept (`physics::car_physics_step`).
    pub vertical_speed: PlainFloat,
    /// `physics::pick_start_gear`'s: 100, 0, the engine speed it began with.
    pub start_gear_figures: [PlainFloat; 3],
    /// Steps since a contact (`physics::car_physics_step`, `physics::apply_contact_torques`).
    pub contact_step: PlainDword,
    /// 10: the contact steps' limit.
    pub contact_steps: PlainDword,
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
    /// The wheels' spring forces (`physics::wheel_springs`).
    pub spring_forces: [PlainFloat; 4],
    /// Unused.
    pub reserved_960: [PlainDword; 9],
    /// The wall edge's two points' x and z (`physics::wall_edge_contact`).
    pub edge_points: [PlainDword; 4],
    /// The edge crossed from / to (`physics::crossed_edge_contact`).
    pub crossed_edge: [PlainDword; 2],
    /// Unused.
    pub reserved_99c: PlainDword,
    /// The wall's heading, -1 none.
    pub wall_heading: PlainDword,
    /// Unused.
    pub reserved_9a4: PlainDword,
    /// Physics steps counted (`physics::car_physics_step`).
    pub step_count: PlainDword,
    /// The two axles' pushes (`physics::drive_wheels_step`, `physics::wheel_springs`).
    pub wheel_spin_push: [PlainFloat; 2],
    /// (`physics::feel_ground`).
    pub feel_word: PlainDword,
    /// The car's place saved (`physics::restore_saved_car`).
    pub saved_place: [PlainDword; 3],
    /// The car's matrix saved.
    pub saved_matrix: [PlainFloat; 16],
    /// (`physics::drive_wheels_step`).
    pub wheel_speed_step: PlainFloat,
    /// (`physics::wheel_surfaces`).
    pub surface_softness: PlainFloat,
    /// |front / rear wheel speed - the car's| (`physics::drive_wheels_step`).
    pub wheel_slips: [PlainFloat; 2],
    /// (`physics::drive_wheels_step`).
    pub drive_limits: [PlainFloat; 3],
    /// The two axles' drive forces.
    pub drive_forces: [PlainFloat; 2],
    /// The drive's slip and its eased value.
    pub drive_slip: [PlainFloat; 2],
    /// The two axles' slips.
    pub axle_slips: [PlainFloat; 2],
    /// (`physics::drive_grip_forces`).
    pub grip_limit: PlainFloat,
    /// (`physics::drive_wheels_step`).
    pub slip_limit: PlainFloat,
    /// The four wheel loads' sum x `wheel_load_scale`.
    pub wheel_load: PlainFloat,
    /// The pitch and roll steps of a car settling (`race::car_starts`).
    pub settle_steps: [PlainDword; 2],
    /// (`physics::apply_torque`).
    pub torque_cross: [PlainFloat; 3],
    /// Unused.
    pub reserved_a58: PlainDword,
    /// The five points' loads (`physics::wheel_surfaces` writes all five; `physics::crash_contact`
    /// reads the fifth).
    pub wheel_loads: [PlainFloat; 5],
    /// The five points' grips.
    pub wheel_grips: [PlainFloat; 5],
    /// The five points' second figures.
    pub wheel_grips_b: [PlainFloat; 5],
    /// The five points' grips blended (`physics::wheel_surfaces`).
    pub point_grips: [PlainFloat; 5],
    /// Unused.
    pub reserved_aac: [PlainDword; 251],
    /// Four of the surface records' figures blended for the five points.
    pub surface_blends: [PlainFloat; 20],
    /// The first surface of each point (`physics::wheel_surfaces`).
    pub surface_firsts: [PlainDword; 5],
    /// The second surface of each point.
    pub surface_seconds: [PlainDword; 5],
    /// `physics::find_clear_place`'s count, 0 when clear.
    pub clear_place_count: PlainDword,
    /// Its tries.
    pub clear_place_tries: PlainDword,
    /// (`physics::drive_force`, `physics::free_rev`).
    pub drive_ratio: PlainFloat,
    /// (`physics::engine_torque`).
    pub engine_torque: PlainFloat,
    /// The engine's torque, 0 when cut.
    pub engine_force: PlainFloat,
    /// (the test drive).
    pub test_drive_figure: PlainFloat,
    /// (`physics::drive_force`).
    pub drive_torque: PlainFloat,
    /// Unused.
    pub reserved_f2c: PlainDword,
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
    /// `physics::contact_force`'s record. Not used: the callers pass the record to it.
    pub contact_record: PlainDword,
    /// `physics::lever_push`'s vector.
    pub lever_push_vector: [PlainDword; 4],
    /// The push turned by the car's matrix.
    pub lever_push_turned: [PlainDword; 4],
    /// The place turned by the inverse.
    pub lever_push_place: [PlainFloat; 4],
    /// (`physics::drive_grip_forces`).
    pub grip_scale: PlainFloat,
    /// (`physics::drive_grip_forces`).
    pub side_grip_scale: PlainFloat,
    /// The spec's torque curve (`physics::load_car_specs`, `physics::engine_torque`).
    pub spec_torque: [PlainDword; 9],
    /// The spec's grip (`physics::drive_grip_forces`).
    pub spec_grip: PlainFloat,
    /// The spec's second grip.
    pub spec_grip_b: PlainFloat,
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
    /// A spec figure that nothing reads (`physics::load_car_specs`).
    pub unused_spec_20: PlainDword,
    /// The spec's ride-height scale for the handling.
    pub ride_height_handling_scale: PlainDword,
    /// (`physics::drive_wheels_step`).
    pub axle_slip_limits: [PlainFloat; 2],
    /// (`physics::hold_spin_on_ground`).
    pub ground_spin: PlainFlag,
    /// (`physics::find_clear_place`).
    pub clear_place_mark: PlainDword,
    /// `physics::sum_forces`' five sums.
    pub force_sums: [PlainFloat; 5],
    /// The gear asked for (`physics::take_asked_gear`).
    pub gear_asked: PlainInt,
    /// (`physics::lever_pushes`).
    pub lever_share_start: PlainFloat,
    /// (`physics::lever_pushes`).
    pub lever_share_left: PlainFloat,
    /// (`physics::reset_car`).
    pub reset_offset: PlainDword,
    /// The checkpoint tried (`physics::reset_point_blocked`).
    pub reset_candidate: PlainDword,
    /// The noise's state (`scene::next_noise`).
    pub noise_state: PlainDword,
    /// Unused.
    pub reserved_1430: [PlainDword; 3],
    /// (`physics::blend_wheel_load`).
    pub wheel_load_limit: PlainFloat,
    /// The distance to the nearest checkpoint.
    pub nearest_checkpoint_distance: PlainFloat,
    /// (`physics::off_line_warnings`).
    pub off_line_distance: PlainFloat,
    /// (`physics::passed_split_point`).
    pub split_distance: PlainFloat,
    /// (`physics::off_line_warnings`).
    pub off_line_word: PlainDword,
    /// (`physics::pace_notes_step`).
    pub pace_scale: PlainFloat,
    /// The events listed (`physics::queue_listed_event`).
    pub listed_events: [PlainDword; 32],
    /// (`physics::passed_split_point`).
    pub split_word: PlainDword,
    /// (`physics::passed_split_point`).
    pub split_heading: PlainDword,
    /// The distance to the split point.
    pub split_point_distance: PlainFloat,
    /// Unused: the checkpoint's near and far colours and the distance were kept here
    /// (`race::frame_draw`); no code reads or writes it.
    pub reserved_14e0: [PlainDword; 3],
    /// Each slot's byte in the face.
    pub pace_slot_offsets: [PlainDword; 8],
    /// The slot 0 .. 7.
    pub pace_slot: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The starting values of `view_counters`.
#[rustfmt::skip]
const VIEW_COUNTERS: [u32; 7] = [
    0x10C, 0x10D, 0x10E, 0x10F, 0x110, 0x111, 0x112,
];

/// The starting values of `debug_key_values`.
#[rustfmt::skip]
const DEBUG_KEY_VALUES: [u32; 31] = [
    0x42C80000, 0x42C80000, 0x42C80000, 0x42C80000, 0x42C80000, 0x42C80000, 0x42C80000, 0x42C80000,
    0x42C80000, 0x42C80000, 0x42C80000, 0x42C80000, 0x42C80000, 0x42C80000, 0x42C80000, 0x42C80000,
    0x42C60000, 0x42C80000, 0x3F800000, 0x3F800000, 0x3F800000, 0x3F800000, 0x3F800000, 0x3F800000,
    0x3F800000, 0x3F800000, 0x42C80000, 0x42C80000, 0x42C80000, 0x42C80000, 0x42C80000,
];

/// The starting values of `surface_tables`.
#[rustfmt::skip]
const SURFACE_TABLES: [u32; 104] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 120, 120, 115, 105, 90, 80, 75, 80, 95, 105, 85, 75, 75,
    85, 65, 70, 75, 85, 80, 65, 60, 65, 70, 75, 80, 65, 75, 45, 50, 55, 75, 70, 55, 40, 45, 50, 65,
    70, 55, 75, 40, 45, 50, 65, 70, 75, 40, 45, 50, 65, 70, 75, 70, 40, 45, 50, 70, 65, 50, 35, 40,
    45, 60, 65, 50, 75, 35, 40, 45, 60, 65, 75, 35, 40, 45, 60, 65, 75, 65, 25, 30, 35, 50, 55, 65,
    25, 30, 35, 50, 55, 65,
];

/// The starting values of `reserved_5f0`.
#[rustfmt::skip]
const UNUSED_5F0: [u32; 10] = [
    0x1010000, 0x101, 0x1000000, 0x1010101, 0x101, 0, 0x1010101, 0x1010101, 0x1010101, 0x10101,
];

/// The starting values of `grip_front_drive`.
#[rustfmt::skip]
const GRIP_FRONT_DRIVE: [f32; 2] = [
    2.4, 1.6,
];

/// The starting values of `grip_rear_drive`.
#[rustfmt::skip]
const GRIP_REAR_DRIVE: [f32; 2] = [
    0.5, 0.4,
];

/// The starting values of `grip_factor`.
#[rustfmt::skip]
const GRIP_FACTOR: [f32; 2] = [
    2.0, 2.2,
];

/// The starting values of `drive_force_factor`.
#[rustfmt::skip]
const DRIVE_FORCE_FACTOR: [f32; 2] = [
    0.0045, 0.0025,
];

/// The starting values of `spec_default_bc0`.
#[rustfmt::skip]
const SPEC_DEFAULT_BC0: [u32; 3] = [
    0x76C, 0x640, 0x384,
];

/// The starting values of `handling_ranges_a`.
#[rustfmt::skip]
const HANDLING_RANGES_A: [f32; 6] = [
    0.1, 0.9, 0.5, 1.5, 0.25, 1.25,
];

/// The starting values of `handling_ranges_b`.
#[rustfmt::skip]
const HANDLING_RANGES_B: [f32; 6] = [
    0.1, 0.2, 0.3, 0.16, 0.4, 0.26,
];

/// The starting values of `surface_blend`.
#[rustfmt::skip]
const SURFACE_BLEND: [f32; 4] = [
    1.0, 0.9, 0.85, 1.0,
];

/// The starting values of `handling_ranges_c`.
#[rustfmt::skip]
const HANDLING_RANGES_C: [f32; 8] = [
    0.15, 0.1, 100.0, 200.0, 1.0, 2.0, 0.45, 0.3,
];

/// The starting values of `reserved_770`.
#[rustfmt::skip]
const UNUSED_770: [u32; 3] = [
    0x3D4CCCCD, 0x3D99999A, 0x400CCCCD,
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

/// The starting values of `torque_curve`.
#[rustfmt::skip]
const TORQUE_CURVE: [f32; 9] = [
    0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.0,
];

/// The starting values of `clear_place_steps_a`.
#[rustfmt::skip]
const CLEAR_PLACE_STEPS_A: [u32; 16] = [
    0, 7, 10, 7, 0, 0xFFFFFFF9, 0xFFFFFFF6, 0xFFFFFFF9, 0, 10, 14, 10, 0, 0xFFFFFFF6, 0xFFFFFFF2,
    0xFFFFFFF6,
];

/// The starting values of `clear_place_steps_b`.
#[rustfmt::skip]
const CLEAR_PLACE_STEPS_B: [u32; 16] = [
    10, 7, 0, 0xFFFFFFF9, 0xFFFFFFF6, 0xFFFFFFF9, 0, 7, 14, 10, 0, 0xFFFFFFF6, 0xFFFFFFF2,
    0xFFFFFFF6, 0, 10,
];

/// The starting values of `reserved_1430`.
#[rustfmt::skip]
const UNUSED_1430: [u32; 3] = [
    0xFE0, 0x1A10, 0x2810,
];

/// The starting values of `listed_events`.
#[rustfmt::skip]
const LISTED_EVENTS: [u32; 32] = [
    0xFFFFFFFF, 8, 9, 10, 11, 12, 13, 14, 15, 8, 9, 10, 11, 12, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
    0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 0xFFFFFFFF, 0xFFFFFFFF,
];

/// The starting values of `pace_slot_offsets`.
#[rustfmt::skip]
const PACE_SLOT_OFFSETS: [u32; 8] = [
    40, 41, 42, 43, 25, 26, 27, 32,
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
            reset_speed_limit: PlainFloat::new(5.0),
            step_time: PlainFloat::new(0.01),
            view_counters: PlainDword::array(VIEW_COUNTERS),
            debug_key_values: PlainDword::array(DEBUG_KEY_VALUES),
            surfaces: Surface::rows(&SURFACE_ROWS),
            surface_tables: PlainDword::array(SURFACE_TABLES),
            reserved_5f0: PlainDword::array(UNUSED_5F0),
            stall_speed: PlainFloat::new(8.0),
            stall_share: PlainFloat::new(0.5),
            drive_speed_cap: PlainFloat::new(800.0),
            drive_speed_cap_a: PlainFloat::new(20.0),
            drive_speed_cap_b: PlainFloat::new(20.0),
            grip_front_drive: PlainFloat::array(GRIP_FRONT_DRIVE),
            grip_rear_drive: PlainFloat::array(GRIP_REAR_DRIVE),
            low_speed_limit: PlainFloat::new(20.0),
            low_speed_add: PlainFloat::new(1.0),
            low_speed_scale: PlainFloat::new(0.5),
            spin_limit: PlainFloat::new(4.0),
            velocity_limit: PlainFloat::new(80.0),
            grip_slide_scale: PlainFloat::new(7.5),
            grip_slip_limit: PlainFloat::new(5.0),
            grip_force_limit: PlainFloat::new(10.0),
            grip_slow_speed: PlainFloat::new(1.0),
            spec_scale_a4: PlainFloat::new(0.9),
            probe_lift: PlainFloat::new(0.15),
            reserved_668: PlainDword::new(0x3E4CCCCD),
            auto_gear_speed_up: PlainFloat::new(1.0),
            auto_gear_speed_down: PlainFloat::new(-1.0),
            engine_ease: PlainFloat::new(0.95),
            ground_spin_limit: PlainFloat::new(2.0),
            grip_ease_step: PlainFloat::new(0.1),
            reserved_680: PlainDword::new(0x3FB33333),
            grip_factor: PlainFloat::array(GRIP_FACTOR),
            side_grip_share: PlainFloat::new(0.5),
            rev_limit_scale: PlainFloat::new(8500.0),
            wheel_spin_share: PlainFloat::new(0.65),
            spring_share: PlainFloat::new(0.325),
            drive_speed_scale: PlainFloat::new(2.5),
            side_speed_limit: PlainFloat::new(15.0),
            side_force_scale: PlainFloat::new(0.05),
            slip_blend: PlainFloat::new(0.5),
            slip_share: PlainFloat::new(0.5),
            wheel_speed_margin: PlainFloat::new(1.5),
            wheel_speed_gap: PlainFloat::new(0.1),
            sound_speed_scale: PlainFloat::new(1.0),
            drive_force_factor: PlainFloat::array(DRIVE_FORCE_FACTOR),
            torque_share_a: PlainDword::new(8),
            torque_share_b: PlainDword::new(32),
            torque_share_c: PlainInt::new(0x42000000),
            crash_depth: PlainDword::new(0x1000),
            spec_default_ba0: PlainFloat::new(2.4),
            spec_default_ba4: PlainFloat::new(0.5),
            spec_default_bc0: PlainDword::array(SPEC_DEFAULT_BC0),
            spec_scale_1b8: PlainFloat::new(0.85),
            spec_default_ba8: PlainFloat::new(4.0),
            auto_gear_engine_share: PlainFloat::new(0.95),
            handling_ranges_a: PlainFloat::array(HANDLING_RANGES_A),
            grip_ramp: PlainFloat::new(2.0),
            grip_ramp_share: PlainFloat::new(0.25),
            settle_fast_step: PlainFloat::new(1.0),
            settle_slow_step: PlainFloat::new(0.01),
            handling_ranges_b: PlainFloat::array(HANDLING_RANGES_B),
            surface_blend: PlainFloat::array(SURFACE_BLEND),
            handling_ranges_c: PlainFloat::array(HANDLING_RANGES_C),
            probe_share: PlainFloat::new(0.5),
            spring_force_scale: PlainFloat::new(2.0),
            torque_ease: PlainFloat::new(0.25),
            reserved_770: PlainDword::array(UNUSED_770),
            drive_cut_steps: PlainDword::new(5),
            inertia: PlainFloat::array(INERTIA),
            downforce_factor: PlainFloat::new(0.0001),
            rev_limits: PlainFloat::array(REV_LIMITS),
            camera_drop: PlainFloat::new(0.7),
            ground_drop: PlainDword::new(0xB333),
            gravity: PlainFloat::new(9.8),
            air_drag: PlainFloat::new(5.0),
            rolling_drag: PlainFloat::new(0.75),
            free_rev_add: PlainFloat::new(5.0),
            free_rev_scale: PlainFloat::new(0.005),
            clear_place_share: PlainFloat::new(0.9),
            start_gear_share: PlainFloat::new(0.99),
            rev_top_scale: PlainFloat::new(1.2),
            gear_ratio_step: PlainFloat::new(0.001),
            wheel_load_scale: PlainFloat::new(0.0025),
            torque_curve: PlainFloat::array(TORQUE_CURVE),
            clear_place_steps_a: PlainDword::array(CLEAR_PLACE_STEPS_A),
            clear_place_steps_b: PlainDword::array(CLEAR_PLACE_STEPS_B),
            camera_frame_limit: PlainFloat::new(0.02),
            camera_frame_scale: PlainFloat::new(0.05),
            contact_steps: PlainDword::new(10),
            reserved_1430: PlainDword::array(UNUSED_1430),
            wheel_load_limit: PlainFloat::new(4.0),
            off_line_distance: PlainFloat::new(32.0),
            split_distance: PlainFloat::new(32.0),
            off_line_word: PlainDword::new(0x60000000),
            pace_scale: PlainFloat::new(1.0),
            listed_events: PlainDword::array(LISTED_EVENTS),
            pace_slot_offsets: PlainDword::array(PACE_SLOT_OFFSETS),
            ..ZeroCells::ZERO
        }
    }
}

impl PhysicsTables {
    /// The dwords of `surface_tables`.
    const TABLES_LEN: usize = 104;

    /// Where ground kind `n`'s table starts in `surface_tables`, as a dword's number. A kind past
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
        assert_eq!(bytes.len(), 5392);
        assert_eq!(hash, 0xb216_3789_ffce_2c43);
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
