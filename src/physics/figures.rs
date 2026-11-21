//! The physics figures nothing writes, as constants: the values the car physics reads. Floats
//! are loaded as the x87 loads a float from memory (`x87_float::ld`). The fixed float constants the
//! x87 steps read are `race::numbers_store::Numbers` fields (initialised from these); the figures
//! the car file or a patch writes stay cells in `physics::tables_store::PhysicsTables`.

/// 5.0: a reset is allowed below this speed (`drive::reset_request`).
pub const RESET_SPEED_LIMIT: f32 = 5.0;

/// 0.01: the physics step's length in seconds.
pub const STEP_TIME: f32 = 0.01;

/// 8.0: below it the car may stall (`physics::stall_when_still`).
pub const STALL_SPEED: f32 = 8.0;

/// 0.5 (`physics::stall_when_still`).
pub const STALL_SHARE: f32 = 0.5;

/// 800.0 (`physics::cap_drive_speed`).
pub const DRIVE_SPEED_CAP: f32 = 800.0;

/// 20.0 (`physics::cap_drive_speed`).
pub const DRIVE_SPEED_CAP_A: f32 = 20.0;

/// 20.0 (`physics::cap_drive_speed`).
pub const DRIVE_SPEED_CAP_B: f32 = 20.0;

/// 20.0 (`physics::ease_low_speed_forces`).
pub const LOW_SPEED_LIMIT: f32 = 20.0;

/// 1.0 (`physics::ease_low_speed_forces`).
pub const LOW_SPEED_ADD: f32 = 1.0;

/// 0.5 (`physics::ease_low_speed_forces`).
pub const LOW_SPEED_SCALE: f32 = 0.5;

/// 4.0: the spin held to it (`physics::limit_motion`).
pub const SPIN_LIMIT: f32 = 4.0;

/// 80.0: the motion held to it (`physics::limit_motion`).
pub const VELOCITY_LIMIT: f32 = 80.0;

/// 7.5 (`physics::drive_grip_forces`).
pub const GRIP_SLIDE_SCALE: f32 = 7.5;

/// 5.0 (`physics::drive_grip_forces`).
pub const GRIP_SLIP_LIMIT: f32 = 5.0;

/// 10.0, held both ways (`physics::drive_grip_forces`).
pub const GRIP_FORCE_LIMIT: f32 = 10.0;

/// 1.0 (`physics::drive_grip_forces`).
pub const GRIP_SLOW_SPEED: f32 = 1.0;

/// 0.9 (`physics::load_car_specs`).
pub const SPEC_SCALE_A4: f32 = 0.9;

/// 0.15 (`physics::probe_ground`).
pub const PROBE_LIFT: f32 = 0.15;

/// 1.0 (`physics::auto_gearbox`).
pub const AUTO_GEAR_SPEED_UP: f32 = 1.0;

/// -1.0 (`physics::auto_gearbox`).
pub const AUTO_GEAR_SPEED_DOWN: f32 = -1.0;

/// 0.95: the engine's speed eased by it (`physics::gearbox_step`).
pub const ENGINE_EASE: f32 = 0.95;

/// 0.1 (`physics::drive_grip_forces`).
pub const GRIP_EASE_STEP: f32 = 0.1;

/// 0.5 (`physics::drive_grip_forces`).
pub const SIDE_GRIP_SHARE: f32 = 0.5;

/// 8500.0 (`physics::load_car_specs`).
pub const REV_LIMIT_SCALE: f32 = 8500.0;

/// 0.65 (`physics::spin_wheels`).
pub const WHEEL_SPIN_SHARE: f32 = 0.65;

/// 0.325 (`physics::wheel_springs`).
pub const SPRING_SHARE: f32 = 0.325;

/// 2.5 (`physics::drive_wheels_step`).
pub const DRIVE_SPEED_SCALE: f32 = 2.5;

/// 15.0, held both ways (`physics::car_physics_step`).
pub const SIDE_SPEED_LIMIT: f32 = 15.0;

/// 0.05 (`physics::drive_grip_forces`).
pub const SIDE_FORCE_SCALE: f32 = 0.05;

/// 0.5 (`physics::drive_wheels_step`).
pub const SLIP_BLEND: f32 = 0.5;

/// 0.5 (`physics::drive_wheels_step`).
pub const SLIP_SHARE: f32 = 0.5;

