//! The menu system's state.

use crate::framework::GameHook;

pub mod add_clio_car;
pub mod add_text_length;
pub mod advanced_audio_loop;
pub mod apply_cheats;
pub mod arcade_best_times_loop;
pub mod arcade_next_stage_loop;
pub mod arcade_results_loop;
pub mod arcade_review_options_loop;
pub mod arcade_select_level_loop;
pub mod audio_setup_loop;
pub mod backspace_savegame_name;
pub mod backup_text;
pub mod begin_player_name;
pub mod begin_replay_name;
pub mod begin_savegame_name;
pub mod best_class_scores_loop;
pub mod best_overall_scores_loop;
pub mod best_scores_48a69e_loop;
pub mod best_times_loop;
pub mod build_notice;
pub mod car_buttons;
pub mod car_description;
pub mod car_specification_loop;
pub mod car_specs_47af4c_loop;
pub mod championship_new_simulation_loop;
pub mod championship_standings_loop;
pub mod choose_key;
pub mod choose_key_0;
pub mod choose_key_1;
pub mod choose_key_10;
pub mod choose_key_11;
pub mod choose_key_2;
pub mod choose_key_3;
pub mod choose_key_4;
pub mod choose_key_5;
pub mod choose_key_6;
pub mod choose_key_7;
pub mod choose_key_8;
pub mod choose_key_9;
pub mod class_championship_standings_loop;
pub mod class_rally_results_loop;
pub mod class_stage_results_loop;
pub mod clear_text;
pub mod confirm_leaving;
pub mod disable_item;
pub mod draw_frame;
pub mod draw_menu;
pub mod drop_missing_controllers;
pub mod edit_player_name;
pub mod edit_replay_name;
pub mod edit_savegame_name;
pub mod enable_item;
pub mod fit_player_names;
pub mod flip_codriver;
pub mod flip_yes_no;
pub mod frame_callback;
pub mod frame_test;
pub mod go_back;
pub mod handle_keys;
pub mod handle_yes_no;
pub mod in_game_options_loop;
pub mod information_loop;
pub mod inspection_area_loop;
pub mod item_at;
pub mod leave_question_loop;
pub mod link_player_names;
pub mod load_background;
pub mod load_car_background;
pub mod load_championship_46e899_loop;
pub mod load_championship_loop;
pub mod load_replay_loop;
pub mod load_slideshow_picture;
pub mod lower_sensitivity;
pub mod main_menu_loop;
pub mod move_highlight_down;
pub mod move_highlight_left;
pub mod move_highlight_right;
pub mod move_highlight_up;
pub mod name_lists;
pub mod navigation;
pub mod network_rally_loop;
pub mod new_simulation_loop;
pub mod next_car;
pub mod next_car_number;
pub mod next_car_page;
pub mod next_nationality;
pub mod next_savegame_page;
pub mod next_second_view;
pub mod next_slideshow_car;
pub mod next_slideshow_picture;
pub mod next_stage_loop;
pub mod next_view;
pub mod options_loop;
pub mod overall_rally_results_loop;
pub mod overall_stage_results_loop;
pub mod play_films;
pub mod player;
pub mod player1_details_loop;
pub mod player2_details_loop;
pub mod player3_details_loop;
pub mod player4_details_loop;
pub mod player_details_loop;
pub mod player_name;
pub mod player_names;
pub mod player_settings;
pub mod previous_car;
pub mod previous_car_number;
pub mod previous_car_page;
pub mod previous_nationality;
pub mod previous_second_view;
pub mod previous_slideshow_car;
pub mod previous_slideshow_picture;
pub mod previous_view;
pub mod raise_sensitivity;
pub mod rally_stages_loop;
pub mod redefine_key;
pub mod redefine_keys;
pub mod redefine_keys_player1_loop;
pub mod redefine_keys_player2_loop;
pub mod redefine_keys_player3_loop;
pub mod redefine_keys_player4_loop;
pub mod redefine_player1_key;
pub mod remove_secret_cars;
pub mod restore_text;
pub mod results_482e9b_loop;
pub mod results_48584b_loop;
pub mod review_options_47d5a5_loop;
pub mod review_options_480d46_loop;
pub mod review_options_48108e_loop;
pub mod review_options_481429_loop;
pub mod review_options_485f5e_loop;
pub mod review_options_4882e2_loop;
pub mod review_options_48861d_loop;
pub mod run_draw_list;
pub mod run_front_end;
pub mod run_item_action;
pub mod run_list;
pub mod save_championship_loop;
pub mod save_replay_loop;
pub mod save_setup_loop;
pub mod screen_47814c_loop;
pub mod screen_4783f6_loop;
pub mod screen_4786aa_loop;
pub mod screen_478954_loop;
pub mod screen_478c08_loop;
pub mod screen_478eb2_loop;
pub mod screen_479166_loop;
pub mod screen_479410_loop;
pub mod screen_483163_loop;
pub mod screen_488f4c_loop;
pub mod screen_loop;
pub mod select_car_loop;
pub mod select_stage_loop;
pub mod set_up_front_end;
pub mod show_loading_picture;
pub mod simple_select_players_loop;
pub mod slideshow_loop;
pub mod stage_result_loop;
pub mod start_rally_loop;
pub mod step_carousel;
pub mod store_player_names;
pub mod text_entry;
pub mod time_trial_best_times_loop;
pub mod time_trial_results_loop;
pub mod time_trial_select_players_loop;
pub mod time_trial_select_stage_loop;
pub mod time_trial_stage_details_loop;
pub mod toggle_car_info;
pub mod toggle_driver_info;
pub mod toggle_map_area;
pub mod toggle_message_display;
pub mod toggle_metric_units;
pub mod toggle_pace_notes;
pub mod toggle_progress_bar;
pub mod toggle_rear_mirror;
pub mod toggle_rev_counter;
pub mod update_blink;
pub mod update_frame_clock;
pub mod view_best_scores_loop;
pub mod view_best_times_loop;
pub mod view_courses_loop;
pub mod view_stage_loop;
pub mod visual_options_loop;
pub mod visual_preferences_loop;
pub mod welcome_loop;

