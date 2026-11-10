//! The work car (`g_car`, IDA 0x719FD8 .. 0x71AFB8; phase 7, typed): the car being stepped.
//! `drive::load_car` copies the current car's block (`g_carBlocks[n]`, the same layout,
//! IDA's `CarBlock`) into it and `drive::store_car` back; the physics step works on it.
//! Named from the code's uses (2026-10-25); most names are readings, `unknown_*` runs are
//! places no ported code names. `#[repr(C)]` with the image's layout, over the race
//! block's bytes (`CAR`) until the code reaching the car by RVA is typed.

use crate::race::owner::race;
use crate::session::{Bytes, Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Car {
    /// +0x0 (0x719FD8): the replay frame's head (camera words and bytes, `drive::replay_cameras`).
    pub frame_head: [Dword; 2],
    /// +0x8 (0x719FE0): the physics step's clock, +1 per running step.
    pub step_clock: Dword,
    /// +0xc (0x719FE4): a random word stirred each step.
    pub noise_word: Dword,
    /// +0x10 (0x719FE8): flags; only bit 0x10 read (steering trim negated).
    pub car_flags: Dword,
    /// +0x14 (0x719FEC): steps since the run began (0x320 the go point).
    pub run_ticks: Dword,
    /// +0x18 (0x719FF0): set once in gear at the start (reading).
    pub start_in_gear: Dword,
    /// +0x1c (0x719FF4): false-start penalty count.
    pub false_start_count: Dword,
    /// +0x20 (0x719FF8): set-up 0 .. 256 (reading).
    pub brake_balance: Dword,
    /// +0x24 (0x719FFC): the car model (at most 0x21).
    pub car_kind: Dword,
    /// +0x28 (0x71A000): set-up's gear ratios 0 .. 4.
    pub gear_ratio_setting: Dword,
    /// +0x2c (0x71A004): tyres 0 .. 5 (-1 unset).
    pub tyre_kind: Dword,
    /// +0x30 (0x71A008): set-up 0 .. 256.
    pub ride_height: Dword,
    /// +0x34 (0x71A00C): set-up 0 .. 256.
    pub suspension_stiffness: Dword,
    /// +0x38 (0x71A010): 2 automatic, else manual.
    pub gearbox_kind: Dword,
    /// +0x3c (0x71A014): set-up 0 .. 256.
    pub steering_sensitivity: Dword,
    /// +0x40 (0x71A018): drive layout (1 both axles).
    pub drive_kind: Dword,
    /// +0x44 (0x71A01C): only cleared at the start (`drive::clear_start_state`); nothing else uses it.
    pub unknown_44: Dword,
    /// +0x48 (0x71A020): per part (six): 1 once the part's wake count (+0x700) reached 10 and the part's sound object was woken (`race::wake_part_sounds`). Never cleared but with the block.
    pub part_woken: [Dword; 6],
    /// +0x60 (0x71A038): least wheel grip scaled; scales the ground's bumps.
    pub ground_bump_scale: Float,
    /// +0x64 (0x71A03C): eight 0 .. 1 gauges, one per octant of a hit's direction.
    pub impact_octant_marks: [Float; 8],
    /// +0x84 (0x71A05C): the lights switched on.
    pub lights_on: Dword,
    /// +0x88 (0x71A060): stage time in steps after the start.
    pub stage_time: Dword,
    /// +0x8c (0x71A064): race mode 2's countdown.
    pub time_left: Dword,
    /// +0x90 (0x71A068): splits crossed.
    pub splits_passed: Dword,
    /// +0x94 (0x71A06C): the checkpoint a reset puts the car back to.
    pub reset_checkpoint: Dword,
    /// +0x98 (0x71A070): the car's current checkpoint.
    pub checkpoint: Dword,
    /// +0x9c (0x71A074): the last checkpoint whose pace note was called.
    pub last_pace_call: Dword,
    /// +0xa0 (0x71A078): slow steps once going (reading).
    pub started_stand_timer: Dword,
    /// +0xa4 (0x71A07C): slow steps after "go" (reading).
    pub go_stand_timer: Dword,
    /// +0xa8 (0x71A080): set on finishing, then counts slow steps.
    pub finish_flag: Dword,
    /// +0xac (0x71A084): splits whose figure is kept.
    pub splits_recorded: Dword,
    /// +0xb0 (0x71A088): -1 stood still too long, 2 stopped after the finish.
    pub stop_state: Dword,
    /// +0xb4 (0x71A08C): g_crashMatrix: the 4 x 4 orientation, translation row at +0x30.
    pub crash_matrix: [Float; 16],
    /// +0xf4 (0x71A0CC): unreferenced bytes (0x40 of them).
    pub _gap_f4: Bytes<64>,
    /// +0x134 (0x71A10C): world position x, y, z (x 65536).
    pub position: [Dword; 3],
    /// +0x140 (0x71A118): one figure per split taken.
    pub split_times: [Dword; 16],
    /// +0x180 (0x71A158): drive speed less forward speed.
    pub drive_slip_speed: Float,
    /// +0x184 (0x71A15C): angular velocity x, y, z.
    pub spin: [Float; 3],
    /// +0x190 (0x71A168): unreferenced bytes (0x4 of them).
    pub _gap_190: Bytes<4>,
    /// +0x194 (0x71A16C): length of the spin.
    pub spin_length: Float,
    /// +0x198 (0x71A170): world-frame velocity.
    pub velocity: [Float; 3],
    /// +0x1a4 (0x71A17C): velocity in the car's frame (z forward: g_carSpeed).
    pub local_velocity: [Float; 3],
    /// +0x1b0 (0x71A188): unreferenced bytes (0x8 of them).
    pub _gap_1b0: Bytes<8>,
    /// +0x1b8 (0x71A190): driven wheels' speed (g_carDriveSpeed).
    pub drive_speed_a: Float,
    /// +0x1bc (0x71A194): second driven speed (g_carDriveSpeed2).
    pub drive_speed_b: Float,
    /// +0x1c0 (0x71A198): mean of the two.
    pub mean_drive_speed: Float,
    /// +0x1c4 (0x71A19C): length of the velocity.
    pub speed: Float,
    /// +0x1c8 (0x71A1A0): sideways speed held to a limit.
    pub side_speed_held: Float,
    /// +0x1cc (0x71A1A4): steps before the next gear change.
    pub gear_change_hold: Dword,
    /// +0x1d0 (0x71A1A8): drive cut after a shift (reading).
    pub drive_cut_steps: Dword,
    /// +0x1d4 (0x71A1AC): -1 reverse, 0 neutral, 1 .. 7 (g_carGear).
    pub gear: Dword,
    /// +0x1d8 (0x71A1B0): engine revs (g_carEngineSpeed).
    pub engine_speed: Float,
    /// +0x1dc (0x71A1B4): engine top revs.
    pub engine_speed_limit: Float,
    /// +0x1e0 (0x71A1B8): a copy of the car spec's scale (`spec_scale_1b8`) written by
    /// load_car_specs; no reader.
    pub spec_scale_copy: Dword,
    /// +0x1e4 (0x71A1BC): revs for the sound object.
    pub engine_sound_revs: Dword,
    /// +0x1e8 (0x71A1C0): steps stuck.
    pub stuck_count: Dword,
    /// +0x1ec (0x71A1C4): sent and received with the position message (`net::send_position`); nothing else reads or writes it.
    pub unknown_1ec: Dword,
    /// +0x1f0 (0x71A1C8): limiter cut counter.
    pub rev_cut_count: Dword,
    /// +0x1f4 (0x71A1CC): steps since a wall, edge or car contact (0x32 then down); a grip-recovery share.
    pub wall_hit_timer: Dword,
    /// +0x1f8 (0x71A1D0): the driven wheels' turn angles (reading: for drawing).
    pub driven_wheel_turns: [Dword; 2],
    /// +0x200 (0x71A1D8): the spin's yaw eased toward the grip-force sum (reading).
    pub eased_yaw_spin: Float,
    /// +0x204 (0x71A1DC): sum of the four grip forces.
    pub grip_force_sum: Float,
    /// +0x208 (0x71A1E0): steering pull from steering damage.
    pub steer_trim: Dword,
    /// +0x20c (0x71A1E4): nothing reads or writes it.
    pub unused_20c: Dword,
    /// +0x210 (0x71A1E8): drive_grip_forces' four terms: lateral, engine braking, yaw, lean (IDA CarBlock wheel_load).
    pub grip_forces: [Float; 4],
    /// +0x220 (0x71A1F8): flags: bit 2 damage notices, bit 8 sound speed scale.
    pub option_bits: Dword,
    /// +0x224 (0x71A1FC): x, y, z of the probe point that crossed an edge or hit.
    pub contact_point: [Float; 3],
    /// +0x230 (0x71A208): size of the contact.
    pub contact_depth: Float,
    /// +0x234 (0x71A20C): 1 on any contact this step.
    pub contact_flag: Dword,
    /// +0x238 (0x71A210): 1 a gear change happened, -1 refused.
    pub gear_changed: Dword,
    /// +0x23c (0x71A214): 1 from every gear change until `drive::set_listener` hands it to the effects' splines (`spline_gear_changed`) and clears it.
    pub gear_change_pending: Dword,
    /// +0x240 (0x71A218): the colour the face's light gives the car.
    pub light_colour: Dword,
    /// +0x244 (0x71A21C): light index 0..0xFF from the face's corner lights.
    pub light_level: Dword,
    /// +0x248 (0x71A220): 1 when the front wheels are off the ground.
    pub front_wheels_off: Dword,
    /// +0x24c (0x71A224): 1 when the rear wheels are off the ground.
    pub rear_wheels_off: Dword,
    /// +0x250 (0x71A228): 1 when the driven wheels are off the ground.
    pub driven_wheels_off: Dword,
    /// +0x254 (0x71A22C): 1 when all four wheels are off the ground.
    pub all_wheels_off: Dword,
    /// +0x258 (0x71A230): ground points on hard surfaces among the first four.
    pub hard_ground_count: Dword,
    /// +0x25c (0x71A234): 0 no driver, 1 a player's car, 2 a computer car (IDA grade_mode).
    pub driver_kind: Dword,
    /// +0x260 (0x71A238): probe points in use (at most 0x40).
    pub probe_count: Dword,
    /// +0x264 (0x71A23C): steps the reset key has been held.
    pub reset_key_count: Dword,
    /// +0x268 (0x71A240): steps stood in neutral (tyre change offered at 300).
    pub stopped_steps: Dword,
    /// +0x26c (0x71A244): 0 or -1 while racing, counts otherwise.
    pub tyre_change_state: Dword,
    /// +0x270 (0x71A248): steps far from the checkpoints.
    pub off_line_steps: Dword,
    /// +0x274 (0x71A24C): steps turned away from the checkpoint's heading.
    pub turned_steps: Dword,
    /// +0x278 (0x71A250): next time the off-line cue fires.
    pub off_line_cue_time: Dword,
    /// +0x27c (0x71A254): next time the turned-away cue fires.
    pub turned_cue_time: Dword,
    /// +0x280 (0x71A258): share blending the two surface records.
    pub surface_blend: Float,
    /// +0x284 (0x71A25C): 1 some parts gone, -1 others missing, 0 all present.
    pub parts_state: Dword,
    /// +0x288 (0x71A260): the wheels' slip speed (car_sound_state).
    pub slip_speed: Float,
    /// +0x28c (0x71A264): slip speed share for the voices.
    pub sound_slip_share: Float,
    /// +0x290 (0x71A268): 1.0, 0 when all wheels are off.
    pub sound_level: Float,
    /// +0x294 (0x71A26C): the motion's length scaled, for the voices.
    pub sound_speed: Float,
    /// +0x298 (0x71A270): bits 1..8: parts and tyres still on the car.
    pub parts_present: Dword,
    /// +0x29c (0x71A274): 1 when a drive-train gauge is at the top.
    pub damage_failed: Dword,
    /// +0x2a0 (0x71A278): brake fade gauge (reading).
    pub brake_fade: Float,
    /// +0x2a4 (0x71A27C): two damper wear gauges.
    pub damper_wear: [Float; 2],
    /// +0x2ac (0x71A284): brake temperature gauges.
    pub brake_heat: [Float; 2],
    /// +0x2b4 (0x71A28C): engine temperature gauge.
    pub engine_heat: Float,
    /// +0x2b8 (0x71A290): engine damage gauge.
    pub engine_damage: Float,
    /// +0x2bc (0x71A294): exhaust damage gauge.
    pub exhaust_damage: Float,
    /// +0x2c0 (0x71A298): turbo damage gauge.
    pub turbo_damage: Float,
    /// +0x2c4 (0x71A29C): cooling damage gauge.
    pub cooling_damage: Float,
    /// +0x2c8 (0x71A2A0): clutch damage gauge.
    pub clutch_damage: Float,
    /// +0x2cc (0x71A2A4): differential damage gauge.
    pub differential_damage: Float,
    /// +0x2d0 (0x71A2A8): per-gear wear, index gear + 1 (reverse, neutral, 1 .. 7).
    pub gear_wear: [Float; 9],
    /// +0x2f4 (0x71A2CC): gearbox damage gauge.
    pub gearbox_damage: Float,
    /// +0x2f8 (0x71A2D0): per-wheel hit gauges FL, FR, RL, RR.
    pub wheel_hit_damage: [Float; 4],
    /// +0x308 (0x71A2E0): general body-hit gauge.
    pub body_hit_damage: Float,
    /// +0x30c (0x71A2E4): body damage gauge.
    pub body_damage: Float,
    /// +0x310 (0x71A2E8): electrics damage gauge.
    pub electrics_damage: Float,
    /// +0x314 (0x71A2EC): lights' failure gauges (FL, FR, RL, RR, spots, brake light).
    pub light_damage: [Float; 6],
    /// +0x32c (0x71A304): the lights' damage gauge.
    pub lights_damage: Float,
    /// +0x330 (0x71A308): steering damage gauge.
    pub steering_damage: Float,
    /// +0x334 (0x71A30C): per-wheel suspension damage gauges.
    pub suspension_damage: [Float; 4],
    /// +0x344 (0x71A31C): the suspension's overall gauge.
    pub overall_suspension_damage: Float,
    /// +0x348 (0x71A320): brake wear gauge (axle not settled).
    pub brake_wear_a: Float,
    /// +0x34c (0x71A324): the other brake wear gauge.
    pub brake_wear_b: Float,
    /// +0x350 (0x71A328): brake damage gauge (handbrake failure ramp).
    pub brake_damage: Float,
    /// +0x354 (0x71A32C): the four tyres' wear, FL, FR, RL, RR, 0 ..= 100 (the developers' wear
    /// page's "tyre1" .. "tyre4"); `tyre_wear` is their overall figure.
    pub tyre_wears: [Float; 4],
    /// +0x364 (0x71A33C): tyre wear gauge.
    pub tyre_wear: Float,
    /// +0x368 (0x71A340): warning levels 0 .. 4, one per gauge group.
    pub damage_levels: [Dword; 35],
    /// +0x3f4 (0x71A3CC): engine health factor on the torque.
    pub engine_power_factor: Float,
    /// +0x3f8 (0x71A3D0): braking factors (a, b).
    pub brake_efficiency: [Float; 2],
    /// +0x400 (0x71A3D8): factors on the two drive-force outputs.
    pub drive_efficiency: [Float; 2],
    /// +0x408 (0x71A3E0): steering factor from the steering damage.
    pub steering_efficiency: Float,
    /// +0x40c (0x71A3E4): intermittent failures: electrics, then lights FL, FR, RL, RR, spots, brake.
    pub electrical_failures: [Dword; 7],
    /// +0x428 (0x71A400): set once by chance from the brake damage.
    pub handbrake_failed: Dword,
    /// +0x42c (0x71A404): by chance each step: the gearbox does nothing.
    pub gear_change_blocked: Dword,
    /// +0x430 (0x71A408): by chance each step: put in neutral.
    pub forced_neutral: Dword,
    /// +0x434 (0x71A40C): bit n: gear n - 1 barred (g_carGearBits).
    pub lost_gear_bits: Dword,
    /// +0x438 (0x71A410): per-wheel ride height %.
    pub wheel_ride_height: [Float; 4],
    /// +0x448 (0x71A420): per-wheel suspension stiffness %.
    pub wheel_stiffness: [Float; 4],
    /// +0x458 (0x71A430): mean of wheel_stiffness.
    pub stiffness_mean: Float,
    /// +0x45c (0x71A434): mean of wheel_ride_height.
    pub ride_height_mean: Float,
    /// +0x460 (0x71A438): side-hit cooldown (never started: game bug candidate).
    pub hit_cooldown: Dword,
    /// +0x464 (0x71A43C): throttle input.
    pub input_throttle: Dword,
    /// +0x468 (0x71A440): key and button bits.
    pub input_keys: Dword,
    /// +0x46c (0x71A444): steering input.
    pub input_steering: Dword,
    /// +0x470 (0x71A448): brake input.
    pub input_brake: Dword,
    /// +0x474 (0x71A44C): key bits newly pressed this step.
    pub keys_pressed: Dword,
    /// +0x478 (0x71A450): queued damage notices (at most 0x20).
    pub notice_count: Dword,
    /// +0x47c (0x71A454): queued notices' string ids.
    pub notice_ids: [Dword; 32],
    /// +0x4fc (0x71A4D4): queued notices' gauge values.
    pub notice_values: [Dword; 32],
    /// +0x57c (0x71A554): queued notices' ranks 1 .. 4.
    pub notice_ranks: [Dword; 32],
    /// +0x5fc (0x71A5D4): highest speed figure so far (reading).
    pub top_speed: Float,
    /// +0x600 (0x71A5D8): each wheel's spring compression 0 .. 1 (0 in the air).
    pub wheel_squeeze: [Float; 4],
    /// +0x610 (0x71A5E8): the previous step's wheel_squeeze.
    pub wheel_squeeze_last: [Float; 4],
    /// +0x620 (0x71A5F8): x and z in the checkpoint's frame.
    pub checkpoint_offset: [Dword; 2],
    /// +0x628 (0x71A600): mode 2's gaps in checkpoints to the nearest car ahead and behind
    /// (`drive::note_gap_ahead` / `note_gap_behind`, block 0's; u32::MAX: none).
    pub pace_gaps: [Dword; 2],
    /// +0x630 (0x71A608): the gaps eased towards those wanted, ahead and behind (the arcade
    /// panel reads them).
    pub pace_gaps_eased: [Dword; 2],
    /// +0x638 (0x71A610): throttle pedal 0 .. 0x100.
    pub throttle: Dword,
    /// +0x63c (0x71A614): brake pedal 0 .. 0x100.
    pub brake: Dword,
    /// +0x640 (0x71A618): handbrake 0 .. 0x100 and over.
    pub handbrake: Dword,
    /// +0x644 (0x71A61C): steering for the wheels, +-0x20000000.
    pub steering: Dword,
    /// +0x648 (0x71A620): steering unscaled by speed.
    pub steering_raw: Dword,
    /// +0x64c (0x71A624): steering + trim.
    pub steer_angle: Dword,
    /// +0x650 (0x71A628): raw steering + 4 x trim (reading).
    pub steer_angle_visual: Dword,
    /// +0x654 (0x71A62C): the place in race mode 2.
    pub mode2_place: Dword,
    /// +0x658 (0x71A630): the cars behind in mode 2.
    pub mode2_behind: Dword,
    /// +0x65c (0x71A634): mode 2's two pace cars (car numbers: the one behind the lead,
    /// the one ahead; `drive::start_cars_behind`, block 0's).
    pub pace_cars: [Dword; 2],
    /// +0x664 (0x71A63C): the start numbers they were put at.
    pub pace_starts: [Dword; 2],
    /// +0x66c (0x71A644): 1 once the pace cars are placed.
    pub pace_placed: Dword,
    /// +0x670 (0x71A648): the checkpoints between the two pace cars.
    pub pace_apart: Dword,
    /// +0x674 (0x71A64C): push apart of the two pace cars (`start_cars_behind`: +-(8 - gap) x scale), added to the start blend; 0 when they are 8 or more checkpoints apart.
    pub pace_push: Float,
    /// +0x678 (0x71A650): grid-place share.
    pub start_share: Float,
    /// +0x67c (0x71A654): checkpoints past the lead's start, scaled.
    pub start_blend_share: Float,
    /// +0x680 (0x71A658): the start blend, held 0 .. 1.
    pub start_blend: Float,
    /// +0x684 (0x71A65C): the car's camera mode 0 .. 0x33.
    pub camera_view: Dword,
    /// +0x688 (0x71A660): the blend's second camera mode.
    pub camera_blend_view: Dword,
    /// +0x68c (0x71A664): mode before the low-camera switch + 1.
    pub camera_view_return: Dword,
    /// +0x690 (0x71A668): clock when the camera switch began.
    pub camera_switch_clock: Dword,
    /// +0x694 (0x71A66C): view-switch flag: 1 while the camera changes mode.
    pub camera_switch_active: Dword,
    /// +0x698 (0x71A670): steps the switch lasts (10 when begun).
    pub camera_switch_length: Dword,
    /// +0x69c (0x71A674): blend share 0..1 of the switch.
    pub camera_switch_share: Float,
    /// +0x6a0 (0x71A678): eased roll of the view.
    pub camera_roll: Float,
    /// +0x6a4 (0x71A67C): camera x, y, z (whole numbers).
    pub camera_position: [Dword; 3],
    /// +0x6b0 (0x71A688): camera heading, whole-number units << 4.
    pub camera_heading: Dword,
    /// +0x6b4 (0x71A68C): third camera turn (reading).
    pub camera_roll_turn: Dword,
    /// +0x6b8 (0x71A690): camera pitch, units << 4, negated.
    pub camera_pitch: Dword,
    /// +0x6bc (0x71A694): three turns (<< 16) applied to the camera's offset.
    pub camera_offset_turns: [Dword; 3],
    /// +0x6c8 (0x71A6A0): zoom of the view.
    pub camera_zoom: Float,
    /// +0x6cc (0x71A6A4): the first three: the view's focus point kept, the car's position with y + 0x8000 (`race::frame_draw::view`); the other six unused.
    pub kept_focus_point: [Dword; 9],
    /// +0x6f0 (0x71A6C8): 1 when the camera mode is 2 or 3.
    pub camera_mode_is_2_or_3: Dword,
    /// +0x6f4 (0x71A6CC): unreferenced bytes (0x8 of them).
    pub _gap_6f4: Bytes<8>,
    /// +0x6fc (0x71A6D4): the car's state bits (lights, seen from inside 0x10, casts a
    /// shadow; `race::car_draw`, `race::car_shadow`, `race::frame_views`); now each view's
    /// (`race::car_draw_state::CarDrawState::car_states`); unused.
    pub state_bits: Dword,
    /// +0x700 (0x71A6D8): the six parts' wake counts (`race::frame_draw`'s lights,
    /// `drive::share_car_state`).
    pub part_wake_counts: [Dword; 6],
    /// +0x718 (0x71A6F0): the engine speed shown (+0x1D8 as a float, not below
    /// `flt_6140F8`; `race::car_parts`), drawn by the panel and the car's number.
    pub shown_engine_speed: Float,
    /// +0x71c (0x71A6F4): id of the notice shown (0 none).
    pub notice_id: Dword,
    /// +0x720 (0x71A6F8): the notice's counter: the name plates set it to 1 when drawn,
    /// `race::frame_views` counts it down as a view's textures are forgotten, `false_start`
    /// reads and clears it.
    pub notice_counter: Dword,
    /// +0x724 (0x71A6FC): id of the last notice shown.
    pub notice_last: Dword,
    /// +0x728 (0x71A700): 1 while a notice is shown.
    pub notice_shown: Dword,
    /// +0x72c (0x71A704): the notice's time, counted down.
    pub notice_time: Dword,
    /// +0x730 (0x71A708): second notice time.
    pub notice_time_b: Dword,
    /// +0x734 (0x71A70C): the view's place kept with the car (`race::frame_draw`).
    pub kept_view_place: [Dword; 3],
    /// +0x740 (0x71A718): the four wheels' matrices, 16 floats each, kept by
    /// `race::car_parts` for `race::car_draw`.
    pub part_matrices: [Float; 64],
    /// +0x840 (0x71A818): the body's turn matrix (`race::car_draw`, `race::car_shadow`,
    /// `drive::set_listener`).
    pub body_matrix: [Float; 16],
    /// +0x880 (0x71A858): the car's place matrix (`flt_641760` kept: `race::car_draw`,
    /// `drive::set_listener`).
    pub place_matrix: [Float; 16],
    /// +0x8c0 (0x71A898): the body's turn times its place (`race::car_parts`).
    pub whole_matrix: [Float; 16],
    /// +0x900 (0x71A8D8): noise sample 1 scaled.
    pub surface_noise_a: Dword,
    /// +0x904 (0x71A8DC): same sample 1 value.
    pub surface_noise_b: Dword,
    /// +0x908 (0x71A8E0): noise sample 2 scaled.
    pub surface_noise_second: Dword,
    /// +0x90c (0x71A8E4): the car's vertical velocity less the vertical speed kept from the step before (`physics::car_physics_step`); `physics::camera_frame` scales it into the camera's shake. Probable.
    pub vertical_speed_change: Dword,
    /// +0x910 (0x71A8E8): per-wheel whole-number value from the wheel's squeeze.
    pub wheel_spring_force: [Dword; 4],
    /// +0x920 (0x71A8F8): per ground point: surface kind of the first cell (5 wet).
    pub wheel_surface_a: [Dword; 5],
    /// +0x934 (0x71A90C): per ground point: surface kind of the second cell.
    pub wheel_surface_b: [Dword; 5],
    /// +0x948 (0x71A920): per point: the surface's `wear_factor` figure blended between the two
    /// surfaces by `wheel_surface_blend`.
    pub point_wear_factor: [Dword; 5],
    /// +0x95c (0x71A934): per point: byte +0x18 of the first cell's record.
    pub wheel_cell_a_byte18: [Dword; 5],
    /// +0x970 (0x71A948): same for the second cell.
    pub wheel_cell_b_byte18: [Dword; 5],
    /// +0x984 (0x71A95C): per point: byte +0x14 of the first cell's record.
    pub wheel_cell_a_byte14: [Dword; 5],
    /// +0x998 (0x71A970): same for the second cell.
    pub wheel_cell_b_byte14: [Dword; 5],
    /// +0x9ac (0x71A984): per point: blend share 0..1 between the two surfaces.
    pub wheel_surface_blend: [Float; 5],
    /// +0x9c0 (0x71A998): car heading, units << 4.
    pub car_heading: Dword,
    /// +0x9c4 (0x71A99C): car pitch (reading), units << 4.
    pub car_pitch: Dword,
    /// +0x9c8 (0x71A9A0): car roll (reading), units << 4.
    pub car_roll: Dword,
    /// +0x9cc (0x71A9A4): g_carInverseMatrix.
    pub car_inverse_matrix: [Float; 16],
    /// +0xa0c (0x71A9E4): the split's figures (figure, gaps, names, two more).
    pub split_gaps: [Dword; 7],
    /// +0xa28 (0x71AA00): crash contact point x.
    pub contact_point_x: Dword,
    /// +0xa2c (0x71AA04): ground height under the contact point.
    pub contact_point_ground: Dword,
    /// +0xa30 (0x71AA08): contact point z.
    pub contact_point_z: Dword,
    /// +0xa34 (0x71AA0C): 1 when the body touches.
    pub contact_found: Dword,
    /// +0xa38 (0x71AA10): the contact's depth or force (reading).
    pub contact_value: Dword,
    /// +0xa3c (0x71AA14): seven parts' damage figures.
    pub part_damage_limits: [Dword; 7],
    /// +0xa58 (0x71AA30): the block's copy of the physics tables' 0x614280 .. (the five points' loads, grips and the like); `drive::share_car_state` fills +0xA5C .., `race::car_starts` clears them. Probable.
    pub saved_point_figures: [Dword; 20],
    /// +0xaa8 (0x71AA80): the start position's table entry (unknown).
    pub start_slot_value: Dword,
    /// +0xaac (0x71AA84): the steering drawn (from +0x650, `race::car_parts`).
    pub shown_steering: Dword,
    /// +0xab0 (0x71AA88): speed scaled, whole number (engine sound).
    pub engine_sound_speed_int: Dword,
    /// +0xab4 (0x71AA8C): the marks switched on (`race::car_parts`).
    pub marks_on: Dword,
    /// +0xab8 (0x71AA90): the mode's marks on (`race::car_parts`; `race::car_draw`'s sound
    /// groups).
    pub mode_on: Dword,
    /// +0xabc (0x71AA94): the marks' bytes (the count x 0x20; `race::car_parts`,
    /// `race::frame_draw`'s headlights).
    pub marks_count: Dword,
    /// +0xac0 (0x71AA98): the parts' switch bytes (`rc2k_core::race::car_parts::
    /// mark_switches`).
    pub part_switches: Bytes<0x11>,
    /// +0xad1 (0x71AAA9): unreferenced bytes (0x3 of them).
    pub _gap_ad1: Bytes<3>,
    /// +0xad4 (0x71AAAC): the course map picture's corner x, y (16-pixel steps;
    /// `race::map_scroll`).
    pub map_corner: [Dword; 2],
    /// +0xadc (0x71AAB4): 1 when the map picture was redrawn this step.
    pub map_redrawn: Dword,
    /// +0xae0 (0x71AAB8): the car on the course map x, y (16.16).
    pub map_place: [Dword; 2],
    /// +0xae8 (0x71AAC0): the car's place in the map picture x, y.
    pub map_picture_place: [Dword; 2],
    /// +0xaf0 (0x71AAC8): x of the 0x40 ground probe points.
    pub probe_point_x: [Float; 64],
    /// +0xbf0 (0x71ABC8): y of the probe points.
    pub probe_point_y: [Float; 64],
    /// +0xcf0 (0x71ACC8): z of the probe points.
    pub probe_point_z: [Float; 64],
    /// +0xdf0 (0x71ADC8): face found last under each probe point.
    pub probe_point_face: [Dword; 64],
    /// +0xef0 (0x71AEC8): last ground faces under the camera and its sides (five saved by
    /// `physics::camera_avoid_ground`).
    pub camera_ground_faces: [Dword; 5],
    /// +0xf04 (0x71AEDC): the object groups drawn with the car, which the original's
    /// `race::car_draw` stored (0x461CD8) and nothing ever read; not stored any more; unused.
    pub group_mask: Dword,
    /// +0xf08 (0x71AEE0): speed scaled (engine sound).
    pub engine_speed_scaled: Float,
    /// +0xf0c (0x71AEE4): the step clock as of the last replay step (`drive::replay_step`).
    pub previous_step_clock: Dword,
    /// +0xf10 (0x71AEE8): world x of the four wheels.
    pub wheel_ground_x: [Dword; 4],
    /// +0xf20 (0x71AEF8): ground height under the four wheels.
    pub wheel_ground_height: [Dword; 4],
    /// +0xf30 (0x71AF08): world z of the four wheels.
    pub wheel_ground_z: [Dword; 4],
    /// +0xf40 (0x71AF18): the front and rear axles' skid grip (`physics::drive_wheels_step`;
    /// `race::lay_skidmarks` lays marks from `flt_643AD0` up).
    pub skid_grip: [Float; 2],
    /// +0xf48 (0x71AF20): each wheel's last skid point (-1 none; `race::lay_skidmarks`).
    pub skid_slots: [Dword; 4],
    /// +0xf58 (0x71AF30): timed-event queue times.
    pub car_event_times: [Dword; 8],
    /// +0xf78 (0x71AF50): event codes of the queue.
    pub car_event_codes: [Dword; 8],
    /// +0xf98 (0x71AF70): queued events (max 7).
    pub car_event_count: Dword,
    /// +0xf9c (0x71AF74): the pace-note symbols' scroll target (`rc2k_core::physics::car_events`: 0x20 less per queued entry).
    pub car_event_limit: Dword,
    /// +0xfa0 (0x71AF78): the symbols' scroll position (y offset from 400 on the HUD, `race::panel_marks`), eased 4 a step to the target.
    pub car_event_position: Dword,
    /// +0xfa4 (0x71AF7C): the car's place on the panel (`drive::two_car_order` writes it, `race::frame_draw::panel` reads it).
    pub two_car_place: Dword,
    /// +0xfa8 (0x71AF80): set to 1 by `race::drop_absent_player` for the player who is out of a two-player race (the first whose `stop_state` is set; also clears `driver_kind`): the view is then not drawn (`draw_view_scene` and `draw_view_overlays` only tint it) and its notice strip stays hidden (`hud::strips`). Zeroed by the block reset. (The only IDA readers are `draw_view_scene` and `draw_view_overlays`; the hud reads it too.)
    pub view_covered: Dword,
    /// +0xfac (0x71AF84): set to 10 by the resets; no reader.
    pub reset_marker: Dword,
    /// +0xfb0 (0x71AF88): steps while ease_car_value jumps to the target.
    pub value_jump_steps: Dword,
    /// +0xfb4 (0x71AF8C): steps while car_lighting holds the light change.
    pub light_hold_steps: Dword,
    /// +0xfb8 (0x71AF90): ease_car_value's target.
    pub eased_target: Dword,
    /// +0xfbc (0x71AF94): the value eased towards eased_target << 8.
    pub eased_value: Dword,
    /// +0xfc0 (0x71AF98): where a position received from the network is to be glided to
    /// (x, y, z; `net::apply_message`, stepped towards by `net::follow_remote`).
    pub glide_target: [Dword; 3],
    /// +0xfcc (0x71AFA4): the steps of the glide left (set to 32 by `net::apply_message`,
    /// one less each step in `net::follow_remote`).
    pub glide_steps_left: Dword,
    /// +0xfd0 (0x71AFA8): the throttle input word kept for the network and replays (first of the
    /// four `drive::car_input_words` copies; the car's `input_throttle`).
    pub kept_throttle_word: Dword,
    /// +0xfd4: the key bits word (the car's `input_keys`).
    pub kept_keys_word: Dword,
    /// +0xfd8: the steering word (the car's `input_steering`).
    pub kept_steering_word: Dword,
    /// +0xfdc: the brake word (the car's `input_brake`).
    pub kept_brake_word: Dword,
}

const _: () = assert!(core::mem::size_of::<Car>() == 0xFE0);

impl Car {
    /// The car block at `address` (a `g_carBlocks` entry, a view's block, `g_car`); any other
    /// address ends the program (`trap::car_at`).
    pub fn at(address: u32) -> &'static Car {
        crate::race::trap::car_at(address)
            .unwrap_or_else(|e| crate::fatal::raise::end("physics::Car::at", e))
    }

    /// The work car (`CAR`) or a car block when `address` is its first byte, found by
    /// arithmetic on their game addresses (statics: found once); none for any other
    /// address, which the callers then look up among the regions.
    pub fn known(address: u32) -> Option<&'static Car> {
        let shared_store = &crate::race::owner::race().shared;

        use crate::race::blocks_store::CAR_BLOCKS;
        static HOME: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
        let &car = HOME
            .get_or_init(|| crate::pointers::game_address(core::ptr::from_ref(&shared_store.car)));
        if address == car {
            return Some(&shared_store.car);
        }
        Self::block_number(address).map(|n| &CAR_BLOCKS.cars[n])
    }

    /// The number of the car block whose first byte is at `address` (none for the work car or
    /// any other address).
    pub fn block_number(address: u32) -> Option<usize> {
        use crate::race::blocks_store::{BLOCK_COUNT, BLOCK_LEN, blocks_addr};
        static BLOCKS: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
        let blocks = *BLOCKS.get_or_init(|| blocks_addr(&race().replay));
        let off = address.wrapping_sub(blocks) as usize;
        (off.is_multiple_of(BLOCK_LEN) && off / BLOCK_LEN < BLOCK_COUNT).then_some(off / BLOCK_LEN)
    }
}

