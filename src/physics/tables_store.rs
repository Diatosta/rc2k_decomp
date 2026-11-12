//! The physics' tables and work variables (IDA 0x613828 .. 0x614D38; phase 7, typed): the
//! race clock, the inputs, the ground kinds' records, the handling constants, the car's
//! working figures, the force and contact records, the gearbox and the spec's figures.
//! Named from the code's uses (2026-10-25); most names are readings, `unknown_*` runs are
//! places no ported code names. `#[repr(C)]` with the image's layout, over the race block's
//! bytes (`PHYSICS`, as `crate::race::owner::race().shared.car()`) until the code reaching them by RVA from
//! the race block is typed.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{PlainDword, PlainFloat};

/// A kind of ground (0x30 bytes, `PhysicsTables.surfaces[0].table`): what `physics::wheel_surfaces` blends
/// between for each point, and `physics::apply_patches` patches (readings).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Surface {
    /// +0x0: its table in `surface_tables` (an address).
    pub table: PlainDword,
    /// +0x4: slide grip (`patchslidegrip`).
    pub slide_grip: PlainFloat,
    /// +0x8: acceleration grip (`patchaccelgrip`).
    pub accel_grip: PlainFloat,
    /// +0xc: brake grip (`patchbrakegrip`).
    pub brake_grip: PlainFloat,
    /// +0x10, +0x14: how much mud the surface lets the wheels throw. Blended per point into
    /// `surface_blends`; the centre point's go to `mud_scale_a` ([0] -> throws, scaled by
    /// the drive speed; [1] -> splashes, scaled by the slip; `physics::accumulate_mud`). Probable.
    pub mud_rates: [PlainFloat; 2],
    /// +0x18: no code reads or writes it (data only).
    pub unknown_18: [PlainDword; 2],
    /// +0x20: the drag (10.0 on two grasses, `patchstickygrass`).
    pub drag: PlainFloat,
    /// +0x24: the speed over the ground above which the rolling drag applies (the centre
    /// point's blend is `wheel_drag[1]`, compared in `drive_wheels_step::speed_over_ground`).
    pub rolling_drag_min_speed: PlainFloat,
    /// +0x28: the grip blended into `point_grips`.
    pub grip: PlainFloat,
    /// +0x2c: how much a hit on this ground wears the car: blended per point into the car's
    /// `point_wear_factor`; the centre point's multiplies the hit figures of the damage model
    /// (`damage_wear::hit_figures`). Probable.
    pub wear_factor: PlainFloat,
}