/// 1.5 (`physics::drive_wheels_step`).
pub const WHEEL_SPEED_MARGIN: f32 = 1.5;

/// 0.1 (`physics::drive_wheels_step`).
pub const WHEEL_SPEED_GAP: f32 = 0.1;

/// 1.0 (`physics::car_sound_state`).
pub const SOUND_SPEED_SCALE: f32 = 1.0;

/// 8 (`physics::drive_torque_share`).
pub const TORQUE_SHARE_A: u32 = 8;

/// 0x20 (`physics::drive_torque_share`).
pub const TORQUE_SHARE_B: u32 = 32;

/// 32.0's bits, divided by as an integer (`physics::drive_torque_share`).
pub const TORQUE_SHARE_C: i32 = 0x42000000;

/// 0x1000 (`physics::crash_contact`).
pub const CRASH_DEPTH: u32 = 0x1000;

/// 2.4, copied into the spec's grip (`physics::load_car_specs`).
pub const SPEC_DEFAULT_BA0: f32 = 2.4;

/// 0.85 (`physics::load_car_specs`).
pub const SPEC_SCALE_1B8: f32 = 0.85;

/// 4.0, copied into the spec's wheel figure.
pub const SPEC_DEFAULT_BA8: f32 = 4.0;

/// 0.95 (`physics::auto_gearbox`).
pub const AUTO_GEAR_ENGINE_SHARE: f32 = 0.95;

/// 2.0 (`physics::drive_grip_forces`).
pub const GRIP_RAMP: f32 = 2.0;

/// 0.25 (`physics::drive_grip_forces`).
pub const GRIP_RAMP_SHARE: f32 = 0.25;

/// 1.0 (`physics::settle_drive_speeds`).
pub const SETTLE_FAST_STEP: f32 = 1.0;

/// 0.01 (`physics::settle_drive_speeds`).
pub const SETTLE_SLOW_STEP: f32 = 0.01;

/// 0.5 (`physics::probe_ground`, `physics::wheel_springs`).
pub const PROBE_SHARE: f32 = 0.5;

/// 2.0 (`physics::wheel_springs`).
pub const SPRING_FORCE_SCALE: f32 = 2.0;

/// 0.25 (`physics::engine_torque`).
pub const TORQUE_EASE: f32 = 0.25;

/// 0.0001 (`physics::set_downforce`).
pub const DOWNFORCE_FACTOR: f32 = 0.0001;

/// 0.7 (`physics::update_camera`, the car's shadow).
pub const CAMERA_DROP: f32 = 0.7;

/// 0xB333: the car's places dropped by it (`race::start_place`, `scene::add_object_shape`).
pub const GROUND_DROP: u32 = 0xB333;

/// 5.0 (`physics::air_drag`, the test drive).
pub const AIR_DRAG: f32 = 5.0;

/// 0.75 (the test drive).
pub const ROLLING_DRAG: f32 = 0.75;

/// 5.0 (`physics::free_rev`).
pub const FREE_REV_ADD: f32 = 5.0;

/// 0.005 (`physics::free_rev`).
pub const FREE_REV_SCALE: f32 = 0.005;

/// 0.9 (`physics::find_clear_place`).
pub const CLEAR_PLACE_SHARE: f32 = 0.9;

/// 0.99 (`physics::pick_start_gear`).
pub const START_GEAR_SHARE: f32 = 0.99;

/// 1.2 (`physics::load_car_specs`).
pub const REV_TOP_SCALE: f32 = 1.2;

/// 0.001 (`physics::make_gear_ratios`).
pub const GEAR_RATIO_STEP: f32 = 0.001;

/// 0.0025 (`physics::wheel_surfaces`).
pub const WHEEL_LOAD_SCALE: f32 = 0.0025;

/// 0.02, held both ways (`physics::camera_frame`).
pub const CAMERA_FRAME_LIMIT: f32 = 0.02;

/// 0.05 (`physics::camera_frame`).
pub const CAMERA_FRAME_SCALE: f32 = 0.05;

/// 10: the contact steps' limit.
pub const CONTACT_STEPS: u32 = 10;

/// (`physics::blend_wheel_load`).
pub const WHEEL_LOAD_LIMIT: f32 = 4.0;

/// (`physics::off_line_warnings`).
pub const OFF_LINE_DISTANCE: f32 = 32.0;

/// (`physics::passed_split_point`).
pub const SPLIT_DISTANCE: f32 = 32.0;

