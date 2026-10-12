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
pub mod begin_replay_name;
pub mod begin_savegame_name;
pub mod best_class_scores_loop;
pub mod best_overall_scores_loop;
pub mod best_scores_48a69e_loop;
pub mod best_times_loop;
pub mod car_specification_loop;
pub mod car_specs_47af4c_loop;
pub mod championship_new_simulation_loop;
pub mod championship_standings_loop;
pub mod class_championship_standings_loop;
pub mod class_rally_results_loop;
pub mod class_stage_results_loop;
pub mod clear_text;
pub mod draw_frame;
pub mod draw_menu;
pub mod drop_missing_controllers;
pub mod edit_replay_name;
pub mod edit_savegame_name;
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
pub mod load_championship_46e899_loop;
pub mod load_championship_loop;
pub mod load_replay_loop;
pub mod main_menu_loop;
pub mod move_highlight_down;
pub mod move_highlight_left;
pub mod move_highlight_right;
pub mod move_highlight_up;
pub mod name_lists;
pub mod navigation;
pub mod network_rally_loop;
pub mod new_simulation_loop;
pub mod next_stage_loop;
pub mod options_loop;
pub mod overall_rally_results_loop;
pub mod overall_stage_results_loop;
pub mod player1_details_loop;
pub mod player2_details_loop;
pub mod player3_details_loop;
pub mod player4_details_loop;
pub mod player_details_loop;
pub mod player_names;
pub mod rally_stages_loop;
pub mod redefine_keys_player1_loop;
pub mod redefine_keys_player2_loop;
pub mod redefine_keys_player3_loop;
pub mod redefine_keys_player4_loop;
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
pub mod simple_select_players_loop;
pub mod slideshow_loop;
pub mod stage_result_loop;
pub mod start_rally_loop;
pub mod store_player_names;
pub mod text_entry;
pub mod time_trial_best_times_loop;
pub mod time_trial_results_loop;
pub mod time_trial_select_players_loop;
pub mod time_trial_select_stage_loop;
pub mod time_trial_stage_details_loop;
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
    &draw_menu::HOOK,
    &load_background::HOOK,
    &draw_frame::HOOK,
    &frame_callback::HOOK,
    &go_back::HOOK,
    &remove_secret_cars::HOOK,
    &apply_cheats::HOOK,
    &add_clio_car::HOOK,
    &drop_missing_controllers::HOOK,
    &link_player_names::HOOK,
    &store_player_names::HOOK,
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