/// A force on the car (0x20 bytes): `physics::apply_force` takes it, the step clears its
/// first six dwords (`physics::clear_records`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ForceRecord {
    /// +0x0: the force.
    pub force: [PlainFloat; 3],
    /// +0xc: where it acts.
    pub point: [PlainFloat; 3],
    /// +0x18: not cleared; no code reads or writes it (data only).
    pub unknown_18: [PlainDword; 2],
}

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PhysicsTables {
    /// +0x0 (0x613828): 5.0: a reset is allowed below this speed (`drive::reset_request`).
    pub reset_speed_limit: PlainFloat,
    /// +0x4 (0x61382C): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_4: PlainDword,
    /// +0x8 (0x613830): `g_sceneClock`: the scene's clock, read by everything timed.
    pub scene_clock: PlainDword,
    /// +0xc (0x613834): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_c: PlainDword,
    /// +0x10 (0x613838): `g_frameRateCount`: frames counted by `race::measure_frame_rate`.
    pub frame_rate_count: PlainDword,
    /// +0x14 (0x61383C): the race clock's last reading (`drive::race_clock_tick`).
    pub race_clock_now: PlainDword,
    /// +0x18 (0x613840): the clock's count the race's time is taken from (`drive::race_clock_tick`, the films).
    pub race_clock_base: PlainDword,
    /// +0x1c (0x613844): the race's ticks to run (`drive::race_clock_tick` -> `drive::race_step`).
    pub race_tenths: PlainDword,
    /// +0x20 (0x613848): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_20: [PlainDword; 17],
    /// +0x64 (0x61388C): 0.01: the physics step's length in seconds.
    pub step_time: PlainFloat,
    /// +0x68 (0x613890): the steering input (`drive::read_car_controls`, `drive::steer_inputs`).
    pub input_steer: PlainDword,
    /// +0x6c (0x613894): the throttle input.
    pub input_throttle: PlainDword,
    /// +0x70 (0x613898): the brake input.
    pub input_brake: PlainDword,
    /// +0x74 (0x61389C): the handbrake input (500 at the start line, `physics::gear_input_rules`).
    pub input_handbrake: PlainDword,
    /// +0x78 (0x6138A0): the gear-up key (`physics::manual_gearbox`).
    pub input_gear_up: PlainDword,
    /// +0x7c (0x6138A4): the gear-down key.
    pub input_gear_down: PlainDword,
    /// +0x80 (0x6138A8): the gear asked for, -1 none (`physics::auto_gearbox`).
    pub input_gear_wanted: PlainDword,
    /// +0x84 (0x6138AC): three of the six are printed by the developers' wear page (`debug::draw_wear_bars`, 0x434C61 ..); nothing else uses them.
    pub wear_page_words: [PlainDword; 6],
    /// +0x9c (0x6138C4): the four input words recorded and replayed (`drive::record_inputs`, `drive::play_inputs`).
    pub input_words: [PlainDword; 4],
    /// +0xac (0x6138D4): a counter by camera view (`race::frame`).
    pub view_counters: [PlainDword; 7],
    /// +0xc8 (0x6138F0): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_c8: PlainDword,
    /// +0xcc (0x6138F4): `g_debugPage`, the developers' debug overlay's page (0 .. 7), and
    /// `g_debugFreezeRace` (`drive::race_step` skips the physics ticks while set): what is
    /// left of the debug keys; `input::end_frame` sets both 0 every frame (`debug::overlay`).
    pub debug: [PlainDword; 2],
    /// +0xd4 (0x6138FC): the developers' debug keys' values (probably): at 0x43AB08 each key held copies one of these dwords into a gauge of the first car block (`engine_damage` ..); no port code names them.
    pub debug_key_values: [PlainDword; 31],
    /// +0x150 (0x613978): `off_613978`: the ground kinds' records (`physics::wheel_surfaces`; twelve patched by `physics::apply_patches`).
    pub surfaces: [Surface; 16],
    /// +0x450 (0x613C78): eight tables of 13 dwords the surface records point at: [0] then six figures by tyre kind and six more (`physics::wheel_surfaces`).
    pub surface_tables: [PlainDword; 104],
    /// +0x5f0 (0x613E18): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_5f0: [PlainDword; 10],
    /// +0x618 (0x613E40): 8.0: below it the car may stall (`physics::stall_when_still`).
    pub stall_speed: PlainFloat,
    /// +0x61c (0x613E44): 0.5 (`physics::stall_when_still`).
    pub stall_share: PlainFloat,
    /// +0x620 (0x613E48): 800.0 (`physics::cap_drive_speed`).
    pub drive_speed_cap: PlainFloat,
    /// +0x624 (0x613E4C): 20.0 (`physics::cap_drive_speed`).
    pub drive_speed_cap_a: PlainFloat,
    /// +0x628 (0x613E50): 20.0 (`physics::cap_drive_speed`).
    pub drive_speed_cap_b: PlainFloat,
    /// +0x62c (0x613E54): 2.4 / 1.6 by drive kind 1 or not (`physics::drive_grip_forces`).
    pub grip_front_drive: [PlainFloat; 2],
    /// +0x634 (0x613E5C): 0.5 / 0.4 by drive kind 1 or not (`physics::drive_grip_forces`).
    pub grip_rear_drive: [PlainFloat; 2],
    /// +0x63c (0x613E64): 20.0 (`physics::ease_low_speed_forces`).
    pub low_speed_limit: PlainFloat,
    /// +0x640 (0x613E68): 1.0 (`physics::ease_low_speed_forces`).
    pub low_speed_add: PlainFloat,
    /// +0x644 (0x613E6C): 0.5 (`physics::ease_low_speed_forces`).
    pub low_speed_scale: PlainFloat,
    /// +0x648 (0x613E70): 4.0: the spin held to it (`physics::limit_motion`).
    pub spin_limit: PlainFloat,
    /// +0x64c (0x613E74): 80.0: the motion held to it (`physics::limit_motion`).
    pub velocity_limit: PlainFloat,
    /// +0x650 (0x613E78): 7.5 (`physics::drive_grip_forces`).
    pub grip_slide_scale: PlainFloat,
    /// +0x654 (0x613E7C): 5.0 (`physics::drive_grip_forces`).
    pub grip_slip_limit: PlainFloat,
    /// +0x658 (0x613E80): 10.0, held both ways (`physics::drive_grip_forces`).
    pub grip_force_limit: PlainFloat,
    /// +0x65c (0x613E84): 1.0 (`physics::drive_grip_forces`).
    pub grip_slow_speed: PlainFloat,
    /// +0x660 (0x613E88): 0.9 (`physics::load_car_specs`).
    pub spec_scale_a4: PlainFloat,
    /// +0x664 (0x613E8C): 0.15 (`physics::probe_ground`).
    pub probe_lift: PlainFloat,
    /// +0x668 (0x613E90): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_668: PlainDword,
    /// +0x66c (0x613E94): 1.0 (`physics::auto_gearbox`).
    pub auto_gear_speed_up: PlainFloat,
    /// +0x670 (0x613E98): -1.0 (`physics::auto_gearbox`).
    pub auto_gear_speed_down: PlainFloat,
    /// +0x674 (0x613E9C): 0.95: the engine's speed eased by it (`physics::gearbox_step`).
    pub engine_ease: PlainFloat,
    /// +0x678 (0x613EA0): 2.0 (`physics::hold_spin_on_ground`).
    pub ground_spin_limit: PlainFloat,
    /// +0x67c (0x613EA4): 0.1 (`physics::drive_grip_forces`).
    pub grip_ease_step: PlainFloat,
    /// +0x680 (0x613EA8): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_680: PlainDword,
    /// +0x684 (0x613EAC): 2.0 / 2.2 by drive kind (`physics::drive_grip_forces`).
    pub grip_factor: [PlainFloat; 2],
    /// +0x68c (0x613EB4): 0.5 (`physics::drive_grip_forces`).
    pub side_grip_share: PlainFloat,
    /// +0x690 (0x613EB8): 8500.0 (`physics::load_car_specs`).
    pub rev_limit_scale: PlainFloat,
    /// +0x694 (0x613EBC): 0.65 (`physics::spin_wheels`).
    pub wheel_spin_share: PlainFloat,
    /// +0x698 (0x613EC0): 0.325 (`physics::wheel_springs`).
    pub spring_share: PlainFloat,
    /// +0x69c (0x613EC4): 2.5 (`physics::drive_wheels_step`).
    pub drive_speed_scale: PlainFloat,
    /// +0x6a0 (0x613EC8): 15.0, held both ways (`physics::car_physics_step`).
    pub side_speed_limit: PlainFloat,
    /// +0x6a4 (0x613ECC): 0.05 (`physics::drive_grip_forces`).
    pub side_force_scale: PlainFloat,
    /// +0x6a8 (0x613ED0): 0.5 (`physics::drive_wheels_step`).
    pub slip_blend: PlainFloat,
    /// +0x6ac (0x613ED4): 0.5 (`physics::drive_wheels_step`).
    pub slip_share: PlainFloat,
    /// +0x6b0 (0x613ED8): 1.5 (`physics::drive_wheels_step`).
    pub wheel_speed_margin: PlainFloat,
    /// +0x6b4 (0x613EDC): 0.1 (`physics::drive_wheels_step`).
    pub wheel_speed_gap: PlainFloat,
    /// +0x6b8 (0x613EE0): 1.0 (`physics::car_sound_state`).
    pub sound_speed_scale: PlainFloat,
    /// +0x6bc (0x613EE4): 0.0045 / 0.0025 (`physics::add_drive_forces`).
    pub drive_force_factor: [PlainFloat; 2],
    /// +0x6c4 (0x613EEC): 8 (`physics::drive_torque_share`).
    pub torque_share_a: PlainDword,
    /// +0x6c8 (0x613EF0): 0x20 (`physics::drive_torque_share`).
    pub torque_share_b: PlainDword,
    /// +0x6cc (0x613EF4): 32.0's bits, divided by as an integer (`physics::drive_torque_share`).
    pub torque_share_c: PlainDword,
    /// +0x6d0 (0x613EF8): 0x1000 (`physics::crash_contact`).
    pub crash_depth: PlainDword,
    /// +0x6d4 (0x613EFC): 2.4, copied into 0x614BA0 (`physics::load_car_specs`).
    pub spec_default_ba0: PlainFloat,
    /// +0x6d8 (0x613F00): 0.5, copied into 0x614BA4.
    pub spec_default_ba4: PlainFloat,
    /// +0x6dc (0x613F04): 0x76C, 0x640, 0x384, scaled into 0x614BC0 .. (`physics::load_car_specs`).
    pub spec_default_bc0: [PlainDword; 3],
    /// +0x6e8 (0x613F10): 0.85 (`physics::load_car_specs`).
    pub spec_scale_1b8: PlainFloat,
    /// +0x6ec (0x613F14): 4.0, copied into 0x614BA8.
    pub spec_default_ba8: PlainFloat,
    /// +0x6f0 (0x613F18): 0.95 (`physics::auto_gearbox`).
    pub auto_gear_engine_share: PlainFloat,
    /// +0x6f4 (0x613F1C): `physics::set_handling`'s (lo, hi) pairs for 0x614BF0, 0x614C0C, 0x614C08.
    pub handling_ranges_a: [PlainFloat; 6],
    /// +0x70c (0x613F34): 2.0 (`physics::drive_grip_forces`).
    pub grip_ramp: PlainFloat,
    /// +0x710 (0x613F38): 0.25 (`physics::drive_grip_forces`).
    pub grip_ramp_share: PlainFloat,
    /// +0x714 (0x613F3C): 1.0 (`physics::settle_drive_speeds`).
    pub settle_fast_step: PlainFloat,
    /// +0x718 (0x613F40): 0.01 (`physics::settle_drive_speeds`).
    pub settle_slow_step: PlainFloat,
    /// +0x71c (0x613F44): `physics::set_handling`'s pairs for 0x614BF4 and, by kind, 0x614BFC.
    pub handling_ranges_b: [PlainFloat; 6],
    /// +0x734 (0x613F5C): 1.0, 0.9, 0.85, 1.0 (`physics::wheel_surfaces`).
    pub surface_blend: [PlainFloat; 4],
    /// +0x744 (0x613F6C): `physics::set_handling`'s pairs for 0x614BF8, 0x614C00, 0x614C04, 0x614BEC.
    pub handling_ranges_c: [PlainFloat; 8],
    /// +0x764 (0x613F8C): 0.5 (`physics::probe_ground`, `physics::wheel_springs`).
    pub probe_share: PlainFloat,
    /// +0x768 (0x613F90): 2.0 (`physics::wheel_springs`).
    pub spring_force_scale: PlainFloat,
    /// +0x76c (0x613F94): 0.25 (`physics::engine_torque`).
    pub torque_ease: PlainFloat,
    /// +0x770 (0x613F98): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_770: [PlainDword; 3],
    /// +0x77c (0x613FA4): 5: the steps the drive is cut after a change (`physics::gearbox`).
    pub drive_cut_steps: PlainDword,
    /// +0x780 (0x613FA8): 0.85, 0.75, 2.5: the car's turning inertia by axis (`physics::apply_torque`).
    pub inertia: [PlainFloat; 3],
    /// +0x78c (0x613FB4): 0.0001 (`physics::set_downforce`).
    pub downforce_factor: PlainFloat,
    /// +0x790 (0x613FB8): 3000.0 then one more, by spec (`physics::load_car_specs`, by
    /// drive kind 0 .. 2: kind 2 reads the next field, `camera_drop`; `notes/bugs.md`).
    pub rev_limits: [PlainFloat; 2],
    /// +0x798 (0x613FC0): 0.7 (`physics::update_camera`, the car's shadow).
    pub camera_drop: PlainFloat,
    /// +0x79c (0x613FC4): 0xB333: the car's places dropped by it (`race::start_place`, `scene::add_object_shape`).
    pub ground_drop: PlainDword,
    /// +0x7a0 (0x613FC8): 9.8 (`physics::apply_patches`).
    pub gravity: PlainFloat,
    /// +0x7a4 (0x613FCC): 5.0 (`physics::air_drag`, the test drive).
    pub air_drag: PlainFloat,
    /// +0x7a8 (0x613FD0): 0.75 (the test drive).
    pub rolling_drag: PlainFloat,
    /// +0x7ac (0x613FD4): 5.0 (`physics::free_rev`).
    pub free_rev_add: PlainFloat,
    /// +0x7b0 (0x613FD8): 0.005 (`physics::free_rev`).
    pub free_rev_scale: PlainFloat,
    /// +0x7b4 (0x613FDC): 0.9 (`physics::find_clear_place`).
    pub clear_place_share: PlainFloat,
    /// +0x7b8 (0x613FE0): 0.99 (`physics::pick_start_gear`).
    pub start_gear_share: PlainFloat,
    /// +0x7bc (0x613FE4): 1.2 (`physics::load_car_specs`).
    pub rev_top_scale: PlainFloat,
    /// +0x7c0 (0x613FE8): 0.001 (`physics::make_gear_ratios`).
    pub gear_ratio_step: PlainFloat,
    /// +0x7c4 (0x613FEC): 0.0025 (`physics::wheel_surfaces`).
    pub wheel_load_scale: PlainFloat,
    /// +0x7c8 (0x613FF0): the engine's torque curve read in pairs (`physics::engine_torque`).
    pub torque_curve: [PlainFloat; 9],
    /// +0x7ec (0x614014): `physics::find_clear_place`'s first table (with 0x71A10C).
    pub clear_place_steps_a: [PlainDword; 16],
    /// +0x82c (0x614054): its second table (with 0x71A114).
    pub clear_place_steps_b: [PlainDword; 16],
    /// +0x86c (0x614094): 0.02, held both ways (`physics::camera_frame`).
    pub camera_frame_limit: PlainFloat,
    /// +0x870 (0x614098): 0.05 (`physics::camera_frame`).
    pub camera_frame_scale: PlainFloat,
    /// +0x874 (0x61409C): the motion kept by `physics::keep_motion`.
    pub kept_motion: [PlainDword; 3],
    /// +0x880 (0x6140A8): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_880: PlainDword,
    /// +0x884 (0x6140AC): `physics::stall_when_still`'s.
    pub stall_figure: PlainFloat,
    /// +0x888 (0x6140B0): two shares held to 0 .. 1 and their blend (`physics::drive_wheels_step`).
    pub drive_shares: [PlainFloat; 3],
    /// +0x894 (0x6140BC): `g_gearRatios` from gear -1 (reverse) to 7 (`physics::make_gear_ratios`).
    pub gear_ratios: [PlainFloat; 9],
    /// +0x8b8 (0x6140E0): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_8b8: PlainDword,
    /// +0x8bc (0x6140E4): the two lever pushes (`physics::lever_push`).
    pub lever_pushes: [PlainDword; 2],
    /// +0x8c4 (0x6140EC): the time marks the original's `physics::note_time_mark` / `copy_step_clock_to_mark` kept; read by nothing, no longer written.
    pub time_marks: [PlainDword; 2],
    /// +0x8cc (0x6140F4): `physics::add_drive_forces`'.
    pub drive_force_scale: PlainFloat,
    /// +0x8d0 (0x6140F8): the engine's lowest speed (`physics::load_car_specs`).
    pub idle_engine_speed: PlainFloat,
    /// +0x8d4 (0x6140FC): the idle speed + a step, the drop to neutral below it.
    pub stall_engine_speed: PlainFloat,
    /// +0x8d8 (0x614100): the motion's limit (`physics::limit_motion`).
    pub top_speed: PlainFloat,
    /// +0x8dc (0x614104): the engine's highest speed.
    pub rev_limit: PlainFloat,
    /// +0x8e0 (0x614108): hits of scene points on the car this step.
    pub scene_point_hits: PlainDword,
    /// +0x8e4 (0x61410C): hits of objects on the car this step.
    pub object_hits: PlainDword,
    /// +0x8e8 (0x614110): hits of points on the car this step.
    pub point_hits: PlainDword,
    /// +0x8ec (0x614114): the count-down's last beep (`drive::countdown_beep`).
    pub countdown_sound: PlainDword,
    /// +0x8f0 (0x614118): the face `physics::find_clear_place` logs.
    pub clear_place_face: PlainDword,
    /// +0x8f4 (0x61411C): the two surfaces' scales by tyre kind (`physics::wheel_surfaces`).
    pub surface_scales: [PlainFloat; 2],
    /// +0x8fc (0x614124): the wheels' spring damping (`physics::wheel_springs`).
    pub spring_damping: [PlainFloat; 4],
    /// +0x90c (0x614134): 0 / 1 (`physics::drive_wheels_step`).
    pub wheels_driven: PlainDword,
    /// +0x910 (0x614138): the car's vertical speed kept (`physics::car_physics_step`).
    pub vertical_speed: PlainFloat,
    /// +0x914 (0x61413C): `physics::pick_start_gear`'s: 100, 0, the engine speed it began
    /// with.
    pub start_gear_figures: [PlainFloat; 3],
    /// +0x920 (0x614148): steps since a contact (`physics::car_physics_step`, `physics::apply_contact_torques`).
    pub contact_step: PlainDword,
    /// +0x924 (0x61414C): 10: the contact steps' limit.
    pub contact_steps: PlainDword,
    /// +0x928 (0x614150): 1 for each point on hard ground (`physics::wheel_surfaces`).
    pub hard_ground: [PlainDword; 5],
    /// +0x93c (0x614164): `physics::drive_grip_forces`' integer.
    pub grip_drive_word: PlainDword,
    /// +0x940 (0x614168): (`physics::cap_drive_speed`).
    pub drive_speed_figure: PlainFloat,
    /// +0x944 (0x61416C): the `noautoneutral` setting.
    pub no_auto_neutral: PlainDword,
    /// +0x948 (0x614170): the car's weight factor (`physics::race_timers`, `physics::set_wheel_weights`).
    pub weight: PlainFloat,
    /// +0x94c (0x614174): (`physics::set_downforce`).
    pub downforce: PlainFloat,
    /// +0x950 (0x614178): the wheels' spring forces (`physics::wheel_springs`).
    pub spring_forces: [PlainFloat; 4],
    /// +0x960 (0x614188): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_960: [PlainDword; 9],
    /// +0x984 (0x6141AC): the wall edge's two points' x and z (`physics::wall_edge_contact`).
    pub edge_points: [PlainDword; 4],
    /// +0x994 (0x6141BC): the edge crossed from / to (`physics::crossed_edge_contact`).
    pub crossed_edge: [PlainDword; 2],
    /// +0x99c (0x6141C4): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_99c: PlainDword,
    /// +0x9a0 (0x6141C8): the wall's heading, -1 none.
    pub wall_heading: PlainDword,
    /// +0x9a4 (0x6141CC): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_9a4: PlainDword,
    /// +0x9a8 (0x6141D0): physics steps counted (`physics::car_physics_step`).
    pub step_count: PlainDword,
    /// +0x9ac (0x6141D4): the two axles' pushes (`physics::drive_wheels_step`, `physics::wheel_springs`).
    pub wheel_spin_push: [PlainFloat; 2],
    /// +0x9b4 (0x6141DC): (`physics::feel_ground`).
    pub feel_word: PlainDword,
    /// +0x9b8 (0x6141E0): the car's place saved (`physics::restore_saved_car`).
    pub saved_place: [PlainDword; 3],
    /// +0x9c4 (0x6141EC): the car's matrix saved.
    pub saved_matrix: [PlainFloat; 16],
    /// +0xa04 (0x61422C): (`physics::drive_wheels_step`).
    pub wheel_speed_step: PlainFloat,
    /// +0xa08 (0x614230): (`physics::wheel_surfaces`).
    pub surface_softness: PlainFloat,
    /// +0xa0c (0x614234): |front / rear wheel speed - the car's| (`physics::drive_wheels_step`).
    pub wheel_slips: [PlainFloat; 2],
    /// +0xa14 (0x61423C): (`physics::drive_wheels_step`).
    pub drive_limits: [PlainFloat; 3],
    /// +0xa20 (0x614248): the two axles' drive forces.
    pub drive_forces: [PlainFloat; 2],
    /// +0xa28 (0x614250): the drive's slip and its eased value.
    pub drive_slip: [PlainFloat; 2],
    /// +0xa30 (0x614258): the two axles' slips.
    pub axle_slips: [PlainFloat; 2],
    /// +0xa38 (0x614260): (`physics::drive_grip_forces`).
    pub grip_limit: PlainFloat,
    /// +0xa3c (0x614264): (`physics::drive_wheels_step`).
    pub slip_limit: PlainFloat,
    /// +0xa40 (0x614268): the four wheel loads' sum x `wheel_load_scale`.
    pub wheel_load: PlainFloat,
    /// +0xa44 (0x61426C): the pitch and roll steps of a car settling (`race::car_starts`).
    pub settle_steps: [PlainDword; 2],
    /// +0xa4c (0x614274): (`physics::apply_torque`).
    pub torque_cross: [PlainFloat; 3],
    /// +0xa58 (0x614280): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_a58: PlainDword,
    /// +0xa5c (0x614284): the five points' loads (`physics::wheel_surfaces` writes all
    /// five; `physics::crash_contact` reads the fifth).
    pub wheel_loads: [PlainFloat; 5],
    /// +0xa70 (0x614298): the five points' grips.
    pub wheel_grips: [PlainFloat; 5],
    /// +0xa84 (0x6142AC): the five points' second figures.
    pub wheel_grips_b: [PlainFloat; 5],
    /// +0xa98 (0x6142C0): the five points' grips blended (`physics::wheel_surfaces`).
    pub point_grips: [PlainFloat; 5],
    /// +0xaac (0x6142D4): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_aac: [PlainDword; 251],
    /// +0xe98 (0x6146C0): the surface records' +0x20, +0x24, +0x14, +0x10 blended for the five points.
    pub surface_blends: [PlainFloat; 20],
    /// +0xee8 (0x614710): the first surface of each point (`physics::wheel_surfaces`).
    pub surface_firsts: [PlainDword; 5],
    /// +0xefc (0x614724): the second surface of each point.
    pub surface_seconds: [PlainDword; 5],
    /// +0xf10 (0x614738): `physics::find_clear_place`'s count, 0 when clear.
    pub clear_place_count: PlainDword,
    /// +0xf14 (0x61473C): its tries.
    pub clear_place_tries: PlainDword,
    /// +0xf18 (0x614740): (`physics::drive_force`, `physics::free_rev`).
    pub drive_ratio: PlainFloat,
    /// +0xf1c (0x614744): (`physics::engine_torque`).
    pub engine_torque: PlainFloat,
    /// +0xf20 (0x614748): the engine's torque, 0 when cut.
    pub engine_force: PlainFloat,
    /// +0xf24 (0x61474C): (the test drive).
    pub test_drive_figure: PlainFloat,
    /// +0xf28 (0x614750): (`physics::drive_force`).
    pub drive_torque: PlainFloat,
    /// +0xf2c (0x614754): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_f2c: PlainDword,
    /// +0xf30 (0x614758): (`physics::drive_torque_share`).
    pub torque_share: PlainFloat,
    /// +0xf34 (0x61475C): (`physics::ease_low_speed_forces`).
    pub low_speed_share: PlainFloat,
    /// +0xf38 (0x614760): (`physics::drive_wheels_step`).
    pub wheel_drag: [PlainFloat; 2],
    /// +0xf40 (0x614768): held to 0 .. 1 (`physics::drive_grip_forces`, `physics::lever_push`).
    pub lever_share: PlainFloat,
    /// +0xf44 (0x61476C): the spec's +0x60 (`physics::load_car_specs`).
    pub downforce_spec: PlainDword,
    /// +0xf48 (0x614770): `g_forceRecords`: force records (0x20: a force, a point): 0..3 the wheels' weights, then springs, drive and grip (`physics::clear_forces`).
    pub forces: [ForceRecord; 23],
    /// +0x1228 (0x614A50): `g_contactRecords`: the contacts' force records (`physics::clear_contacts`).
    pub contacts: [ForceRecord; 7],
    /// +0x1308 (0x614B30): the push `physics::apply_force` takes.
    pub push_record: [PlainDword; 4],
    /// +0x1318 (0x614B40): `physics::contact_force`'s record. Not used: the callers pass the
    /// record to it.
    pub contact_record: PlainDword,
    /// +0x131c (0x614B44): `physics::lever_push`'s vector.
    pub lever_push_vector: [PlainDword; 4],
    /// +0x132c (0x614B54): the push turned by the car's matrix.
    pub lever_push_turned: [PlainDword; 4],
    /// +0x133c (0x614B64): the place turned by the inverse.
    pub lever_push_place: [PlainDword; 4],
    /// +0x134c (0x614B74): (`physics::drive_grip_forces`).
    pub grip_scale: PlainFloat,
    /// +0x1350 (0x614B78): (`physics::drive_grip_forces`).
    pub side_grip_scale: PlainFloat,
    /// +0x1354 (0x614B7C): the spec's +0x7C .. (`physics::load_car_specs`, `physics::engine_torque`).
    pub spec_torque: [PlainDword; 9],
    /// +0x1378 (0x614BA0): (`physics::drive_grip_forces`; from 0x613EFC).
    pub spec_grip: PlainFloat,
    /// +0x137c (0x614BA4): from 0x613F00.
    pub spec_grip_b: PlainFloat,
    /// +0x1380 (0x614BA8): from 0x613F14 (`physics::drive_wheels_step`).
    pub spec_wheel: PlainFloat,
    /// +0x1384 (0x614BAC): (`physics::drive_wheels_step`).
    pub spec_drive: [PlainFloat; 2],
    /// +0x138c (0x614BB4): the spec's +0x2C .. +0x34 (`physics::wheel_surfaces`).
    pub spec_surface: [PlainFloat; 3],
    /// +0x1398 (0x614BC0): the spec's +0x40 .. scaled (`physics::load_car_specs`).
    pub spec_ints: [PlainDword; 3],
    /// +0x13a4 (0x614BCC): the motion and spin kept (`physics::keep_motion_vectors`).
    pub kept_vectors: [PlainDword; 8],
    /// +0x13c4 (0x614BEC): `physics::set_handling`'s figures (0x614BEC .. 0x614C0C) by the set-up.
    pub handling: [PlainFloat; 9],
    /// +0x13e8 (0x614C10): `g_gearCount`: forward gears.
    pub gear_count: PlainDword,
    /// +0x13ec (0x614C14): the spec's +0x20 (`physics::load_car_specs`).
    pub unused_spec_20: PlainDword,
    /// +0x13f0 (0x614C18): the spec's +0xA0.
    pub ride_height_handling_scale: PlainDword,
    /// +0x13f4 (0x614C1C): (`physics::drive_wheels_step`).
    pub axle_slip_limits: [PlainFloat; 2],
    /// +0x13fc (0x614C24): (`physics::hold_spin_on_ground`).
    pub ground_spin: PlainFloat,
    /// +0x1400 (0x614C28): (`physics::find_clear_place`).
    pub clear_place_mark: PlainDword,
    /// +0x1404 (0x614C2C): `physics::sum_forces`' five sums.
    pub force_sums: [PlainFloat; 5],
    /// +0x1418 (0x614C40): the gear asked for (`physics::take_asked_gear`).
    pub gear_asked: PlainDword,
    /// +0x141c (0x614C44): (`physics::lever_pushes`).
    pub lever_share_start: PlainFloat,
    /// +0x1420 (0x614C48): (`physics::lever_pushes`).
    pub lever_share_left: PlainFloat,
    /// +0x1424 (0x614C4C): (`physics::reset_car`).
    pub reset_offset: PlainDword,
    /// +0x1428 (0x614C50): the checkpoint tried (`physics::reset_point_blocked`).
    pub reset_candidate: PlainDword,
    /// +0x142c (0x614C54): `g_noiseState`: the noise's state (`scene::next_noise`).
    pub noise_state: PlainDword,
    /// +0x1430 (0x614C58): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_1430: [PlainDword; 3],
    /// +0x143c (0x614C64): (`physics::blend_wheel_load`).
    pub wheel_load_limit: PlainFloat,
    /// +0x1440 (0x614C68): `g_nearestCheckpointDistance`.
    pub nearest_checkpoint_distance: PlainFloat,
    /// +0x1444 (0x614C6C): (`physics::off_line_warnings`).
    pub off_line_distance: PlainFloat,
    /// +0x1448 (0x614C70): (`physics::passed_split_point`).
    pub split_distance: PlainFloat,
    /// +0x144c (0x614C74): (`physics::off_line_warnings`).
    pub off_line_word: PlainDword,
    /// +0x1450 (0x614C78): (`physics::pace_notes_step`).
    pub pace_scale: PlainFloat,
    /// +0x1454 (0x614C7C): the events listed (`physics::queue_listed_event`).
    pub listed_events: [PlainDword; 32],
    /// +0x14d4 (0x614CFC): (`physics::passed_split_point`).
    pub split_word: PlainDword,
    /// +0x14d8 (0x614D00): (`physics::passed_split_point`).
    pub split_heading: PlainDword,
    /// +0x14dc (0x614D04): the distance to the split point.
    pub split_point_distance: PlainFloat,
    /// +0x14e0 (0x614D08): the original kept here the checkpoint's near and far colours and the distance (`race::frame_draw`); no code reads or writes it (the field's name nowhere in Rust), it only keeps the image's layout.
    pub unused_14e0: [PlainDword; 3],
    /// +0x14ec (0x614D14): `g_paceSlotOffsets`: each slot's byte in the face.
    pub pace_slot_offsets: [PlainDword; 8],
    /// +0x150c (0x614D34): `g_paceSlot`: the slot 0 .. 7.
    pub pace_slot: PlainDword,
}

