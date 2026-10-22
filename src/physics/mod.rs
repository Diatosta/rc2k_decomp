//! The cars' physics.

use crate::framework::GameHook;

pub mod add_drive_forces;
pub mod air_drag;
pub mod allowed_gear;
pub mod apply_contact_forces;
pub mod apply_contact_torques;
pub mod apply_force;
pub mod apply_forces;
pub mod apply_patches;
pub mod apply_torque;
pub mod apply_torques;
pub mod apply_wheel_forces;
pub mod apply_wheel_torques;
pub mod auto_gearbox;
pub mod axis_turn_matrix;
pub mod blend_wheel_load;
pub mod bump_cars;
pub mod camera_avoid_ground;
pub mod camera_clear_of_ground;
pub mod camera_frame;
pub mod camera_ground;
pub mod camera_height_bias;
pub mod camera_low_switch;
pub mod camera_over_ground;
pub mod camera_view;
pub mod cap_drive_speed;
pub mod cap_reverse_inputs;
pub mod car_angles;
pub mod car_collisions;
pub mod car_frame_motion;
pub mod car_lighting;
pub mod car_physics_step;
pub mod car_sound_state;
pub mod car_sounds;
pub mod cell_face_contacts;
pub mod cells;
pub mod change_gear;
pub mod checkpoint_offset;
pub mod clamped_asin;
pub mod clear_contacts;
pub mod clear_forces;
pub mod clear_records;
pub mod compare_split_times;
pub mod contact_force;
pub mod copy_car_word_8;
pub mod copy_hooks_test;
pub mod corner_hits_car;
pub mod count_down_notice;
pub mod count_down_start;
pub mod count_down_timers;
pub mod count_ground_probe;
pub mod count_start_second;
pub mod count_stop_timers;
pub mod crash_contact;
pub mod crossed_edge;
pub mod crossed_edge_contact;
pub mod cue_car_sound;
pub mod cue_car_sound_last;
pub mod cue_listed_car_sound;
pub mod cue_listed_car_sound_last;
pub mod damage_factors;
pub mod damage_gauges;
pub mod damage_model;
pub mod damage_notices;
pub mod damage_step;
pub mod damage_wear;
pub mod dot_product;
pub mod drive_force;
pub mod drive_grip_forces;
pub mod drive_torque_share;
pub mod drive_wheels_step;
pub mod drop_broken_parts;
pub mod drop_hit_parts;
pub mod drop_to_neutral;
pub mod ease_car_value;
pub mod ease_low_speed_forces;
pub mod ease_pitch_down;
pub mod ease_pitch_up;
pub mod ease_view_angles;
pub mod ease_view_blend;
pub mod ease_view_turn;
pub mod edge_side;
pub mod engine_sound;
pub mod engine_torque;
pub mod engine_torque_or_cut;
pub mod expire_car_events;
pub mod face_contact;
pub mod face_contacts;
pub mod face_contacts_around;
pub mod face_has_edge;
pub mod face_shares;
pub mod face_under_point;
pub mod false_start;
pub mod feel_ground;
pub mod fill_race_slot;
pub mod fill_rival_table;
pub mod fill_slot_on_bit;
pub mod find_car_checkpoint;
pub mod find_clear_place;
pub mod find_course_record;
pub mod find_ground;
pub mod flag_result;
pub mod flip_cell_index;
pub mod flow;
pub mod free_rev;
pub mod free_rev_clutch;
pub mod free_rev_in_neutral;
pub mod gear_bit;
pub mod gear_change_sound;
pub mod gear_input_rules;
pub mod gearbox;
pub mod gearbox_step;
pub mod gearing;
pub mod grade_or_reset;
pub mod ground;
pub mod ground_contact;
pub mod ground_search;
pub mod higher_free_gear;
pub mod hit_part_checks;
pub mod hold_spin_on_ground;
pub mod keep_motion;
pub mod keep_motion_vectors;
pub mod keep_split_time;
pub mod lever_push;
pub mod lever_pushes;
pub mod limit_motion;
pub mod listed_car_flags;
pub mod load_car_specs;
pub mod lower_free_gear;
pub mod lowest_body_point;
pub mod make_gear_ratios;
pub mod manual_gearbox;
pub mod manual_gearbox_call;
pub mod mark_car_started;
pub mod matrix_angles;
pub mod mirror;
pub mod move_car;
pub mod move_slot;
pub mod move_slots;
pub mod multiply_parts;
pub mod note_surface_mode;
pub mod note_time_mark;
pub mod object_hits_car;
pub mod object_turn_matrix;
pub mod off_line_warnings;
pub mod pace_notes_step;
pub mod passed_split_point;
pub mod pick_start_gear;
pub mod place_camera;
pub mod place_car_sound;
pub mod place_sound_e1;
pub mod place_sound_e1_other;
pub mod place_sound_e5;
pub mod place_sound_object;
pub mod plane_height;
pub mod point_hits_car;
pub mod probe_and_keep;
pub mod probe_ground;
pub mod put_back_motion_vectors;
pub mod queue_car_event;
pub mod queue_listed_event;
pub mod queue_notice;
pub mod queue_pace_note;
pub mod queue_sound_cue;
pub mod race_timers;
pub mod rank_split_time;
pub mod rate_current_car;
pub mod rate_damage;
pub mod record_face_contacts;
pub mod reset_car;
pub mod reset_car_grades;
pub mod reset_car_on_line;
pub mod reset_current_car;
pub mod reset_parts;
pub mod reset_point_blocked;
pub mod reset_when_stuck;
pub mod restore_saved_car;
pub mod revs_from_speed;
pub mod revs_from_wheels;
pub mod save_car_state;
pub mod scale_vector;
pub mod scene_point_hits_car;
pub mod search_cell;
pub mod search_record_faces;
pub mod set_downforce;
pub mod set_flag_51659c;
pub mod set_flag_5165a0;
pub mod set_flag_5165a4;
pub mod set_handling;
pub mod set_wheel_weights;
pub mod settle_drive_speeds;
pub mod share_engine_sound;
pub mod sound_band_1;
pub mod sound_band_2;
pub mod sound_band_3;
pub mod sound_band_4;
pub mod sound_bands;
pub mod sound_cue_e5;
pub mod sound_cues;
pub mod sound_hit_parts;
pub mod sound_on_bit;
pub mod spin_matrix;
pub mod spin_wheels;
pub mod split_by_mode;
pub mod splits;
pub mod stall_when_still;
pub mod start_go_notice;
pub mod start_skid_mark;
pub mod stir_car_word;
pub mod store_motion_length;
pub mod sum_forces;
pub mod surface_noise;
pub mod take_asked_gear;
pub mod take_notice;
pub mod take_split;
pub mod take_split_counted;
pub mod take_view_a;
pub mod take_view_b;
pub mod test_car_word;
pub mod test_car_word_234;
pub mod test_drive;
pub mod test_drive_run;
pub mod touch_face;
pub mod track_checkpoints;
pub mod try_last_face;
pub mod turn_by_car_matrix;
pub mod turn_by_heading;
pub mod turn_by_inverse;
pub mod turn_by_inverse_at;
pub mod turn_by_matrix_at;
pub mod turn_wheel_angles;
pub mod update_camera;
pub mod update_car_angles;
pub mod value_notice;
pub mod vector_length;
pub mod wall_contacts;
pub mod wall_edge_contact;
pub mod wall_push_back;
pub mod wheel_springs;
pub mod wheel_surfaces;
pub mod wheels_from_revs;