impl Car {
    /// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
    /// `race::store::LEFT`), with the image's first values.
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x719FD8, the first byte of the piece.
pub const START: usize = 0x719FD8;
/// IDA 0x71AFB8, the end of the piece.
pub const END: usize = 0x71AFB8;

const _: () = assert!(END - START == core::mem::size_of::<Car>());

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::offset_of;

    #[test]
    fn known_cars_are_the_records_their_addresses_name() {
        use crate::race::blocks_store::{BLOCK_COUNT, block_addr};
        let by_region = |at: u32| {
            crate::pointers::GamePtr::new(at)
                .record::<Car>("a test's car")
                .ok()
                .map(core::ptr::from_ref)
        };
        let at = crate::pointers::game_address(core::ptr::from_ref(
            &crate::race::owner::race().shared.car,
        ));
        assert_eq!(Car::known(at).map(core::ptr::from_ref), by_region(at));
        for n in 0..BLOCK_COUNT {
            let at = block_addr(n);
            assert_eq!(
                Car::known(at).map(core::ptr::from_ref),
                by_region(at),
                "block {n}"
            );
            assert!(Car::known(at + 4).is_none(), "inside block {n}");
        }
        assert!(Car::known(block_addr(BLOCK_COUNT)).is_none());
        assert!(Car::known(0).is_none());
    }