const _: () = assert!(core::mem::size_of::<PhysicsTables>() == 0x1510);
const _: () = assert!(core::mem::size_of::<Surface>() == 0x30);
const _: () = assert!(core::mem::size_of::<ForceRecord>() == 0x20);

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl PhysicsTables {
    /// The piece, with the image's first values.
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

impl PhysicsTables {
    /// IDA's address of the first surface table (`g_surfaceTables`) and its length.
    const TABLES: usize = 0x613C78;
    const TABLES_LEN: usize = 104 * 4;

    /// Where ground kind `n`'s table starts in `surface_tables`, as a dword's number (the
    /// records keep IDA's addresses as names). A kind past the 16, or a name off the tables (or between two dwords), is
    /// refused.
    pub fn surface_table_index(&self, n: u32) -> Result<usize, crate::errors::BadInput> {
        let record = self
            .surfaces
            .get(n as usize)
            .ok_or(crate::errors::BadInput("a ground kind past the 16"))?;
        let at = (record.table.get() as usize).wrapping_sub(Self::TABLES);
        if at >= Self::TABLES_LEN || !at.is_multiple_of(4) {
            return Err(crate::errors::BadInput(
                "a ground record's table off the tables",
            ));
        }
        Ok(at / 4)
    }
}

/// IDA 0x613828, the first byte of the piece.
pub const START: usize = 0x613828;
/// IDA 0x614D38, the end of the piece.
pub const END: usize = 0x614D38;

const _: () = assert!(END - START == core::mem::size_of::<PhysicsTables>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn tables_addr() -> usize {
    core::ptr::from_ref(race().shared.physics_tables.get()).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn fields_are_at_the_games_offsets() {
        let race = race();
        assert_eq!(tables_addr() % 4, 0);
        assert_eq!(offset_of!(PhysicsTables, scene_clock), 0x8);
        assert_eq!(offset_of!(PhysicsTables, surfaces), 0x150);
        assert_eq!(offset_of!(PhysicsTables, gravity), 0x7a0);
        assert_eq!(offset_of!(PhysicsTables, gear_ratios), 0x894);
        assert_eq!(offset_of!(PhysicsTables, forces), 0xf48);
        assert_eq!(offset_of!(PhysicsTables, contacts), 0x1228);
        assert_eq!(offset_of!(PhysicsTables, gear_count), 0x13e8);
        assert_eq!(offset_of!(PhysicsTables, pace_slot), 0x150c);
        assert_eq!(race.shared.physics_tables.get().gravity.get(), 9.8);
        assert_eq!(
            race.shared.physics_tables.get().surfaces[1]
                .slide_grip
                .get(),
            1.0
        );
    }

    #[test]
    fn the_ground_records_name_their_tables_as_the_image_does() {
        let shared = &race().shared;
        for n in 0..16u32 {
            let image = crate::test_support::first_values::dword(
                "race.owner.RACE.shared.physics_tables.surfaces[0].table/768",
                0xC * n as usize,
            );
            // Kept as the image's name; resolved to the same table's place.
            assert_eq!(
                shared.physics_tables.get().surfaces[n as usize].table.get(),
                image
            );
            let slot = (image as usize - PhysicsTables::TABLES) / 4;
            assert_eq!(
                shared.physics_tables.get().surface_table_index(n).unwrap(),
                slot
            );
            assert!(slot.is_multiple_of(13) && slot < 104);
        }
        assert!(shared.physics_tables.get().surface_table_index(16).is_err());
    }
}
