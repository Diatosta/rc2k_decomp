//! The work car: the copy of the current car's block that the physics step works on.
//! `drive::load_car` copies the current car's block (the same layout) into it
//! and `drive::store_car` back. Most names are readings; `reserved_*` runs are places no ported
//! code names. Its layout is Rust's own: no code addresses it by byte (the replay file's layout
//! is `drive::replay_state`'s, field by field).

/// The record: the work car, the cars' blocks and a view's copy of its car (`ViewCar`).
#[derive(Clone, rc2k_derive::PlainRecord)]
pub struct Car {
    /// The replay frame's head (camera words and bytes, `drive::replay_cameras`).
    pub frame_head: [u32; 2],
    /// The physics step's clock, +1 per running step.
    pub step_clock: u32,
    /// A random word stirred each step.
    pub noise_word: u32,
    /// Flags; only bit 0x10 read (steering trim negated).
    pub car_flags: u32,
    /// Steps since the run began (0x320 the go point).
    pub run_ticks: u32,
    /// Set once in gear at the start (reading).
    pub start_in_gear: u32,
    /// False-start penalty count.
    pub false_start_count: u32,
    /// Set-up 0 .. 256 (reading).
    pub brake_balance: u32,
    /// The car model (at most 0x21).
    pub car_kind: u32,
    /// Set-up's gear ratios 0 .. 4.
    pub gear_ratio_setting: u32,
    /// Tyres 0 .. 5 (-1 unset).
    pub tyre_kind: u32,
    /// Set-up 0 .. 256.
    pub ride_height: u32,
    /// Set-up 0 .. 256.
    pub suspension_stiffness: u32,
    /// 2 automatic, else manual.
    pub gearbox_kind: u32,
    /// Set-up 0 .. 256.
    pub steering_sensitivity: u32,
    /// Drive layout (1 both axles).
    pub drive_kind: u32,
    /// Only cleared at the start (`drive::clear_start_state`); nothing else uses it.
    pub reserved_44: u32,
    /// Per part (six): 1 once the part's wake count reached 10 and the part's sound object was
    /// woken (`race::wake_part_sounds`). Never cleared but with the block.
    pub part_woken: [u32; 6],
    /// Least wheel grip scaled; scales the ground's bumps.
    pub ground_bump_scale: f32,
    /// Eight 0 .. 1 gauges, one per octant of a hit's direction.
    pub impact_octant_marks: [f32; 8],
    /// The lights switched on.
    pub lights_on: u32,
    /// Stage time in steps after the start.
    pub stage_time: u32,
    /// Race mode 2's countdown.
    pub time_left: u32,
    /// Splits crossed.
    pub splits_passed: u32,
    /// The checkpoint a reset puts the car back to.
    pub reset_checkpoint: u32,
    /// The car's current checkpoint.
    pub checkpoint: u32,
    /// The last checkpoint whose pace note was called.
    pub last_pace_call: u32,
    /// Slow steps once going (reading).
    pub started_stand_timer: u32,
    /// Slow steps after "go" (reading).
    pub go_stand_timer: u32,
    /// Set on finishing, then counts slow steps.
    pub finish_flag: u32,
    /// Splits whose figure is kept.
    pub splits_recorded: u32,
    /// -1 stood still too long, 2 stopped after the finish.
    pub stop_state: u32,
    /// The 4 x 4 orientation matrix (the crash matrix); the translation row is its last.
    pub crash_matrix: [f32; 16],
    /// World position x, y, z (x 65536).
    pub position: [f32; 3],
    /// One figure per split taken.
    pub split_times: [u32; 16],
    /// Drive speed less forward speed.
    pub drive_slip_speed: f32,
    /// Angular velocity x, y, z.
    pub spin: [f32; 3],
    /// Length of the spin.
    pub spin_length: f32,
    /// World-frame velocity.
    pub velocity: [f32; 3],
    /// Velocity in the car's frame (z forward: the car speed).
    pub local_velocity: [f32; 3],
    /// Driven wheels' speed.
    pub drive_speed_a: f32,
    /// Second driven speed.
    pub drive_speed_b: f32,
    /// Mean of the two.
    pub mean_drive_speed: f32,
    /// Length of the velocity.
    pub speed: f32,
    /// Sideways speed held to a limit.
    pub side_speed_held: f32,
    /// Steps before the next gear change.
    pub gear_change_hold: u32,
    /// Drive cut after a shift (reading).
    pub drive_cut_steps: u32,
    /// -1 reverse, 0 neutral, 1 .. 7.
    pub gear: i32,
    /// Engine revs.
    pub engine_speed: f32,
    /// Engine top revs.
    pub engine_speed_limit: f32,
    /// A copy of the car spec's scale (`spec_scale_1b8`) written by
    /// load_car_specs; no reader.
    pub spec_scale_copy: f32,
    /// Revs for the sound object.
    pub engine_sound_revs: u32,
    /// Steps stuck.
    pub stuck_count: u32,
    /// Sent and received with the position message (`net::send_position`); nothing else reads or
    /// writes it.
    pub position_message_word: u32,
    /// Limiter cut counter.
    pub rev_cut_count: u32,
    /// Steps since a wall, edge or car contact (0x32 then down); a grip-recovery share.
    pub wall_hit_timer: u32,
    /// The driven wheels' turn angles (reading: for drawing).
    pub driven_wheel_turns: [u32; 2],
    /// The spin's yaw eased toward the grip-force sum (reading).
    pub eased_yaw_spin: f32,
    /// Sum of the four grip forces.
    pub grip_force_sum: f32,
    /// Steering pull from steering damage.
    pub steer_trim: u32,
    /// Nothing reads or writes it.
    pub reserved_20c: u32,
    /// The four terms: lateral, engine braking, yaw, lean.
    pub grip_forces: [f32; 4],
    /// Flags: bit 2 damage notices, bit 8 sound speed scale.
    pub option_bits: u32,
    /// The x, y, z of the probe point that crossed an edge or hit.
    pub contact_point: [f32; 3],
    /// Size of the contact.
    pub contact_depth: f32,
    /// 1 on any contact this step.
    pub contact_flag: u32,
    /// 1 a gear change happened, -1 refused.
    pub gear_changed: i32,
    /// 1 from every gear change until `drive::set_listener` hands it to the effects' splines
    /// (`spline_gear_changed`) and clears it.
    pub gear_change_pending: u32,
    /// The colour the face's light gives the car.
    pub light_colour: u32,
    /// Light index 0..0xFF from the face's corner lights.
    pub light_level: u32,
    /// 1 when the front wheels are off the ground.
    pub front_wheels_off: u32,
    /// 1 when the rear wheels are off the ground.
    pub rear_wheels_off: u32,
    /// 1 when the driven wheels are off the ground.
    pub driven_wheels_off: u32,
    /// 1 when all four wheels are off the ground.
    pub all_wheels_off: u32,
    /// Ground points on hard surfaces among the first four.
    pub hard_ground_count: u32,
    /// 0 no driver, 1 a player's car, 2 a computer car.
    pub driver_kind: u32,
    /// Probe points in use (at most 0x40).
    pub probe_count: u32,
    /// Steps the reset key has been held.
    pub reset_key_count: u32,
    /// Steps stood in neutral (tyre change offered at 300).
    pub stopped_steps: u32,
    /// 0 or -1 while racing, counts otherwise.
    pub tyre_change_state: u32,
    /// Steps far from the checkpoints.
    pub off_line_steps: u32,
    /// Steps turned away from the checkpoint's heading.
    pub turned_steps: u32,
    /// Next time the off-line cue fires.
    pub off_line_cue_time: u32,
    /// Next time the turned-away cue fires.
    pub turned_cue_time: u32,
    /// Share blending the two surface records.
    pub surface_blend: f32,
    /// 1 some parts gone, -1 others missing, 0 all present.
    pub parts_state: u32,
    /// The wheels' slip speed (car_sound_state).
    pub slip_speed: f32,
    /// Slip speed share for the voices.
    pub sound_slip_share: f32,
    /// 1.0, 0 when all wheels are off.
    pub sound_level: f32,
    /// The motion's length scaled, for the voices.
    pub sound_speed: f32,
    /// Bits 1..8: parts and tyres still on the car.
    pub parts_present: u32,
    /// 1 when a drive-train gauge is at the top.
    pub damage_failed: u32,
    /// Brake fade gauge (reading).
    pub brake_fade: f32,
    /// Two damper wear gauges.
    pub damper_wear: [f32; 2],
    /// Brake temperature gauges.
    pub brake_heat: [f32; 2],
    /// Engine temperature gauge.
    pub engine_heat: f32,
    /// Engine damage gauge.
    pub engine_damage: f32,
    /// Exhaust damage gauge.
    pub exhaust_damage: f32,
    /// Turbo damage gauge.
    pub turbo_damage: f32,
    /// Cooling damage gauge.
    pub cooling_damage: f32,
    /// Clutch damage gauge.
    pub clutch_damage: f32,
    /// Differential damage gauge.
    pub differential_damage: f32,
    /// Per-gear wear, index gear + 1 (reverse, neutral, 1 .. 7).
    pub gear_wear: [f32; 9],
    /// Gearbox damage gauge.
    pub gearbox_damage: f32,
    /// Per-wheel hit gauges FL, FR, RL, RR.
    pub wheel_hit_damage: [f32; 4],
    /// General body-hit gauge.
    pub body_hit_damage: f32,
    /// Body damage gauge.
    pub body_damage: f32,
    /// Electrics damage gauge.
    pub electrics_damage: f32,
    /// Lights' failure gauges (FL, FR, RL, RR, spots, brake light).
    pub light_damage: [f32; 6],
    /// The lights' damage gauge.
    pub lights_damage: f32,
    /// Steering damage gauge.
    pub steering_damage: f32,
    /// Per-wheel suspension damage gauges.
    pub suspension_damage: [f32; 4],
    /// The suspension's overall gauge.
    pub overall_suspension_damage: f32,
    /// Brake wear gauge (axle not settled).
    pub brake_wear_a: f32,
    /// The other brake wear gauge.
    pub brake_wear_b: f32,
    /// Brake damage gauge (handbrake failure ramp).
    pub brake_damage: f32,
    /// The four tyres' wear, FL, FR, RL, RR, 0 ..= 100 (the developers' wear
    /// page's "tyre1" .. "tyre4"); `tyre_wear` is their overall figure.
    pub tyre_wears: [f32; 4],
    /// Tyre wear gauge.
    pub tyre_wear: f32,
    /// Warning levels 0 .. 4, one per gauge group.
    pub damage_levels: [u32; 35],
    /// Engine health factor on the torque.
    pub engine_power_factor: f32,
    /// Braking factors (a, b).
    pub brake_efficiency: [f32; 2],
    /// Factors on the two drive-force outputs.
    pub drive_efficiency: [f32; 2],
    /// Steering factor from the steering damage.
    pub steering_efficiency: f32,
    /// Intermittent failures: electrics, then lights FL, FR, RL, RR, spots, brake.
    pub electrical_failures: [u32; 7],
    /// Set once by chance from the brake damage.
    pub handbrake_failed: u32,
    /// By chance each step: the gearbox does nothing.
    pub gear_change_blocked: u32,
    /// By chance each step: put in neutral.
    pub forced_neutral: u32,
    /// Bit n: gear n - 1 barred.
    pub lost_gear_bits: u32,
    /// Per-wheel ride height %.
    pub wheel_ride_height: [f32; 4],
    /// Per-wheel suspension stiffness %.
    pub wheel_stiffness: [f32; 4],
    /// Mean of wheel_stiffness.
    pub stiffness_mean: f32,
    /// Mean of wheel_ride_height.
    pub ride_height_mean: f32,
    /// Side-hit cooldown (never started).
    pub hit_cooldown: u32,
    /// Throttle input.
    pub input_throttle: u32,
    /// Key and button bits.
    pub input_keys: u32,
    /// Steering input.
    pub input_steering: u32,
    /// Brake input.
    pub input_brake: u32,
    /// Key bits newly pressed this step.
    pub keys_pressed: u32,
    /// Queued damage notices (at most 0x20).
    pub notice_count: u32,
    /// Queued notices' string ids.
    pub notice_ids: [u32; 32],
    /// Queued notices' gauge values.
    pub notice_values: [u32; 32],
    /// Queued notices' ranks 1 .. 4.
    pub notice_ranks: [u32; 32],
    /// Highest speed figure so far (reading).
    pub top_speed: f32,
    /// Each wheel's spring compression 0 .. 1 (0 in the air).
    pub wheel_squeeze: [f32; 4],
    /// The previous step's wheel_squeeze.
    pub wheel_squeeze_last: [f32; 4],
    /// X and z in the checkpoint's frame.
    pub checkpoint_offset: [u32; 2],
    /// Mode 2's gaps in checkpoints to the nearest car ahead and behind
    /// (`drive::note_gap_ahead` / `note_gap_behind`, block 0's; u32::MAX: none).
    pub pace_gaps: [u32; 2],
    /// The gaps eased towards those wanted, ahead and behind (the arcade
    /// panel reads them).
    pub pace_gaps_eased: [u32; 2],
    /// Throttle pedal 0 .. 0x100.
    pub throttle: u32,
    /// Brake pedal 0 .. 0x100.
    pub brake: u32,
    /// Handbrake 0 .. 0x100 and over.
    pub handbrake: u32,
    /// Steering for the wheels, +-0x20000000.
    pub steering: u32,
    /// Steering unscaled by speed.
    pub steering_raw: u32,
    /// Steering + trim.
    pub steer_angle: u32,
    /// Raw steering + 4 x trim (reading).
    pub steer_angle_visual: u32,
    /// The place in race mode 2.
    pub mode2_place: u32,
    /// The cars behind in mode 2.
    pub mode2_behind: u32,
    /// Mode 2's two pace cars (car numbers: the one behind the lead,
    /// the one ahead; `drive::start_cars_behind`, block 0's).
    pub pace_cars: [u32; 2],
    /// The start numbers they were put at.
    pub pace_starts: [u32; 2],
    /// 1 once the pace cars are placed.
    pub pace_placed: u32,
    /// The checkpoints between the two pace cars.
    pub pace_apart: u32,
    /// Push apart of the two pace cars (`start_cars_behind`: +-(8 - gap) x scale), added to the
    /// start blend; 0 when they are 8 or more checkpoints apart.
    pub pace_push: f32,
    /// Grid-place share.
    pub start_share: f32,
    /// Checkpoints past the lead's start, scaled.
    pub start_blend_share: f32,
    /// The start blend, held 0 .. 1.
    pub start_blend: f32,
    /// The car's camera mode 0 .. 0x33.
    pub camera_view: u32,
    /// The blend's second camera mode.
    pub camera_blend_view: u32,
    /// Mode before the low-camera switch + 1.
    pub camera_view_return: u32,
    /// Clock when the camera switch began.
    pub camera_switch_clock: u32,
    /// View-switch flag: 1 while the camera changes mode.
    pub camera_switch_active: u32,
    /// Steps the switch lasts (10 when begun).
    pub camera_switch_length: u32,
    /// Blend share 0..1 of the switch.
    pub camera_switch_share: f32,
    /// Eased roll of the view.
    pub camera_roll: f32,
    /// Camera x, y, z (whole numbers).
    pub camera_position: [f32; 3],
    /// Camera heading, whole-number units << 4.
    pub camera_heading: u32,
    /// Third camera turn (reading).
    pub camera_roll_turn: u32,
    /// Camera pitch, units << 4, negated.
    pub camera_pitch: u32,
    /// Three turns (<< 16) applied to the camera's offset.
    pub camera_offset_turns: [f32; 3],
    /// Zoom of the view.
    pub camera_zoom: f32,
    /// The first three: the view's focus point kept, the car's position with y + 0x8000
    /// (`race::frame_draw::view`); the other six unused.
    pub kept_focus_point: [u32; 9],
    /// 1 when the camera mode is 2 or 3.
    pub camera_mode_is_2_or_3: u32,
    /// The six parts' wake counts (`race::frame_draw`'s lights,
    /// `drive::share_car_state`).
    pub part_wake_counts: [u32; 6],
    /// Id of the notice shown (0 none).
    pub notice_id: u32,
    /// The notice's counter: the name plates set it to 1 when drawn,
    /// `race::frame_views` counts it down as a view's textures are forgotten, `false_start`
    /// reads and clears it.
    pub notice_counter: u32,
    /// Id of the last notice shown.
    pub notice_last: u32,
    /// 1 while a notice is shown.
    pub notice_shown: u32,
    /// The notice's time, counted down.
    pub notice_time: u32,
    /// Second notice time.
    pub notice_time_b: u32,
    /// The view's place kept with the car (`race::frame_draw`).
    pub kept_view_place: [f32; 3],
    /// Noise sample 1 scaled.
    pub surface_noise_a: u32,
    /// Same sample 1 value.
    pub surface_noise_b: u32,
    /// Noise sample 2 scaled.
    pub surface_noise_second: u32,
    /// The car's vertical velocity less the vertical speed kept from the step before
    /// (`physics::car_physics_step`); `physics::camera_frame` scales it into the camera's shake
    /// (a probable reading).
    pub vertical_speed_change: f32,
    /// Per-wheel whole-number value from the wheel's squeeze.
    pub wheel_spring_force: [u32; 4],
    /// Per ground point: surface kind of the first cell (5 wet).
    pub wheel_surface_a: [u32; 5],
    /// Per ground point: surface kind of the second cell.
    pub wheel_surface_b: [u32; 5],
    /// Per point: the surface's `wear_factor` figure blended between the two
    /// surfaces by `wheel_surface_blend`.
    pub point_wear_factor: [f32; 5],
    /// Per point: byte 0x18 of the first cell's record.
    pub wheel_cell_a_byte18: [u32; 5],
    /// Same for the second cell.
    pub wheel_cell_b_byte18: [u32; 5],
    /// Per point: byte 0x14 of the first cell's record.
    pub wheel_cell_a_byte14: [u32; 5],
    /// Same for the second cell.
    pub wheel_cell_b_byte14: [u32; 5],
    /// Per point: blend share 0..1 between the two surfaces.
    pub wheel_surface_blend: [f32; 5],
    /// Car heading, units << 4.
    pub car_heading: u32,
    /// Car pitch (reading), units << 4.
    pub car_pitch: u32,
    /// Car roll (reading), units << 4.
    pub car_roll: u32,
    /// The inverse of the crash matrix.
    pub car_inverse_matrix: [f32; 16],
    /// The split's figures (figure, gaps, names, two more).
    pub split_gaps: [u32; 7],
    /// Crash contact point x.
    pub contact_point_x: u32,
    /// Ground height under the contact point.
    pub contact_point_ground: u32,
    /// Contact point z.
    pub contact_point_z: u32,
    /// 1 when the body touches.
    pub contact_found: u32,
    /// The contact's depth or force (reading).
    pub contact_value: u32,
    /// Seven parts' damage figures.
    pub part_damage_limits: [u32; 7],
    /// The block's copy of the physics tables' per-point figures (the five points' loads, grips and
    /// the like); `drive::share_car_state` fills them, `race::car_starts` clears them (a probable
    /// reading).
    pub saved_point_figures: [u32; 20],
    /// The start position's table entry (unknown).
    pub start_slot_value: u32,
    /// The steering drawn (`race::car_parts`).
    pub shown_steering: u32,
    /// Speed scaled, whole number (engine sound).
    pub engine_sound_speed_int: u32,
    /// The marks switched on (`race::car_parts`).
    pub marks_on: u32,
    /// The course map picture's corner x, y (16-pixel steps;
    /// `race::map_scroll`).
    pub map_corner: [u32; 2],
    /// 1 when the map picture was redrawn this step.
    pub map_redrawn: u32,
    /// The car on the course map x, y (16.16).
    pub map_place: [u32; 2],
    /// The car's place in the map picture x, y.
    pub map_picture_place: [u32; 2],
    /// X of the 0x40 ground probe points.
    pub probe_point_x: [f32; 64],
    /// Y of the probe points.
    pub probe_point_y: [f32; 64],
    /// Z of the probe points.
    pub probe_point_z: [f32; 64],
    /// Face found last under each probe point.
    pub probe_point_face: [u32; 64],
    /// Last ground faces under the camera and its sides (five saved by
    /// `physics::camera_avoid_ground`).
    pub camera_ground_faces: [u32; 5],
    /// Unused: the object groups drawn with the car, which nothing reads.
    pub group_mask: u32,
    /// Speed scaled (engine sound).
    pub engine_speed_scaled: f32,
    /// The step clock as of the last replay step (`drive::replay_step`).
    pub previous_step_clock: u32,
    /// World x of the four wheels.
    pub wheel_ground_x: [u32; 4],
    /// Ground height under the four wheels.
    pub wheel_ground_height: [u32; 4],
    /// World z of the four wheels.
    pub wheel_ground_z: [u32; 4],
    /// The front and rear axles' skid grip (`physics::drive_wheels_step`;
    /// `race::lay_skidmarks` lays marks from `ShadowSkids.skid_grip` up).
    pub skid_grip: [f32; 2],
    /// Each wheel's last skid point (-1 none; `race::lay_skidmarks`).
    pub skid_slots: [u32; 4],
    /// Timed-event queue times.
    pub car_event_times: [u32; 8],
    /// Event codes of the queue.
    pub car_event_codes: [u32; 8],
    /// Queued events (max 7).
    pub car_event_count: u32,
    /// The pace-note symbols' scroll target (`rc2k_core::physics::car_events`: 0x20 less per queued
    /// entry).
    pub car_event_limit: u32,
    /// The symbols' scroll position (y offset from 400 on the HUD, `race::panel_marks`), eased 4 a
    /// step to the target.
    pub car_event_position: u32,
    /// The car's place on the panel (`drive::two_car_order` writes it, `race::frame_draw::panel`
    /// reads it).
    pub two_car_place: u32,
    /// Set to 1 by `race::drop_absent_player` for the player who is out of a two-player race (the
    /// first whose `stop_state` is set; also clears `driver_kind`): the view is then not drawn
    /// (`draw_view_scene` and `draw_view_overlays` only tint it) and its notice strip stays hidden
    /// (`hud::strips`). Zeroed by the block reset.
    pub view_covered: u32,
    /// Set to 10 by the resets; no reader.
    pub reset_marker: u32,
    /// Steps while ease_car_value jumps to the target.
    pub value_jump_steps: u32,
    /// Steps while car_lighting holds the light change.
    pub light_hold_steps: u32,
    /// The target `ease_car_value` follows.
    pub eased_target: u32,
    /// The value eased towards eased_target << 8.
    pub eased_value: u32,
    /// Where a position received from the network is to be glided to
    /// (x, y, z; `net::apply_message`, stepped towards by `net::follow_remote`).
    pub glide_target: [f32; 3],
    /// The steps of the glide left (set to 32 by `net::apply_message`,
    /// one less each step in `net::follow_remote`).
    pub glide_steps_left: u32,
    /// The throttle input word kept for the network and replays (first of the
    /// four `drive::car_input_words` copies; the car's `input_throttle`).
    pub kept_throttle_word: u32,
    /// The key bits word (the car's `input_keys`).
    pub kept_keys_word: u32,
    /// The steering word (the car's `input_steering`).
    pub kept_steering_word: u32,
    /// The brake word (the car's `input_brake`).
    pub kept_brake_word: u32,
}

