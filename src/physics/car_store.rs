//! The work car: the copy of the current car's block that the physics step works on.
//! `drive::load_car` copies the current car's block (the same layout) into it
//! and `drive::store_car` back. Most names are readings; `reserved_*` runs are places no ported
//! code names. Its layout is Rust's own: no code addresses it by byte (the replay file's layout
//! is `drive::replay_state`'s, field by field).

use crate::memory::plain_floats::PlainFloats;
use crate::memory::session::{CellKind, PlainCells};

/// The record, on the cells of kind `K`: plain for the work car and the cars' blocks (`Car`),
/// which only the game's thread writes, and for a view's copy of its car (`ViewCar`).
#[derive(rc2k_derive::Cells)]
pub struct Car<K: CellKind = PlainCells> {
    /// The replay frame's head (camera words and bytes, `drive::replay_cameras`).
    pub frame_head: [K::Dword; 2],
    /// The physics step's clock, +1 per running step.
    pub step_clock: K::Dword,
    /// A random word stirred each step.
    pub noise_word: K::Dword,
    /// Flags; only bit 0x10 read (steering trim negated).
    pub car_flags: K::Dword,
    /// Steps since the run began (0x320 the go point).
    pub run_ticks: K::Dword,
    /// Set once in gear at the start (reading).
    pub start_in_gear: K::Dword,
    /// False-start penalty count.
    pub false_start_count: K::Dword,
    /// Set-up 0 .. 256 (reading).
    pub brake_balance: K::Dword,
    /// The car model (at most 0x21).
    pub car_kind: K::Dword,
    /// Set-up's gear ratios 0 .. 4.
    pub gear_ratio_setting: K::Dword,
    /// Tyres 0 .. 5 (-1 unset).
    pub tyre_kind: K::Dword,
    /// Set-up 0 .. 256.
    pub ride_height: K::Dword,
    /// Set-up 0 .. 256.
    pub suspension_stiffness: K::Dword,
    /// 2 automatic, else manual.
    pub gearbox_kind: K::Dword,
    /// Set-up 0 .. 256.
    pub steering_sensitivity: K::Dword,
    /// Drive layout (1 both axles).
    pub drive_kind: K::Dword,
    /// Only cleared at the start (`drive::clear_start_state`); nothing else uses it.
    pub reserved_44: K::Dword,
    /// Per part (six): 1 once the part's wake count reached 10 and the part's sound object was
    /// woken (`race::wake_part_sounds`). Never cleared but with the block.
    pub part_woken: [K::Dword; 6],
    /// Least wheel grip scaled; scales the ground's bumps.
    pub ground_bump_scale: K::Float,
    /// Eight 0 .. 1 gauges, one per octant of a hit's direction.
    pub impact_octant_marks: [K::Float; 8],
    /// The lights switched on.
    pub lights_on: K::Dword,
    /// Stage time in steps after the start.
    pub stage_time: K::Dword,
    /// Race mode 2's countdown.
    pub time_left: K::Dword,
    /// Splits crossed.
    pub splits_passed: K::Dword,
    /// The checkpoint a reset puts the car back to.
    pub reset_checkpoint: K::Dword,
    /// The car's current checkpoint.
    pub checkpoint: K::Dword,
    /// The last checkpoint whose pace note was called.
    pub last_pace_call: K::Dword,
    /// Slow steps once going (reading).
    pub started_stand_timer: K::Dword,
    /// Slow steps after "go" (reading).
    pub go_stand_timer: K::Dword,
    /// Set on finishing, then counts slow steps.
    pub finish_flag: K::Dword,
    /// Splits whose figure is kept.
    pub splits_recorded: K::Dword,
    /// -1 stood still too long, 2 stopped after the finish.
    pub stop_state: K::Dword,
    /// The 4 x 4 orientation matrix (the crash matrix); the translation row is its last.
    pub crash_matrix: PlainFloats<16>,
    /// World position x, y, z (x 65536).
    pub position: [K::Dword; 3],
    /// One figure per split taken.
    pub split_times: [K::Dword; 16],
    /// Drive speed less forward speed.
    pub drive_slip_speed: K::Float,
    /// Angular velocity x, y, z.
    pub spin: [K::Float; 3],
    /// Length of the spin.
    pub spin_length: K::Float,
    /// World-frame velocity.
    pub velocity: [K::Float; 3],
    /// Velocity in the car's frame (z forward: the car speed).
    pub local_velocity: [K::Float; 3],
    /// Driven wheels' speed.
    pub drive_speed_a: K::Float,
    /// Second driven speed.
    pub drive_speed_b: K::Float,
    /// Mean of the two.
    pub mean_drive_speed: K::Float,
    /// Length of the velocity.
    pub speed: K::Float,
    /// Sideways speed held to a limit.
    pub side_speed_held: K::Float,
    /// Steps before the next gear change.
    pub gear_change_hold: K::Dword,
    /// Drive cut after a shift (reading).
    pub drive_cut_steps: K::Dword,
    /// -1 reverse, 0 neutral, 1 .. 7.
    pub gear: K::Int,
    /// Engine revs.
    pub engine_speed: K::Float,
    /// Engine top revs.
    pub engine_speed_limit: K::Float,
    /// A copy of the car spec's scale (`spec_scale_1b8`) written by
    /// load_car_specs; no reader.
    pub spec_scale_copy: K::Float,
    /// Revs for the sound object.
    pub engine_sound_revs: K::Dword,
    /// Steps stuck.
    pub stuck_count: K::Dword,
    /// Sent and received with the position message (`net::send_position`); nothing else reads or
    /// writes it.
    pub position_message_word: K::Dword,
    /// Limiter cut counter.
    pub rev_cut_count: K::Dword,
    /// Steps since a wall, edge or car contact (0x32 then down); a grip-recovery share.
    pub wall_hit_timer: K::Dword,
    /// The driven wheels' turn angles (reading: for drawing).
    pub driven_wheel_turns: [K::Dword; 2],
    /// The spin's yaw eased toward the grip-force sum (reading).
    pub eased_yaw_spin: K::Float,
    /// Sum of the four grip forces.
    pub grip_force_sum: K::Float,
    /// Steering pull from steering damage.
    pub steer_trim: K::Dword,
    /// Nothing reads or writes it.
    pub reserved_20c: K::Dword,
    /// The four terms: lateral, engine braking, yaw, lean.
    pub grip_forces: [K::Float; 4],
    /// Flags: bit 2 damage notices, bit 8 sound speed scale.
    pub option_bits: K::Dword,
    /// The x, y, z of the probe point that crossed an edge or hit.
    pub contact_point: [K::Float; 3],
    /// Size of the contact.
    pub contact_depth: K::Float,
    /// 1 on any contact this step.
    pub contact_flag: K::Dword,
    /// 1 a gear change happened, -1 refused.
    pub gear_changed: K::Int,
    /// 1 from every gear change until `drive::set_listener` hands it to the effects' splines
    /// (`spline_gear_changed`) and clears it.
    pub gear_change_pending: K::Dword,
    /// The colour the face's light gives the car.
    pub light_colour: K::Dword,
    /// Light index 0..0xFF from the face's corner lights.
    pub light_level: K::Dword,
    /// 1 when the front wheels are off the ground.
    pub front_wheels_off: K::Dword,
    /// 1 when the rear wheels are off the ground.
    pub rear_wheels_off: K::Dword,
    /// 1 when the driven wheels are off the ground.
    pub driven_wheels_off: K::Dword,
    /// 1 when all four wheels are off the ground.
    pub all_wheels_off: K::Dword,
    /// Ground points on hard surfaces among the first four.
    pub hard_ground_count: K::Dword,
    /// 0 no driver, 1 a player's car, 2 a computer car.
    pub driver_kind: K::Dword,
    /// Probe points in use (at most 0x40).
    pub probe_count: K::Dword,
    /// Steps the reset key has been held.
    pub reset_key_count: K::Dword,
    /// Steps stood in neutral (tyre change offered at 300).
    pub stopped_steps: K::Dword,
    /// 0 or -1 while racing, counts otherwise.
    pub tyre_change_state: K::Dword,
    /// Steps far from the checkpoints.
    pub off_line_steps: K::Dword,
    /// Steps turned away from the checkpoint's heading.
    pub turned_steps: K::Dword,
    /// Next time the off-line cue fires.
    pub off_line_cue_time: K::Dword,
    /// Next time the turned-away cue fires.
    pub turned_cue_time: K::Dword,
    /// Share blending the two surface records.
    pub surface_blend: K::Float,
    /// 1 some parts gone, -1 others missing, 0 all present.
    pub parts_state: K::Dword,
    /// The wheels' slip speed (car_sound_state).
    pub slip_speed: K::Float,
    /// Slip speed share for the voices.
    pub sound_slip_share: K::Float,
    /// 1.0, 0 when all wheels are off.
    pub sound_level: K::Float,
    /// The motion's length scaled, for the voices.
    pub sound_speed: K::Float,
    /// Bits 1..8: parts and tyres still on the car.
    pub parts_present: K::Dword,
    /// 1 when a drive-train gauge is at the top.
    pub damage_failed: K::Dword,
    /// Brake fade gauge (reading).
    pub brake_fade: K::Float,
    /// Two damper wear gauges.
    pub damper_wear: [K::Float; 2],
    /// Brake temperature gauges.
    pub brake_heat: [K::Float; 2],
    /// Engine temperature gauge.
    pub engine_heat: K::Float,
    /// Engine damage gauge.
    pub engine_damage: K::Float,
    /// Exhaust damage gauge.
    pub exhaust_damage: K::Float,
    /// Turbo damage gauge.
    pub turbo_damage: K::Float,
    /// Cooling damage gauge.
    pub cooling_damage: K::Float,
    /// Clutch damage gauge.
    pub clutch_damage: K::Float,
    /// Differential damage gauge.
    pub differential_damage: K::Float,
    /// Per-gear wear, index gear + 1 (reverse, neutral, 1 .. 7).
    pub gear_wear: [K::Float; 9],
    /// Gearbox damage gauge.
    pub gearbox_damage: K::Float,
    /// Per-wheel hit gauges FL, FR, RL, RR.
    pub wheel_hit_damage: [K::Float; 4],
    /// General body-hit gauge.
    pub body_hit_damage: K::Float,
    /// Body damage gauge.
    pub body_damage: K::Float,
    /// Electrics damage gauge.
    pub electrics_damage: K::Float,
    /// Lights' failure gauges (FL, FR, RL, RR, spots, brake light).
    pub light_damage: [K::Float; 6],
    /// The lights' damage gauge.
    pub lights_damage: K::Float,
    /// Steering damage gauge.
    pub steering_damage: K::Float,
    /// Per-wheel suspension damage gauges.
    pub suspension_damage: [K::Float; 4],
    /// The suspension's overall gauge.
    pub overall_suspension_damage: K::Float,
    /// Brake wear gauge (axle not settled).
    pub brake_wear_a: K::Float,
    /// The other brake wear gauge.
    pub brake_wear_b: K::Float,
    /// Brake damage gauge (handbrake failure ramp).
    pub brake_damage: K::Float,
    /// The four tyres' wear, FL, FR, RL, RR, 0 ..= 100 (the developers' wear
    /// page's "tyre1" .. "tyre4"); `tyre_wear` is their overall figure.
    pub tyre_wears: [K::Float; 4],
    /// Tyre wear gauge.
    pub tyre_wear: K::Float,
    /// Warning levels 0 .. 4, one per gauge group.
    pub damage_levels: [K::Dword; 35],
    /// Engine health factor on the torque.
    pub engine_power_factor: K::Float,
    /// Braking factors (a, b).
    pub brake_efficiency: [K::Float; 2],
    /// Factors on the two drive-force outputs.
    pub drive_efficiency: [K::Float; 2],
    /// Steering factor from the steering damage.
    pub steering_efficiency: K::Float,
    /// Intermittent failures: electrics, then lights FL, FR, RL, RR, spots, brake.
    pub electrical_failures: [K::Dword; 7],
    /// Set once by chance from the brake damage.
    pub handbrake_failed: K::Dword,
    /// By chance each step: the gearbox does nothing.
    pub gear_change_blocked: K::Dword,
    /// By chance each step: put in neutral.
    pub forced_neutral: K::Dword,
    /// Bit n: gear n - 1 barred.
    pub lost_gear_bits: K::Dword,
    /// Per-wheel ride height %.
    pub wheel_ride_height: [K::Float; 4],
    /// Per-wheel suspension stiffness %.
    pub wheel_stiffness: [K::Float; 4],
    /// Mean of wheel_stiffness.
    pub stiffness_mean: K::Float,
    /// Mean of wheel_ride_height.
    pub ride_height_mean: K::Float,
    /// Side-hit cooldown (never started).
    pub hit_cooldown: K::Dword,
    /// Throttle input.
    pub input_throttle: K::Dword,
    /// Key and button bits.
    pub input_keys: K::Dword,
    /// Steering input.
    pub input_steering: K::Dword,
    /// Brake input.
    pub input_brake: K::Dword,
    /// Key bits newly pressed this step.
    pub keys_pressed: K::Dword,
    /// Queued damage notices (at most 0x20).
    pub notice_count: K::Dword,
    /// Queued notices' string ids.
    pub notice_ids: [K::Dword; 32],
    /// Queued notices' gauge values.
    pub notice_values: [K::Dword; 32],
    /// Queued notices' ranks 1 .. 4.
    pub notice_ranks: [K::Dword; 32],
    /// Highest speed figure so far (reading).
    pub top_speed: K::Float,
    /// Each wheel's spring compression 0 .. 1 (0 in the air).
    pub wheel_squeeze: [K::Float; 4],
    /// The previous step's wheel_squeeze.
    pub wheel_squeeze_last: [K::Float; 4],
    /// X and z in the checkpoint's frame.
    pub checkpoint_offset: [K::Dword; 2],
    /// Mode 2's gaps in checkpoints to the nearest car ahead and behind
    /// (`drive::note_gap_ahead` / `note_gap_behind`, block 0's; u32::MAX: none).
    pub pace_gaps: [K::Dword; 2],
    /// The gaps eased towards those wanted, ahead and behind (the arcade
    /// panel reads them).
    pub pace_gaps_eased: [K::Dword; 2],
    /// Throttle pedal 0 .. 0x100.
    pub throttle: K::Dword,
    /// Brake pedal 0 .. 0x100.
    pub brake: K::Dword,
    /// Handbrake 0 .. 0x100 and over.
    pub handbrake: K::Dword,
    /// Steering for the wheels, +-0x20000000.
    pub steering: K::Dword,
    /// Steering unscaled by speed.
    pub steering_raw: K::Dword,
    /// Steering + trim.
    pub steer_angle: K::Dword,
    /// Raw steering + 4 x trim (reading).
    pub steer_angle_visual: K::Dword,
    /// The place in race mode 2.
    pub mode2_place: K::Dword,
    /// The cars behind in mode 2.
    pub mode2_behind: K::Dword,
    /// Mode 2's two pace cars (car numbers: the one behind the lead,
    /// the one ahead; `drive::start_cars_behind`, block 0's).
    pub pace_cars: [K::Dword; 2],
    /// The start numbers they were put at.
    pub pace_starts: [K::Dword; 2],
    /// 1 once the pace cars are placed.
    pub pace_placed: K::Dword,
    /// The checkpoints between the two pace cars.
    pub pace_apart: K::Dword,
    /// Push apart of the two pace cars (`start_cars_behind`: +-(8 - gap) x scale), added to the
    /// start blend; 0 when they are 8 or more checkpoints apart.
    pub pace_push: K::Float,
    /// Grid-place share.
    pub start_share: K::Float,
    /// Checkpoints past the lead's start, scaled.
    pub start_blend_share: K::Float,
    /// The start blend, held 0 .. 1.
    pub start_blend: K::Float,
    /// The car's camera mode 0 .. 0x33.
    pub camera_view: K::Dword,
    /// The blend's second camera mode.
    pub camera_blend_view: K::Dword,
    /// Mode before the low-camera switch + 1.
    pub camera_view_return: K::Dword,
    /// Clock when the camera switch began.
    pub camera_switch_clock: K::Dword,
    /// View-switch flag: 1 while the camera changes mode.
    pub camera_switch_active: K::Dword,
    /// Steps the switch lasts (10 when begun).
    pub camera_switch_length: K::Dword,
    /// Blend share 0..1 of the switch.
    pub camera_switch_share: K::Float,
    /// Eased roll of the view.
    pub camera_roll: K::Float,
    /// Camera x, y, z (whole numbers).
    pub camera_position: [K::Dword; 3],
    /// Camera heading, whole-number units << 4.
    pub camera_heading: K::Dword,
    /// Third camera turn (reading).
    pub camera_roll_turn: K::Dword,
    /// Camera pitch, units << 4, negated.
    pub camera_pitch: K::Dword,
    /// Three turns (<< 16) applied to the camera's offset.
    pub camera_offset_turns: [K::Dword; 3],
    /// Zoom of the view.
    pub camera_zoom: K::Float,
    /// The first three: the view's focus point kept, the car's position with y + 0x8000
    /// (`race::frame_draw::view`); the other six unused.
    pub kept_focus_point: [K::Dword; 9],
    /// 1 when the camera mode is 2 or 3.
    pub camera_mode_is_2_or_3: K::Dword,
    /// The six parts' wake counts (`race::frame_draw`'s lights,
    /// `drive::share_car_state`).
    pub part_wake_counts: [K::Dword; 6],
    /// Id of the notice shown (0 none).
    pub notice_id: K::Dword,
    /// The notice's counter: the name plates set it to 1 when drawn,
    /// `race::frame_views` counts it down as a view's textures are forgotten, `false_start`
    /// reads and clears it.
    pub notice_counter: K::Dword,
    /// Id of the last notice shown.
    pub notice_last: K::Dword,
    /// 1 while a notice is shown.
    pub notice_shown: K::Dword,
    /// The notice's time, counted down.
    pub notice_time: K::Dword,
    /// Second notice time.
    pub notice_time_b: K::Dword,
    /// The view's place kept with the car (`race::frame_draw`).
    pub kept_view_place: [K::Dword; 3],
    /// Noise sample 1 scaled.
    pub surface_noise_a: K::Dword,
    /// Same sample 1 value.
    pub surface_noise_b: K::Dword,
    /// Noise sample 2 scaled.
    pub surface_noise_second: K::Dword,
    /// The car's vertical velocity less the vertical speed kept from the step before
    /// (`physics::car_physics_step`); `physics::camera_frame` scales it into the camera's shake
    /// (a probable reading).
    pub vertical_speed_change: K::Float,
    /// Per-wheel whole-number value from the wheel's squeeze.
    pub wheel_spring_force: [K::Dword; 4],
    /// Per ground point: surface kind of the first cell (5 wet).
    pub wheel_surface_a: [K::Dword; 5],
    /// Per ground point: surface kind of the second cell.
    pub wheel_surface_b: [K::Dword; 5],
    /// Per point: the surface's `wear_factor` figure blended between the two
    /// surfaces by `wheel_surface_blend`.
    pub point_wear_factor: [K::Float; 5],
    /// Per point: byte 0x18 of the first cell's record.
    pub wheel_cell_a_byte18: [K::Dword; 5],
    /// Same for the second cell.
    pub wheel_cell_b_byte18: [K::Dword; 5],
    /// Per point: byte 0x14 of the first cell's record.
    pub wheel_cell_a_byte14: [K::Dword; 5],
    /// Same for the second cell.
    pub wheel_cell_b_byte14: [K::Dword; 5],
    /// Per point: blend share 0..1 between the two surfaces.
    pub wheel_surface_blend: [K::Float; 5],
    /// Car heading, units << 4.
    pub car_heading: K::Dword,
    /// Car pitch (reading), units << 4.
    pub car_pitch: K::Dword,
    /// Car roll (reading), units << 4.
    pub car_roll: K::Dword,
    /// The inverse of the crash matrix.
    pub car_inverse_matrix: PlainFloats<16>,
    /// The split's figures (figure, gaps, names, two more).
    pub split_gaps: [K::Dword; 7],
    /// Crash contact point x.
    pub contact_point_x: K::Dword,
    /// Ground height under the contact point.
    pub contact_point_ground: K::Dword,
    /// Contact point z.
    pub contact_point_z: K::Dword,
    /// 1 when the body touches.
    pub contact_found: K::Dword,
    /// The contact's depth or force (reading).
    pub contact_value: K::Dword,
    /// Seven parts' damage figures.
    pub part_damage_limits: [K::Dword; 7],
    /// The block's copy of the physics tables' per-point figures (the five points' loads, grips and
    /// the like); `drive::share_car_state` fills them, `race::car_starts` clears them (a probable
    /// reading).
    pub saved_point_figures: [K::Dword; 20],
    /// The start position's table entry (unknown).
    pub start_slot_value: K::Dword,
    /// The steering drawn (`race::car_parts`).
    pub shown_steering: K::Dword,
    /// Speed scaled, whole number (engine sound).
    pub engine_sound_speed_int: K::Dword,
    /// The marks switched on (`race::car_parts`).
    pub marks_on: K::Dword,
    /// The course map picture's corner x, y (16-pixel steps;
    /// `race::map_scroll`).
    pub map_corner: [K::Dword; 2],
    /// 1 when the map picture was redrawn this step.
    pub map_redrawn: K::Dword,
    /// The car on the course map x, y (16.16).
    pub map_place: [K::Dword; 2],
    /// The car's place in the map picture x, y.
    pub map_picture_place: [K::Dword; 2],
    /// X of the 0x40 ground probe points.
    pub probe_point_x: [K::Float; 64],
    /// Y of the probe points.
    pub probe_point_y: [K::Float; 64],
    /// Z of the probe points.
    pub probe_point_z: [K::Float; 64],
    /// Face found last under each probe point.
    pub probe_point_face: [K::Dword; 64],
    /// Last ground faces under the camera and its sides (five saved by
    /// `physics::camera_avoid_ground`).
    pub camera_ground_faces: [K::Dword; 5],
    /// Unused: the object groups drawn with the car, which nothing reads.
    pub group_mask: K::Dword,
    /// Speed scaled (engine sound).
    pub engine_speed_scaled: K::Float,
    /// The step clock as of the last replay step (`drive::replay_step`).
    pub previous_step_clock: K::Dword,
    /// World x of the four wheels.
    pub wheel_ground_x: [K::Dword; 4],
    /// Ground height under the four wheels.
    pub wheel_ground_height: [K::Dword; 4],
    /// World z of the four wheels.
    pub wheel_ground_z: [K::Dword; 4],
    /// The front and rear axles' skid grip (`physics::drive_wheels_step`;
    /// `race::lay_skidmarks` lays marks from `ShadowSkids.skid_grip` up).
    pub skid_grip: [K::Float; 2],
    /// Each wheel's last skid point (-1 none; `race::lay_skidmarks`).
    pub skid_slots: [K::Dword; 4],
    /// Timed-event queue times.
    pub car_event_times: [K::Dword; 8],
    /// Event codes of the queue.
    pub car_event_codes: [K::Dword; 8],
    /// Queued events (max 7).
    pub car_event_count: K::Dword,
    /// The pace-note symbols' scroll target (`rc2k_core::physics::car_events`: 0x20 less per queued
    /// entry).
    pub car_event_limit: K::Dword,
    /// The symbols' scroll position (y offset from 400 on the HUD, `race::panel_marks`), eased 4 a
    /// step to the target.
    pub car_event_position: K::Dword,
    /// The car's place on the panel (`drive::two_car_order` writes it, `race::frame_draw::panel`
    /// reads it).
    pub two_car_place: K::Dword,
    /// Set to 1 by `race::drop_absent_player` for the player who is out of a two-player race (the
    /// first whose `stop_state` is set; also clears `driver_kind`): the view is then not drawn
    /// (`draw_view_scene` and `draw_view_overlays` only tint it) and its notice strip stays hidden
    /// (`hud::strips`). Zeroed by the block reset.
    pub view_covered: K::Dword,
    /// Set to 10 by the resets; no reader.
    pub reset_marker: K::Dword,
    /// Steps while ease_car_value jumps to the target.
    pub value_jump_steps: K::Dword,
    /// Steps while car_lighting holds the light change.
    pub light_hold_steps: K::Dword,
    /// The target `ease_car_value` follows.
    pub eased_target: K::Dword,
    /// The value eased towards eased_target << 8.
    pub eased_value: K::Dword,
    /// Where a position received from the network is to be glided to
    /// (x, y, z; `net::apply_message`, stepped towards by `net::follow_remote`).
    pub glide_target: [K::Dword; 3],
    /// The steps of the glide left (set to 32 by `net::apply_message`,
    /// one less each step in `net::follow_remote`).
    pub glide_steps_left: K::Dword,
    /// The throttle input word kept for the network and replays (first of the
    /// four `drive::car_input_words` copies; the car's `input_throttle`).
    pub kept_throttle_word: K::Dword,
    /// The key bits word (the car's `input_keys`).
    pub kept_keys_word: K::Dword,
    /// The steering word (the car's `input_steering`).
    pub kept_steering_word: K::Dword,
    /// The brake word (the car's `input_brake`).
    pub kept_brake_word: K::Dword,
}

