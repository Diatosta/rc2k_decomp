//! The race (part P): the stage's loading (`race::load_stage`, 0x444040) and, later, the
//! race's run, ported bottom-up.

use crate::framework::GameHook;

pub mod add_surface_share;
pub mod allocate_course_memory;
pub mod best_line;
pub mod build_best_line;
pub mod build_texture_infos;
pub mod car_file_number;
pub mod car_kind_10;
pub mod car_kind_11;
pub mod checkpoint_angles;
pub mod checkpoint_distances;
pub mod clear_lit_faces;
pub mod clear_mud_state;
pub mod clear_prefs_qword;
pub mod copy_course_header;
pub mod copy_lights_data;
pub mod copy_texture_info_tail;
pub mod count_checks;
pub mod course_data;
pub mod create_controls;
pub mod face_normal;
pub mod free_game_map;
pub mod game_map;
pub mod game_music;
pub mod game_music_8;
pub mod get_mat_crash;
pub mod group_records_by_kind;
pub mod init_checkpoints;
pub mod init_mud;
pub mod init_overlay;
pub mod init_skidmarks;
pub mod lay_out_course;
pub mod level_init;
pub mod load_error_list;
pub mod load_game_map;
pub mod load_level;
pub mod load_lights_data;
pub mod load_mud;
pub mod load_packed_game_map;
pub mod load_prefs;
pub mod load_texture_info;
pub mod locate_current_course;
pub mod map_checkpoints;
pub mod map_segment;
pub mod mud;
pub mod mud_pixel;
pub mod music;
pub mod next_map_anchor;
pub mod normalise_work_vector;
pub mod norms;
pub mod objects;
pub mod reload_objects;
pub mod render3d_init;
pub mod replay_cameras_1;
pub mod replay_cameras_2;
pub mod reset_face_lights;
pub mod reset_refresh;
pub mod reset_skid_slots;
pub mod sample_mud_colours;
pub mod set_all_plates;
pub mod set_cdat_number;
pub mod set_course_number;
pub mod set_line_face_kind;
pub mod set_object_kinds;
pub mod set_up_car_points;
pub mod sort_checkpoints;
pub mod sort_points_by_x_down;
pub mod sort_points_by_x_down_b;
pub mod sort_points_by_y;
pub mod sort_points_by_z;
pub mod sounds;
pub mod stage_prefs;
pub mod start_race_sounds;
pub mod store_face_normal;
pub mod store_norms;
pub mod surface_shares;
pub mod swap_stage_texture;
pub mod tweak_stage;
pub mod wake_car_sounds;

pub static HOOKS: &[&dyn GameHook] = &[
    &create_controls::HOOK,
    &game_music::HOOK,
    &game_music_8::HOOK,
    &start_race_sounds::HOOK,
    &wake_car_sounds::HOOK,
    &allocate_course_memory::HOOK,
    &copy_texture_info_tail::HOOK,
    &load_texture_info::HOOK,
    &set_course_number::HOOK,
    &load_error_list::HOOK,
    &build_texture_infos::HOOK,
    &reset_refresh::HOOK,
    &render3d_init::HOOK,
    &init_checkpoints::HOOK,
    &init_overlay::HOOK,
    &set_cdat_number::HOOK,
    &copy_course_header::HOOK,
    &clear_lit_faces::HOOK,
    &reset_face_lights::HOOK,
    &group_records_by_kind::HOOK,
    &load_level::HOOK,
    &set_line_face_kind::HOOK,
    &add_surface_share::HOOK,
    &next_map_anchor::HOOK,
    &count_checks::HOOK,
    &sort_checkpoints::HOOK,
    &checkpoint_angles::HOOK,
    &checkpoint_distances::HOOK,
    &replay_cameras_1::HOOK,
    &replay_cameras_2::HOOK,
    &surface_shares::HOOK,
    &map_segment::HOOK,
    &map_checkpoints::HOOK,
    &build_best_line::HOOK,
    &locate_current_course::HOOK,
    &set_all_plates::HOOK,
    &free_game_map::HOOK,
    &load_packed_game_map::HOOK,
    &load_game_map::HOOK,
    &swap_stage_texture::HOOK,
    &tweak_stage::HOOK,
    &clear_prefs_qword::HOOK,
    &load_prefs::HOOK,
    &reset_skid_slots::HOOK,
    &init_skidmarks::HOOK,
    &copy_lights_data::HOOK,
    &load_lights_data::HOOK,
    &get_mat_crash::HOOK,
    &normalise_work_vector::HOOK,
    &face_normal::HOOK,
    &store_face_normal::HOOK,
    &lay_out_course::HOOK,
    &store_norms::HOOK,
    &car_file_number::HOOK,
    &mud_pixel::HOOK,
    &load_mud::HOOK,
    &clear_mud_state::HOOK,
    &sample_mud_colours::HOOK,
    &init_mud::HOOK,
    &car_kind_10::HOOK,
    &car_kind_11::HOOK,
    &sort_points_by_y::HOOK,
    &sort_points_by_z::HOOK,
    &sort_points_by_x_down::HOOK,
    &sort_points_by_x_down_b::HOOK,
    &set_up_car_points::HOOK,
    &set_object_kinds::HOOK,
    &reload_objects::HOOK,
];