/// A view's copy of its car (`race::view_scratch::ViewScratch::car`): the same record, which
/// the view's drawing part writes and no other thread reads.
pub type ViewCar = Car;

impl Car {
    /// The piece at the start of a race: zero but for the set-up (ride height 0x80,
    /// suspension stiffness 0x40) and the parts present (all bits).
    pub const fn new() -> Self {
        Car {
            frame_head: [0; 2],
            step_clock: 0,
            noise_word: 0,
            car_flags: 0,
            run_ticks: 0,
            start_in_gear: 0,
            false_start_count: 0,
            brake_balance: 0,
            car_kind: 0,
            gear_ratio_setting: 0,
            tyre_kind: 0,
            ride_height: 0x80,
            suspension_stiffness: 0x40,
            gearbox_kind: 0,
            steering_sensitivity: 0,
            drive_kind: 0,
            reserved_44: 0,
            part_woken: [0; 6],
            ground_bump_scale: 0.0,
            impact_octant_marks: [0.0; 8],
            lights_on: 0,
            stage_time: 0,
            time_left: 0,
            splits_passed: 0,
            reset_checkpoint: 0,
            checkpoint: 0,
            last_pace_call: 0,
            started_stand_timer: 0,
            go_stand_timer: 0,
            finish_flag: 0,
            splits_recorded: 0,
            stop_state: 0,
            crash_matrix: [0.0; 16],
            position: [0.0; 3],
            split_times: [0; 16],
            drive_slip_speed: 0.0,
            spin: [0.0; 3],
            spin_length: 0.0,
            velocity: [0.0; 3],
            local_velocity: [0.0; 3],
            drive_speed_a: 0.0,
            drive_speed_b: 0.0,
            mean_drive_speed: 0.0,
            speed: 0.0,
            side_speed_held: 0.0,
            gear_change_hold: 0,
            drive_cut_steps: 0,
            gear: 0,
            engine_speed: 0.0,
            engine_speed_limit: 0.0,
            spec_scale_copy: 0.0,
            engine_sound_revs: 0,
            stuck_count: 0,
            position_message_word: 0,
            rev_cut_count: 0,
            wall_hit_timer: 0,
            driven_wheel_turns: [0; 2],
            eased_yaw_spin: 0.0,
            grip_force_sum: 0.0,
            steer_trim: 0,
            reserved_20c: 0,
            grip_forces: [0.0; 4],
            option_bits: 0,
            contact_point: [0.0; 3],
            contact_depth: 0.0,
            contact_flag: 0,
            gear_changed: 0,
            gear_change_pending: 0,
            light_colour: 0,
            light_level: 0,
            front_wheels_off: 0,
            rear_wheels_off: 0,
            driven_wheels_off: 0,
            all_wheels_off: 0,
            hard_ground_count: 0,
            driver_kind: 0,
            probe_count: 0,
            reset_key_count: 0,
            stopped_steps: 0,
            tyre_change_state: 0,
            off_line_steps: 0,
            turned_steps: 0,
            off_line_cue_time: 0,
            turned_cue_time: 0,
            surface_blend: 0.0,
            parts_state: 0,
            slip_speed: 0.0,
            sound_slip_share: 0.0,
            sound_level: 0.0,
            sound_speed: 0.0,
            parts_present: u32::MAX,
            damage_failed: 0,
            brake_fade: 0.0,
            damper_wear: [0.0; 2],
            brake_heat: [0.0; 2],
            engine_heat: 0.0,
            engine_damage: 0.0,
            exhaust_damage: 0.0,
            turbo_damage: 0.0,
            cooling_damage: 0.0,
            clutch_damage: 0.0,
            differential_damage: 0.0,
            gear_wear: [0.0; 9],
            gearbox_damage: 0.0,
            wheel_hit_damage: [0.0; 4],
            body_hit_damage: 0.0,
            body_damage: 0.0,
            electrics_damage: 0.0,
            light_damage: [0.0; 6],
            lights_damage: 0.0,
            steering_damage: 0.0,
            suspension_damage: [0.0; 4],
            overall_suspension_damage: 0.0,
            brake_wear_a: 0.0,
            brake_wear_b: 0.0,
            brake_damage: 0.0,
            tyre_wears: [0.0; 4],
            tyre_wear: 0.0,
            damage_levels: [0; 35],
            engine_power_factor: 0.0,
            brake_efficiency: [0.0; 2],
            drive_efficiency: [0.0; 2],
            steering_efficiency: 0.0,
            electrical_failures: [0; 7],
            handbrake_failed: 0,
            gear_change_blocked: 0,
            forced_neutral: 0,
            lost_gear_bits: 0,
            wheel_ride_height: [0.0; 4],
            wheel_stiffness: [0.0; 4],
            stiffness_mean: 0.0,
            ride_height_mean: 0.0,
            hit_cooldown: 0,
            input_throttle: 0,
            input_keys: 0,
            input_steering: 0,
            input_brake: 0,
            keys_pressed: 0,
            notice_count: 0,
            notice_ids: [0; 32],
            notice_values: [0; 32],
            notice_ranks: [0; 32],
            top_speed: 0.0,
            wheel_squeeze: [0.0; 4],
            wheel_squeeze_last: [0.0; 4],
            checkpoint_offset: [0; 2],
            pace_gaps: [0; 2],
            pace_gaps_eased: [0; 2],
            throttle: 0,
            brake: 0,
            handbrake: 0,
            steering: 0,
            steering_raw: 0,
            steer_angle: 0,
            steer_angle_visual: 0,
            mode2_place: 0,
            mode2_behind: 0,
            pace_cars: [0; 2],
            pace_starts: [0; 2],
            pace_placed: 0,
            pace_apart: 0,
            pace_push: 0.0,
            start_share: 0.0,
            start_blend_share: 0.0,
            start_blend: 0.0,
            camera_view: 0,
            camera_blend_view: 0,
            camera_view_return: 0,
            camera_switch_clock: 0,
            camera_switch_active: 0,
            camera_switch_length: 0,
            camera_switch_share: 0.0,
            camera_roll: 0.0,
            camera_position: [0.0; 3],
            camera_heading: 0,
            camera_roll_turn: 0,
            camera_pitch: 0,
            camera_offset_turns: [0.0; 3],
            camera_zoom: 0.0,
            kept_focus_point: [0; 9],
            camera_mode_is_2_or_3: 0,
            part_wake_counts: [0; 6],
            notice_id: 0,
            notice_counter: 0,
            notice_last: 0,
            notice_shown: 0,
            notice_time: 0,
            notice_time_b: 0,
            kept_view_place: [0.0; 3],
            surface_noise_a: 0,
            surface_noise_b: 0,
            surface_noise_second: 0,
            vertical_speed_change: 0.0,
            wheel_spring_force: [0; 4],
            wheel_surface_a: [0; 5],
            wheel_surface_b: [0; 5],
            point_wear_factor: [0.0; 5],
            wheel_cell_a_byte18: [0; 5],
            wheel_cell_b_byte18: [0; 5],
            wheel_cell_a_byte14: [0; 5],
            wheel_cell_b_byte14: [0; 5],
            wheel_surface_blend: [0.0; 5],
            car_heading: 0,
            car_pitch: 0,
            car_roll: 0,
            car_inverse_matrix: [0.0; 16],
            split_gaps: [0; 7],
            contact_point_x: 0,
            contact_point_ground: 0,
            contact_point_z: 0,
            contact_found: 0,
            contact_value: 0,
            part_damage_limits: [0; 7],
            saved_point_figures: [0; 20],
            start_slot_value: 0,
            shown_steering: 0,
            engine_sound_speed_int: 0,
            marks_on: 0,
            map_corner: [0; 2],
            map_redrawn: 0,
            map_place: [0; 2],
            map_picture_place: [0; 2],
            probe_point_x: [0.0; 64],
            probe_point_y: [0.0; 64],
            probe_point_z: [0.0; 64],
            probe_point_face: [0; 64],
            camera_ground_faces: [0; 5],
            group_mask: 0,
            engine_speed_scaled: 0.0,
            previous_step_clock: 0,
            wheel_ground_x: [0; 4],
            wheel_ground_height: [0; 4],
            wheel_ground_z: [0; 4],
            skid_grip: [0.0; 2],
            skid_slots: [0; 4],
            car_event_times: [0; 8],
            car_event_codes: [0; 8],
            car_event_count: 0,
            car_event_limit: 0,
            car_event_position: 0,
            two_car_place: 0,
            view_covered: 0,
            reset_marker: 0,
            value_jump_steps: 0,
            light_hold_steps: 0,
            eased_target: 0,
            eased_value: 0,
            glide_target: [0.0; 3],
            glide_steps_left: 0,
            kept_throttle_word: 0,
            kept_keys_word: 0,
            kept_steering_word: 0,
            kept_brake_word: 0,
        }
    }
}

/// Puts `values` into `fields`, the fields' dwords laid end to end in the order given, the first
/// value into dword `skip`. A value past the last field is refused (nothing is written for it).
pub(super) fn put_dwords(
    fields: &mut [&mut dyn crate::memory::plain_run::PlainRun],
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