/// (`physics::off_line_warnings`).
pub const OFF_LINE_WORD: u32 = 0x60000000;

/// (`physics::pace_notes_step`).
pub const PACE_SCALE: f32 = 1.0;

/// 2.4 / 1.6 by drive kind 1 or not (`physics::drive_grip_forces`).
pub const GRIP_FRONT_DRIVE: [f32; 2] = [2.4, 1.6];

/// 0.5 / 0.4 by drive kind 1 or not (`physics::drive_grip_forces`).
pub const GRIP_REAR_DRIVE: [f32; 2] = [0.5, 0.4];

/// 2.0 / 2.2 by drive kind (`physics::drive_grip_forces`).
pub const GRIP_FACTOR: [f32; 2] = [2.0, 2.2];

/// 0.0045 / 0.0025 by drive kind (`physics::add_drive_forces`).
pub const DRIVE_FORCE_FACTOR: [f32; 2] = [0.0045, 0.0025];

/// `physics::set_handling`'s (lo, hi) pairs for three of its figures.
pub const HANDLING_RANGES_A: [f32; 6] = [0.1, 0.9, 0.5, 1.5, 0.25, 1.25];

/// `physics::set_handling`'s pairs for two more figures (the second chosen by kind).
pub const HANDLING_RANGES_B: [f32; 6] = [0.1, 0.2, 0.3, 0.16, 0.4, 0.26];

/// `physics::set_handling`'s pairs for four more figures.
pub const HANDLING_RANGES_C: [f32; 8] = [0.15, 0.1, 100.0, 200.0, 1.0, 2.0, 0.45, 0.3];

/// The events `physics::queue_listed_event` lists by number; -1 for none.
pub const LISTED_EVENTS: [u32; 32] = [
    0xFFFF_FFFF,
    8,
    9,
    10,
    11,
    12,
    13,
    14,
    15,
    8,
    9,
    10,
    11,
    12,
    0xFFFF_FFFF,
    0xFFFF_FFFF,
    0xFFFF_FFFF,
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    0,
    1,
    2,
    3,
    4,
    0xFFFF_FFFF,
    0xFFFF_FFFF,
];

/// Each pace-note slot's byte in the face (`race::pace_notes`).
pub const PACE_SLOT_OFFSETS: [u32; 8] = [40, 41, 42, 43, 25, 26, 27, 32];

/// The engine's torque curve read in pairs (`physics::engine_torque`).
pub const TORQUE_CURVE: [f32; 9] = [0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.0];

/// 5: the steps the drive is cut after a change.
pub const DRIVE_CUT_STEPS: u32 = 5;

/// `physics::find_clear_place`'s first step table.
pub const CLEAR_PLACE_STEPS_A: [u32; 16] = [
    0,
    7,
    10,
    7,
    0,
    0xFFFF_FFF9,
    0xFFFF_FFF6,
    0xFFFF_FFF9,
    0,
    10,
    14,
    10,
    0,
    0xFFFF_FFF6,
    0xFFFF_FFF2,
    0xFFFF_FFF6,
];

/// Its second step table.
pub const CLEAR_PLACE_STEPS_B: [u32; 16] = [
    10,
    7,
    0,
    0xFFFF_FFF9,
    0xFFFF_FFF6,
    0xFFFF_FFF9,
    0,
    7,
    14,
    10,
    0,
    0xFFFF_FFF6,
    0xFFFF_FFF2,
    0xFFFF_FFF6,
    0,
    10,
];

/// Eight tables of 13 dwords the ground kinds' records point at: [0] then six figures by tyre kind
/// and six more (`physics::wheel_surfaces`).
#[rustfmt::skip]
pub const SURFACE_TABLES: [u32; 104] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 120, 120, 115, 105, 90, 80, 75, 80, 95, 105, 85, 75, 75,
    85, 65, 70, 75, 85, 80, 65, 60, 65, 70, 75, 80, 65, 75, 45, 50, 55, 75, 70, 55, 40, 45, 50, 65,
    70, 55, 75, 40, 45, 50, 65, 70, 75, 40, 45, 50, 65, 70, 75, 70, 40, 45, 50, 70, 65, 50, 35, 40,
    45, 60, 65, 50, 75, 35, 40, 45, 60, 65, 75, 35, 40, 45, 60, 65, 75, 65, 25, 30, 35, 50, 55, 65,
    25, 30, 35, 50, 55, 65,
];