    #[test]
    fn car_starts_as_the_games() {
        let shared_store = &crate::race::owner::race().shared;

        // other tests write the work car while they run
        let _lock = crate::store::test_lock();
        assert_eq!(offset_of!(Car, crash_matrix), 0xB4);
        assert_eq!(offset_of!(Car, state_bits), 0x6FC);
        assert_eq!(offset_of!(Car, part_wake_counts), 0x700);
        assert_eq!(offset_of!(Car, part_matrices), 0x740);
        assert_eq!(offset_of!(Car, body_matrix), 0x840);
        assert_eq!(offset_of!(Car, whole_matrix), 0x8C0);
        assert_eq!(offset_of!(Car, shown_engine_speed), 0x718);
        assert_eq!(offset_of!(Car, part_switches), 0xAC0);
        assert_eq!(offset_of!(Car, map_corner), 0xAD4);
        assert_eq!(offset_of!(Car, skid_slots), 0xF48);
        assert_eq!(offset_of!(Car, gear), 0x1D4);
        assert_eq!(offset_of!(Car, probe_point_face), 0xDF0);
        let car_addr = || crate::pointers::game_address(core::ptr::from_ref(&shared_store.car));
        assert_eq!(car_addr() % 4, 0);
        assert_eq!(
            car_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            crate::pointers::game_address(&raw const shared_store.car.gear),
            car_addr() + 0x1D4
        );
        let mine = unsafe {
            std::slice::from_raw_parts(crate::pointers::host(car_addr()).cast_const(), END - START)
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
    }
}