/// A view's copy of its car (`race::view_scratch::ViewScratch::car`): the same record, which
/// the view's drawing part writes and no other thread reads.
pub type ViewCar = Car;

impl Car {
    /// The piece at the start of a race: zero but for the set-up (ride height 0x80,
    /// suspension stiffness 0x40) and the parts present (all bits).
    pub const fn new() -> Self {
        use crate::memory::session::PlainDword;
        let mut car: Self = crate::memory::cells::zeroed_cells();
        car.ride_height = PlainDword::new(0x80);
        car.suspension_stiffness = PlainDword::new(0x40);
        car.parts_present = PlainDword::new(u32::MAX);
        car
    }
}

/// Puts `values` into `fields`, the fields' dwords laid end to end in the order given, the first
/// value into dword `skip`. A value past the last field is refused (nothing is written for it).
pub(super) fn put_dwords(
    fields: &[&dyn crate::memory::session::Run],
    skip: usize,
    values: impl IntoIterator<Item = u32>,
) -> Result<(), crate::errors::BadInput> {
    let mut values = values.into_iter().peekable();
    let mut skip = skip;
    for field in fields {
        for i in skip.min(field.count())..field.count() {
            let Some(v) = values.next() else {
                return Ok(());
            };
            field.set_bits(i, v);
        }
        skip = skip.saturating_sub(field.count());
    }
    match values.peek() {
        None => Ok(()),
        Some(_) => Err(crate::errors::BadInput(
            "a dword past the fields of its record",
        )),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn car_is_aligned() {
        let shared_store = &crate::test_support::own::race().shared;

        // other tests write the work car while they run
        let _lock = crate::test_support::test_lock();
        assert_eq!(core::ptr::from_ref(shared_store.car()).addr() % 4, 0);
    }
}