pub static HOOKS: &[&dyn GameHook] = &[
    &step_carousel::HOOK,
    &draw_menu::HOOK,
    &load_background::HOOK,
    &draw_frame::HOOK,
    &frame_callback::HOOK,
    &go_back::HOOK,
    &remove_secret_cars::HOOK,
    &apply_cheats::HOOK,
    &build_notice::HOOK,
    &add_clio_car::HOOK,
    &drop_missing_controllers::HOOK,
    &fit_player_names::HOOK,
    &link_player_names::HOOK,
    &store_player_names::HOOK,
    &set_up_front_end::HOOK,
    &show_loading_picture::HOOK,
    &run_front_end::HOOK,
    &play_films::HOOK,
    &backup_text::HOOK,
    &restore_text::HOOK,
    &clear_text::HOOK,
    &add_text_length::HOOK,
    &backspace_savegame_name::HOOK,
    &edit_replay_name::HOOK,
    &edit_savegame_name::HOOK,
    &begin_savegame_name::HOOK,
    &begin_replay_name::HOOK,
    &update_frame_clock::HOOK,
    &update_blink::HOOK,
    &item_at::HOOK,
    &run_list::HOOK,
    &run_draw_list::HOOK,
    &run_item_action::HOOK,
    &handle_yes_no::HOOK,
    &move_highlight_up::HOOK,
    &move_highlight_down::HOOK,
    &move_highlight_right::HOOK,
    &move_highlight_left::HOOK,
    &handle_keys::HOOK,
    &class_stage_results_loop::HOOK,
    &overall_stage_results_loop::HOOK,
    &class_rally_results_loop::HOOK,
    &overall_rally_results_loop::HOOK,
    &rally_stages_loop::HOOK,
    &championship_standings_loop::HOOK,
    &screen_47814c_loop::HOOK,
    &screen_4783f6_loop::HOOK,
    &screen_4786aa_loop::HOOK,
    &screen_478954_loop::HOOK,
    &screen_478c08_loop::HOOK,
    &screen_478eb2_loop::HOOK,
    &screen_479166_loop::HOOK,
    &screen_479410_loop::HOOK,
    &time_trial_stage_details_loop::HOOK,
    &leave_question_loop::HOOK,
    &inspection_area_loop::HOOK,
    &visual_options_loop::HOOK,
    &advanced_audio_loop::HOOK,
    &screen_483163_loop::HOOK,
    &stage_result_loop::HOOK,
    &screen_488f4c_loop::HOOK,
    &best_times_loop::HOOK,
    &class_championship_standings_loop::HOOK,
    &best_class_scores_loop::HOOK,
    &best_overall_scores_loop::HOOK,
    &audio_setup_loop::HOOK,
    &visual_preferences_loop::HOOK,
    &time_trial_select_players_loop::HOOK,
    &view_best_times_loop::HOOK,
    &view_courses_loop::HOOK,
    &view_best_scores_loop::HOOK,
    &new_simulation_loop::HOOK,
    &options_loop::HOOK,
    &player_details_loop::HOOK,
    &slideshow_loop::HOOK,
    &in_game_options_loop::HOOK,
    &load_championship_loop::HOOK,
    &player1_details_loop::HOOK,
    &player2_details_loop::HOOK,
    &player3_details_loop::HOOK,
    &player4_details_loop::HOOK,
    &save_championship_loop::HOOK,
    &save_setup_loop::HOOK,
    &information_loop::HOOK,
    &review_options_47d5a5_loop::HOOK,
    &review_options_480d46_loop::HOOK,
    &review_options_48108e_loop::HOOK,
    &review_options_4882e2_loop::HOOK,
    &review_options_48861d_loop::HOOK,
    &arcade_review_options_loop::HOOK,
    &arcade_results_loop::HOOK,
    &arcade_best_times_loop::HOOK,
    &best_scores_48a69e_loop::HOOK,
    &time_trial_best_times_loop::HOOK,
    &time_trial_results_loop::HOOK,
    &network_rally_loop::HOOK,
    &review_options_481429_loop::HOOK,
    &review_options_485f5e_loop::HOOK,
    &simple_select_players_loop::HOOK,
    &results_48584b_loop::HOOK,
    &next_stage_loop::HOOK,
    &view_stage_loop::HOOK,
    &arcade_next_stage_loop::HOOK,
    &time_trial_select_stage_loop::HOOK,
    &championship_new_simulation_loop::HOOK,
    &arcade_select_level_loop::HOOK,
    &welcome_loop::HOOK,
    &main_menu_loop::HOOK,
    &confirm_leaving::HOOK,
    &begin_player_name::HOOK,
    &edit_player_name::HOOK,
    &next_nationality::HOOK,
    &previous_nationality::HOOK,
    &next_view::HOOK,
    &previous_view::HOOK,
    &next_second_view::HOOK,
    &previous_second_view::HOOK,
    &raise_sensitivity::HOOK,
    &lower_sensitivity::HOOK,
    &next_car_number::HOOK,
    &previous_car_number::HOOK,
    &choose_key_0::HOOK,
    &choose_key_1::HOOK,
    &choose_key_2::HOOK,
    &choose_key_3::HOOK,
    &choose_key_4::HOOK,
    &choose_key_5::HOOK,
    &choose_key_6::HOOK,
    &choose_key_7::HOOK,
    &choose_key_8::HOOK,
    &choose_key_9::HOOK,
    &choose_key_10::HOOK,
    &choose_key_11::HOOK,
    &redefine_player1_key::HOOK,
    &redefine_key::HOOK,
    &toggle_map_area::HOOK,
    &toggle_rear_mirror::HOOK,
    &toggle_rev_counter::HOOK,
    &toggle_car_info::HOOK,
    &toggle_message_display::HOOK,
    &toggle_progress_bar::HOOK,
    &toggle_driver_info::HOOK,
    &toggle_pace_notes::HOOK,
    &toggle_metric_units::HOOK,
    &flip_codriver::HOOK,
    &disable_item::HOOK,
    &enable_item::HOOK,
    &load_car_background::HOOK,
    &load_slideshow_picture::HOOK,
    &next_car_page::HOOK,
    &next_car::HOOK,
    &previous_car::HOOK,
    &previous_car_page::HOOK,
    &next_slideshow_car::HOOK,
    &previous_slideshow_car::HOOK,
    &previous_slideshow_picture::HOOK,
    &next_slideshow_picture::HOOK,
    &flip_yes_no::HOOK,
    &next_savegame_page::HOOK,
    &start_rally_loop::HOOK,
    &redefine_keys_player1_loop::HOOK,
    &redefine_keys_player2_loop::HOOK,
    &redefine_keys_player3_loop::HOOK,
    &redefine_keys_player4_loop::HOOK,
    &car_specification_loop::HOOK,
    &select_car_loop::HOOK,
    &car_specs_47af4c_loop::HOOK,
    &load_replay_loop::HOOK,
    &save_replay_loop::HOOK,
    &select_stage_loop::HOOK,
    &results_482e9b_loop::HOOK,
    &load_championship_46e899_loop::HOOK,
];
