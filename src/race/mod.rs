//! The race (part P): the stage's loading (`race::load_stage`, 0x444040) and, later, the
//! race's run, ported bottom-up.

use crate::framework::GameHook;

pub mod add_replay_frame;
pub mod add_sort_entry;
pub mod add_surface_share;
pub mod allocate_course_memory;
pub mod auto_set_up_car;
pub mod best_line;
pub mod both_countdown_lights;
pub mod both_split_colours;
pub mod build_best_line;
pub mod build_texture_infos;
pub mod car_file_number;
pub mod car_grades;
pub mod car_kind_10;
pub mod car_kind_11;
pub mod car_label;
pub mod car_starts;
pub mod checkpoint_angles;
pub mod checkpoint_colours;
pub mod checkpoint_distances;
pub mod choose_arcade_opponents;
pub mod clear_lit_faces;
pub mod clear_mud_state;
pub mod clear_player_absences;
pub mod clear_point_records;
pub mod clear_prefs_qword;
pub mod clear_race_slots;
pub mod clear_race_sound_objects;
pub mod clear_race_tables;
pub mod close_replay;
pub mod copy_course_header;
pub mod copy_league_table;
pub mod copy_lights_data;
pub mod copy_rally_times;
pub mod copy_settings_in;
pub mod copy_settings_out;
pub mod copy_texture_info_tail;
pub mod count_checks;
pub mod countdown_lights;
pub mod course_data;
pub mod create_controls;
pub mod distance_to_checkpoint;
pub mod drop_absent_player;
pub mod face_normal;
pub mod fetch_view_value;
pub mod first_check_checkpoint;
pub mod frame;
pub mod frame_draw;
pub mod free_course_list_2;
pub mod free_course_memory;
pub mod free_game_fonts;
pub mod free_game_map;
pub mod free_mud;
pub mod free_object_points;
pub mod free_point_records;
pub mod game_map;
pub mod game_music;
pub mod game_music_8;
pub mod get_mat_crash;
pub mod grade_all_cars;
pub mod grade_car_values;
pub mod group_records_by_kind;
pub mod hold_to_limit;
pub mod index_of_kind;
pub mod init_checkpoints;
pub mod init_mud;
pub mod init_overlay;
pub mod init_skidmarks;
pub mod invert_car_matrix;
pub mod kind_past_table;
pub mod kind_unusable;
pub mod lamps_wanted;
pub mod lay_out_course;
pub mod level_init;
pub mod load_car_a;
pub mod load_car_b;
pub mod load_car_c;
pub mod load_error_list;
pub mod load_game_map;
pub mod load_level;
pub mod load_lights_data;
pub mod load_mud;
pub mod load_packed_game_map;
pub mod load_prefs;
pub mod load_stage;
pub mod load_texture_info;
pub mod loadlevel;
pub mod loadlevel_init;
pub mod locate_current_course;
pub mod map_checkpoints;
pub mod map_segment;
pub mod mud;
pub mod mud_pixel;
pub mod music;
pub mod nearest_checkpoint;
pub mod nearest_checkpoint_in;
pub mod next_map_anchor;
pub mod nonzero_random;
pub mod normalise_work_vector;
pub mod norms;
pub mod object_bytes;
pub mod objects;
pub mod objects_tidy;
pub mod open_replay;
pub mod pack_runs;
pub mod panel_marks;
pub mod panel_motion;
pub mod panel_neighbours;
pub mod panel_players;
pub mod panel_values;
pub mod parse_2_digits;
pub mod parse_3_digits;
pub mod permute_bits;
pub mod place_car_on_line;
pub mod players_all_out;
pub mod prepare_sound_tables;
pub mod race_cars;
pub mod race_end;
pub mod race_from_menu;
pub mod race_init;
pub mod race_keys;
pub mod race_shell;
pub mod rally;
pub mod read_stage_car_numbers;
pub mod record_replay_header;
pub mod release_controls;
pub mod release_stage_textures;
pub mod release_textures;
pub mod reload_objects;
pub mod render3d_init;
pub mod render3d_tidy;
pub mod replay;
pub mod replay_cameras_1;
pub mod replay_cameras_2;
pub mod reset_car_blocks;
pub mod reset_car_motion;
pub mod reset_car_race_state;
pub mod reset_car_starts;
pub mod reset_face_lights;
pub mod reset_lamps;
pub mod reset_race_sounds;
pub mod reset_refresh;
pub mod reset_skid_slots;
pub mod restart_race_sounds;
pub mod run_race;
pub mod sample_mud_colours;
pub mod save_seal;
pub mod seal_block;
pub mod seal_replay_header;
pub mod seal_stage_time;
pub mod seed_car_randoms;
pub mod set_all_plates;
pub mod set_both_counters;
pub mod set_car_counter;
pub mod set_car_times;
pub mod set_cdat_number;
pub mod set_course_number;
pub mod set_line_face_kind;
pub mod set_object_kinds;
pub mod set_player_car_kinds;
pub mod set_race_screen_mode;
pub mod set_start_states;
pub mod set_up_arcade_cars;
pub mod set_up_car_points;
pub mod set_up_cars;
pub mod set_up_mode_0_cars;
pub mod set_up_mode_1_cars;
pub mod set_up_mode_2_cars;
pub mod set_up_mode_4_cars;
pub mod set_up_panel;
pub mod set_up_race_cars;
pub mod set_up_race_sound_objects;
pub mod settings_copy;
pub mod settle_car;
pub mod share_sound_spots;
pub mod silence_race_sounds;
pub mod single_rally;
pub mod single_rally_race;
pub mod sort_checkpoints;
pub mod sort_points_by_x_down;
pub mod sort_points_by_x_down_b;
pub mod sort_points_by_y;
pub mod sort_points_by_z;
pub mod sound_row;
pub mod sound_span;
pub mod sound_tables;
pub mod sounds;
pub mod split_colours;
pub mod stage_free;
pub mod stage_loading;
pub mod stage_prefs;
pub mod start_car;
pub mod start_cars;
pub mod start_place;
pub mod start_race;
pub mod start_race_sounds;
pub mod stop_race_sounds;
pub mod store_car_a;
pub mod store_car_b;
pub mod store_car_c;
pub mod store_face_normal;
pub mod store_norms;
pub mod store_results_by_mode;
pub mod store_view_value;
pub mod surface_shares;
pub mod swap_stage_texture;
pub mod tidy_everything;
pub mod tidy_game_mode;
pub mod tidy_stage;
pub mod tweak_stage;
pub mod tyre_change;
pub mod wake_car_sounds;
pub mod wake_part_sounds;
pub mod whole_to_float;
pub mod write_replay_block;
pub mod write_replay_chunk;

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
    &clear_race_slots::HOOK,
    &reset_car_starts::HOOK,
    &nonzero_random::HOOK,
    &seed_car_randoms::HOOK,
    &reset_car_race_state::HOOK,
    &first_check_checkpoint::HOOK,
    &place_car_on_line::HOOK,
    &invert_car_matrix::HOOK,
    &distance_to_checkpoint::HOOK,
    &nearest_checkpoint_in::HOOK,
    &nearest_checkpoint::HOOK,
    &settle_car::HOOK,
    &reset_car_motion::HOOK,
    &start_car::HOOK,
    &start_cars::HOOK,
    &reset_car_blocks::HOOK,
    &drop_absent_player::HOOK,
    &set_player_car_kinds::HOOK,
    &parse_2_digits::HOOK,
    &parse_3_digits::HOOK,
    &set_up_race_cars::HOOK,
    &read_stage_car_numbers::HOOK,
    &auto_set_up_car::HOOK,
    &set_up_mode_0_cars::HOOK,
    &set_up_mode_1_cars::HOOK,
    &set_up_mode_4_cars::HOOK,
    &clear_race_tables::HOOK,
    &set_car_times::HOOK,
    &set_up_mode_2_cars::HOOK,
    &set_up_cars::HOOK,
    &grade_car_values::HOOK,
    &grade_all_cars::HOOK,
    &loadlevel_init::HOOK,
    &load_stage::HOOK,
    &free_course_memory::HOOK,
    &free_course_list_2::HOOK,
    &free_point_records::HOOK,
    &free_object_points::HOOK,
    &free_mud::HOOK,
    &free_game_fonts::HOOK,
    &release_controls::HOOK,
    &clear_player_absences::HOOK,
    &kind_past_table::HOOK,
    &kind_unusable::HOOK,
    &index_of_kind::HOOK,
    &set_race_screen_mode::HOOK,
    &release_stage_textures::HOOK,
    &seal_stage_time::HOOK,
    &permute_bits::HOOK,
    &copy_league_table::HOOK,
    &set_start_states::HOOK,
    &store_results_by_mode::HOOK,
    &pack_runs::HOOK,
    &seal_block::HOOK,
    &choose_arcade_opponents::HOOK,
    &set_up_arcade_cars::HOOK,
    &seal_replay_header::HOOK,
    &write_replay_chunk::HOOK,
    &write_replay_block::HOOK,
    &add_replay_frame::HOOK,
    &close_replay::HOOK,
    &open_replay::HOOK,
    &record_replay_header::HOOK,
    &tidy_game_mode::HOOK,
    &render3d_tidy::HOOK,
    &objects_tidy::HOOK,
    &reset_race_sounds::HOOK,
    &stop_race_sounds::HOOK,
    &tidy_stage::HOOK,
    &tidy_everything::HOOK,
    &copy_settings_in::HOOK,
    &copy_settings_out::HOOK,
    &run_race::HOOK,
    &whole_to_float::HOOK,
    &sound_span::HOOK,
    &sound_row::HOOK,
    &prepare_sound_tables::HOOK,
    &restart_race_sounds::HOOK,
    &silence_race_sounds::HOOK,
    &set_up_race_sound_objects::HOOK,
    &clear_race_sound_objects::HOOK,
    &release_textures::HOOK,
    &start_race::HOOK,
    &race_from_menu::HOOK,
    &copy_rally_times::HOOK,
    &single_rally_race::HOOK,
    &players_all_out::HOOK,
    &set_car_counter::HOOK,
    &set_both_counters::HOOK,
    &load_car_a::HOOK,
    &load_car_b::HOOK,
    &load_car_c::HOOK,
    &store_car_a::HOOK,
    &store_car_b::HOOK,
    &store_car_c::HOOK,
    &race_keys::HOOK,
    &tyre_change::HOOK,
    &car_label::HOOK,
    &split_colours::HOOK,
    &panel_players::HOOK,
    &lamps_wanted::HOOK,
    &store_view_value::HOOK,
    &fetch_view_value::HOOK,
    &share_sound_spots::HOOK,
    &clear_point_records::HOOK,
    &object_bytes::HOOK,
    &panel_values::HOOK,
    &wake_part_sounds::HOOK,
    &panel_neighbours::HOOK,
    &panel_motion::HOOK,
    &panel_marks::HOOK,
    &countdown_lights::HOOK,
    &both_countdown_lights::HOOK,
    &both_split_colours::HOOK,
    &set_up_panel::HOOK,
    &reset_lamps::HOOK,
    &hold_to_limit::HOOK,
    &add_sort_entry::HOOK,
    &checkpoint_colours::HOOK,
];
