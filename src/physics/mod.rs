//! The cars' physics.

use crate::framework::GameHook;

pub mod air_drag;
pub mod apply_contact_forces;
pub mod apply_force;
pub mod apply_forces;
pub mod apply_patches;
pub mod apply_wheel_forces;
pub mod axis_turn_matrix;
pub mod blend_wheel_load;
pub mod clear_contacts;
pub mod clear_forces;
pub mod clear_records;
pub mod copy_hooks_test;
pub mod count_down_notice;
pub mod count_down_timers;
pub mod dot_product;
pub mod drop_to_neutral;
pub mod ease_car_value;
pub mod edge_side;
pub mod face_shares;
pub mod face_under_point;
pub mod feel_ground;
pub mod find_course_record;
pub mod find_ground;
pub mod flag_result;
pub mod gear_bit;
pub mod gear_input_rules;
pub mod gearing;
pub mod grade_or_reset;
pub mod ground;
pub mod ground_search;
pub mod higher_free_gear;
pub mod hold_spin_on_ground;
pub mod keep_motion;
pub mod keep_motion_vectors;
pub mod lower_free_gear;
pub mod lowest_body_point;
pub mod make_gear_ratios;
pub mod mirror;
pub mod move_car;
pub mod move_slot;
pub mod move_slots;
pub mod note_surface_mode;
pub mod note_time_mark;
pub mod plane_height;
pub mod probe_and_keep;
pub mod probe_ground;
pub mod queue_notice;
pub mod rate_current_car;
pub mod rate_damage;
pub mod reset_car_grades;
pub mod reset_current_car;
pub mod reset_point_blocked;
pub mod restore_saved_car;
pub mod revs_from_speed;
pub mod revs_from_wheels;
pub mod save_car_state;
pub mod scale_vector;
pub mod search_cell;
pub mod search_record_faces;
pub mod set_downforce;
pub mod set_handling;
pub mod set_wheel_weights;
pub mod share_engine_sound;
pub mod spin_wheels;
pub mod stall_when_still;
pub mod start_skid_mark;
pub mod stir_car_word;
pub mod store_motion_length;
pub mod sum_forces;
pub mod take_notice;
pub mod test_car_word;
pub mod try_last_face;
pub mod turn_by_car_matrix;
pub mod turn_by_inverse;
pub mod value_notice;
pub mod vector_length;
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
];