pub static HOOKS: &[&dyn GameHook] = &[
    &apply_patches::HOOK,
    &vector_length::HOOK,
    &scale_vector::HOOK,
    &revs_from_wheels::HOOK,
    &wheels_from_revs::HOOK,
    &revs_from_speed::HOOK,
    &dot_product::HOOK,
    &gear_bit::HOOK,
    &test_car_word::HOOK,
    &turn_by_car_matrix::HOOK,
    &turn_by_inverse::HOOK,
    &restore_saved_car::HOOK,
    &queue_notice::HOOK,
    &store_motion_length::HOOK,
    &lower_free_gear::HOOK,
    &higher_free_gear::HOOK,
    &reset_point_blocked::HOOK,
    &take_notice::HOOK,
    &drop_to_neutral::HOOK,
    &plane_height::HOOK,
    &edge_side::HOOK,
    &find_course_record::HOOK,
    &face_shares::HOOK,
    &face_under_point::HOOK,
    &search_record_faces::HOOK,
    &search_cell::HOOK,
    &try_last_face::HOOK,
    &ground_search::HOOK,
    &find_ground::HOOK,
    &probe_ground::HOOK,
    &stir_car_word::HOOK,
    &probe_and_keep::HOOK,
    &ease_car_value::HOOK,
    &save_car_state::HOOK,
    &count_down_notice::HOOK,
    &set_downforce::HOOK,
    &reset_car_grades::HOOK,
    &reset_current_car::HOOK,
    &keep_motion::HOOK,
    &keep_motion_vectors::HOOK,
    &note_time_mark::HOOK,
    &value_notice::HOOK,
    &rate_damage::HOOK,
    &rate_current_car::HOOK,
    &grade_or_reset::HOOK,
    &set_handling::HOOK,
    &gear_input_rules::HOOK,
    &axis_turn_matrix::HOOK,
    &move_slot::HOOK,
    &move_slots::HOOK,
    &clear_records::HOOK,
    &clear_forces::HOOK,
    &clear_contacts::HOOK,
    &spin_wheels::HOOK,
    &move_car::HOOK,
    &share_engine_sound::HOOK,
    &start_skid_mark::HOOK,
    &stall_when_still::HOOK,
    &sum_forces::HOOK,
    &air_drag::HOOK,
    &blend_wheel_load::HOOK,
    &count_down_timers::HOOK,
    &make_gear_ratios::HOOK,
    &set_wheel_weights::HOOK,
    &hold_spin_on_ground::HOOK,
    &note_surface_mode::HOOK,
    &lowest_body_point::HOOK,
    &feel_ground::HOOK,
    &apply_force::HOOK,
    &apply_forces::HOOK,
    &apply_wheel_forces::HOOK,
    &apply_contact_forces::HOOK,
    &turn_by_inverse_at::HOOK,
    &turn_by_matrix_at::HOOK,
    &apply_torque::HOOK,
    &apply_torques::HOOK,
    &apply_wheel_torques::HOOK,
    &apply_contact_torques::HOOK,
    &car_frame_motion::HOOK,
    &clamped_asin::HOOK,
    &matrix_angles::HOOK,
    &car_angles::HOOK,
    &update_car_angles::HOOK,
    &put_back_motion_vectors::HOOK,
    &settle_drive_speeds::HOOK,
    &false_start::HOOK,
    &reset_car_on_line::HOOK,
    &reset_car::HOOK,
    &reset_when_stuck::HOOK,
    &spin_matrix::HOOK,
    &lever_push::HOOK,
    &lever_pushes::HOOK,
    &turn_wheel_angles::HOOK,
    &wheel_springs::HOOK,
    &set_flag_51659c::HOOK,
    &set_flag_5165a0::HOOK,
    &set_flag_5165a4::HOOK,
    &multiply_parts::HOOK,
    &copy_car_word_8::HOOK,
    &checkpoint_offset::HOOK,
    &test_car_word_234::HOOK,
    &face_has_edge::HOOK,
    &flip_cell_index::HOOK,
    &queue_car_event::HOOK,
    &gear_change_sound::HOOK,
    &ease_pitch_up::HOOK,
    &ease_pitch_down::HOOK,
    &mark_car_started::HOOK,
    &count_start_second::HOOK,
    &turn_by_heading::HOOK,
    &queue_listed_event::HOOK,
    &keep_split_time::HOOK,
    &sound_cue_e5::HOOK,
    &count_down_start::HOOK,
    &start_go_notice::HOOK,
    &cap_reverse_inputs::HOOK,
    &ease_low_speed_forces::HOOK,
    &add_drive_forces::HOOK,
    &allowed_gear::HOOK,
    &sound_band_1::HOOK,
    &sound_band_2::HOOK,
    &sound_band_3::HOOK,
    &sound_band_4::HOOK,
    &find_car_checkpoint::HOOK,
    &count_ground_probe::HOOK,
    &cap_drive_speed::HOOK,
    &place_car_sound::HOOK,
    &expire_car_events::HOOK,
    &change_gear::HOOK,
    &drive_torque_share::HOOK,
    &count_stop_timers::HOOK,
    &place_sound_e1::HOOK,
    &place_sound_e5::HOOK,
    &place_sound_e1_other::HOOK,
    &place_sound_object::HOOK,
    &sound_on_bit::HOOK,
    &take_asked_gear::HOOK,
    &pick_start_gear::HOOK,
    &manual_gearbox::HOOK,
    &auto_gearbox::HOOK,
    &manual_gearbox_call::HOOK,
    &gearbox_step::HOOK,
    &fill_rival_table::HOOK,
    &compare_split_times::HOOK,
    &rank_split_time::HOOK,
    &split_by_mode::HOOK,
    &take_split::HOOK,
    &take_split_counted::HOOK,
    &race_timers::HOOK,
    &fill_race_slot::HOOK,
    &fill_slot_on_bit::HOOK,
    &drop_broken_parts::HOOK,
    &engine_torque::HOOK,
    &engine_torque_or_cut::HOOK,
    &drive_force::HOOK,
    &object_turn_matrix::HOOK,
    &free_rev::HOOK,
    &free_rev_in_neutral::HOOK,
    &free_rev_clutch::HOOK,
    &surface_noise::HOOK,
    &car_sound_state::HOOK,
    &engine_sound::HOOK,
    &queue_sound_cue::HOOK,
    &cue_car_sound::HOOK,
    &cue_listed_car_sound::HOOK,
    &off_line_warnings::HOOK,
    &drop_hit_parts::HOOK,
    &sound_hit_parts::HOOK,
    &hit_part_checks::HOOK,
    &passed_split_point::HOOK,
    &track_checkpoints::HOOK,
    &limit_motion::HOOK,
    &load_car_specs::HOOK,
    &car_lighting::HOOK,
    &wheel_surfaces::HOOK,
    &drive_grip_forces::HOOK,
    &drive_wheels_step::HOOK,
    &contact_force::HOOK,
    &point_hits_car::HOOK,
    &scene_point_hits_car::HOOK,
    &object_hits_car::HOOK,
    &touch_face::HOOK,
    &crash_contact::HOOK,
    &ground_contact::HOOK,
    &face_contact::HOOK,
    &record_face_contacts::HOOK,
    &cell_face_contacts::HOOK,
    &face_contacts_around::HOOK,
    &face_contacts::HOOK,
    &bump_cars::HOOK,
    &corner_hits_car::HOOK,
    &car_collisions::HOOK,
    &wall_push_back::HOOK,
    &wall_edge_contact::HOOK,
    &wall_contacts::HOOK,
    &crossed_edge_contact::HOOK,
    &crossed_edge::HOOK,
    &find_clear_place::HOOK,
    &camera_low_switch::HOOK,
    &camera_over_ground::HOOK,
    &camera_clear_of_ground::HOOK,
    &take_view_a::HOOK,
    &take_view_b::HOOK,
    &ease_view_angles::HOOK,
    &ease_view_blend::HOOK,
    &camera_height_bias::HOOK,
    &ease_view_turn::HOOK,
    &place_camera::HOOK,
    &camera_avoid_ground::HOOK,
    &update_camera::HOOK,
    &camera_frame::HOOK,
    &damage_notices::HOOK,
    &damage_factors::HOOK,
    &damage_wear::HOOK,
    &damage_gauges::HOOK,
    &damage_step::HOOK,
    &car_physics_step::HOOK,
    &test_drive_run::HOOK,
    &cue_car_sound_last::HOOK,
    &cue_listed_car_sound_last::HOOK,
    &queue_pace_note::HOOK,
    &pace_notes_step::HOOK,
];
