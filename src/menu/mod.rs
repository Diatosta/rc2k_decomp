//! The menu system's state.

use crate::framework::GameHook;

pub mod a8_welcome;
pub mod add_clio_car;
pub mod add_player_times;
pub mod add_shown_car;
pub mod add_text_length;
pub mod advanced_audio_loop;
pub mod apply_audio_choices;
pub mod apply_cheats;
pub mod apply_controller;
pub mod apply_screen_mode;
pub mod arcade_best_times_loop;
pub mod arcade_event;
pub mod arcade_finale;
pub mod arcade_level_1;
pub mod arcade_level_2;
pub mod arcade_level_3;
pub mod arcade_level_4;
pub mod arcade_level_5;
pub mod arcade_level_end;
pub mod arcade_levels;
pub mod arcade_next_stage_loop;
pub mod arcade_results_loop;
pub mod arcade_review_options_loop;
pub mod arcade_select_level_loop;
pub mod arcade_steps;
pub mod ask_to_leave;
pub mod audio_setup_loop;
pub mod award_overall_points;
pub mod award_stage_points;
pub mod backspace_savegame_name;
pub mod backup_text;
pub mod beat_target_time;
pub mod begin_event;
pub mod begin_player_name;
pub mod begin_replay_name;
pub mod begin_savegame_name;
pub mod best_class_scores;
pub mod best_class_scores_loop;
pub mod best_overall_scores;
pub mod best_overall_scores_loop;
pub mod best_scores_48a69e_loop;
pub mod best_times_loop;
pub mod build_notice;
pub mod calc_cpu_times;
pub mod car_buttons;
pub mod car_choice;
pub mod car_description;
pub mod car_specification_loop;
pub mod car_specs_47af4c_loop;
pub mod championship_finish;
pub mod championship_new_simulation_loop;
pub mod championship_standings;
pub mod championship_standings_loop;
pub mod championship_welcome;
pub mod cheat_car_list;
pub mod check_arcade_level_score;
pub mod check_arcade_stage_time;
pub mod check_cheat_name;
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
pub mod class_championship_pass;
pub mod class_championship_standings_loop;
pub mod class_rally_results_loop;
pub mod class_rally_results_pass;
pub mod class_stage_results_loop;
pub mod class_stage_results_pass;
pub mod clear_arcade_score_marks;
pub mod clear_arcade_time_marks;
pub mod clear_choice_marks;
pub mod clear_event_state;
pub mod clear_players_284;
pub mod clear_players_288;
pub mod clear_record_entries;
pub mod clear_result_slots;
pub mod clear_score_marks;
pub mod clear_standings_a;
pub mod clear_standings_b;
pub mod clear_text;
pub mod clear_time_trial_marks;
pub mod confirm_leaving;
pub mod controller_items;
pub mod copy_standing_entry;
pub mod copy_standings_summary;
pub mod count_stage_pages;
pub mod disable_item;
pub mod draw_frame;
pub mod draw_menu;
pub mod draw_stage_map;
pub mod driver_stats;
pub mod drop_missing_controllers;
pub mod edit_player_name;
pub mod edit_replay_name;
pub mod edit_savegame_name;
pub mod enable_item;
pub mod enter_arcade_level_score;
pub mod enter_arcade_stage_time;
pub mod enter_best_score;
pub mod enter_best_score_4;
pub mod enter_record_entry;
pub mod enter_time_trial_best;
pub mod event_flow;
pub mod fade_frame;
pub mod fade_in;
pub mod fade_out;
pub mod fades;
pub mod find_class_time;
pub mod find_cpu_team;
pub mod find_player_car;
pub mod find_section_player;
pub mod finish_calls;
pub mod first_page_stage;
pub mod first_player_a;
pub mod first_player_b;
pub mod first_player_unplaced;
pub mod fit_player_names;
pub mod flip_codriver;
pub mod flip_gearing;
pub mod flip_yes_no;
pub mod frame_callback;
pub mod frame_test;
pub mod go_back;
pub mod handle_keys;
pub mod handle_yes_no;
pub mod hide_in_game_options;
pub mod in_game_options_items;
pub mod in_game_options_loop;
pub mod information_loop;
pub mod init_standings_a;
pub mod init_standings_b;
pub mod inspection_area_loop;
pub mod inspection_faults;
pub mod inspection_frame;
pub mod inspection_pass;
pub mod inspection_screen;
pub mod inspection_verdict;
pub mod item_actions;
pub mod item_at;
pub mod keep_arcade_stage_best;
pub mod keep_record_best;
pub mod keep_stage_best;
pub mod learn_bindings;
pub mod leave_question_loop;
pub mod link_player_names;
pub mod load_background;
pub mod load_car_background;
pub mod load_championship_46e899_loop;
pub mod load_championship_loop;
pub mod load_replay_loop;
pub mod load_slideshow_picture;
pub mod load_welcome_picture;
pub mod lobby_items;
pub mod lower_controller_sensitivity;
pub mod lower_sensitivity;
pub mod main_menu_loop;
pub mod mark_stage_choice;
pub mod menu_sounds_a_volume_down;
pub mod menu_sounds_a_volume_up;
pub mod menu_sounds_b_volume_down;
pub mod menu_sounds_b_volume_up;
pub mod move_highlight_down;
pub mod move_highlight_left;
pub mod move_highlight_right;
pub mod move_highlight_up;
pub mod music_volume_down;
pub mod music_volume_up;
pub mod name_lists;
pub mod navigation;
pub mod network_rally_loop;
pub mod new_simulation_loop;
pub mod next_car;
pub mod next_car_number;
pub mod next_car_page;
pub mod next_choice_car;
pub mod next_colour_a;
pub mod next_colour_b;
pub mod next_controller;
pub mod next_gearing;
pub mod next_nationality;
pub mod next_record_kind;
pub mod next_savegame_page;
pub mod next_screen_mode;
pub mod next_second_view;
pub mod next_slideshow_car;
pub mod next_slideshow_picture;
pub mod next_stage_loop;
pub mod next_stage_number;
pub mod next_stage_page;
pub mod next_stage_rows;
pub mod next_stage_rows_of_90;
pub mod next_stage_sub;
pub mod next_texture_setting;
pub mod next_view;
pub mod number_all_a;
pub mod number_all_b;
pub mod number_standings_a;
pub mod number_standings_b;
pub mod option_items;
pub mod option_steps;
pub mod options_loop;
pub mod overall_rally_results_loop;
pub mod overall_stage_results_loop;
pub mod parts;
pub mod picture_frame;
pub mod plain_frame;
pub mod play_films;
pub mod play_rally_speech;
pub mod play_stage_speech;
pub mod player;
pub mod player1_details_loop;
pub mod player2_details_loop;
pub mod player3_details_loop;
pub mod player4_details_loop;
pub mod player_actions;
pub mod player_details_loop;
pub mod player_entered;
pub mod player_name;
pub mod player_names;
pub mod player_settings;
pub mod player_to_standing_b;
pub mod previous_car;
pub mod previous_car_number;
pub mod previous_car_page;
pub mod previous_choice_car;
pub mod previous_colour_a;
pub mod previous_colour_b;
pub mod previous_controller;
pub mod previous_gearing;
pub mod previous_nationality;
pub mod previous_second_view;
pub mod previous_slideshow_car;
pub mod previous_slideshow_picture;
pub mod previous_stage_number;
pub mod previous_stage_page;
pub mod previous_stage_rows;
pub mod previous_stage_rows_of_90;
pub mod previous_stage_sub;
pub mod previous_texture_setting;
pub mod previous_view;
pub mod raise_controller_sensitivity;
pub mod raise_dead_zone_a;
pub mod raise_dead_zone_b;
pub mod raise_sensitivity;
pub mod rally_results;
pub mod rally_stages_loop;
pub mod rank_best_score;
pub mod rank_best_score_4;
pub mod rank_record_entry;
pub mod rank_stage_times;
pub mod rank_time_trial_best;
pub mod rank_totals;
pub mod ranked_lookups;
pub mod ready_players;
pub mod record_entries_pass;
pub mod redefine_key;
pub mod redefine_keys;
pub mod redefine_keys_player1_loop;
pub mod redefine_keys_player2_loop;
pub mod redefine_keys_player3_loop;
pub mod redefine_keys_player4_loop;
pub mod redefine_player1_key;
pub mod release_controller;
pub mod reload_front_end;
pub mod remove_secret_cars;
pub mod remove_shown_car;
pub mod replay_item_a;
pub mod replay_item_b;
pub mod replay_item_c;
pub mod replay_items;
pub mod reset_cars_750724;
pub mod reset_driver_condition;
pub mod reset_drivers;
pub mod reset_player_times;
pub mod reset_player_totals;
pub mod reset_single_event;
pub mod restore_text;
pub mod results_482e9b_loop;
pub mod results_48584b_loop;
pub mod results_setups;
pub mod review_options_47d5a5_loop;
pub mod review_options_480d46_loop;
pub mod review_options_48108e_loop;
pub mod review_options_481429_loop;
pub mod review_options_485f5e_loop;
pub mod review_options_4882e2_loop;
pub mod review_options_48861d_loop;
pub mod rounds;
pub mod run_a8_championship;
pub mod run_championship;
pub mod run_draw_list;
pub mod run_event_races;
pub mod run_front_end;
pub mod run_item_action;
pub mod run_list;
pub mod save_championship_loop;
pub mod save_replay_loop;
pub mod save_setup_loop;
pub mod scale_stage_times;
pub mod score_arcade_stage;
pub mod score_four_players;
pub mod score_two_players;
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
pub mod scripted;
pub mod sealed_player_time;
pub mod section_finish;
pub mod select_car_loop;
pub mod select_stage_loop;
pub mod service_pass;
pub mod set_player_entry_kind;
pub mod set_up_calls;
pub mod set_up_front_end;
pub mod show_car_specs;
pub mod show_championship_standings;
pub mod show_class_championship_standings;
pub mod show_class_rally_results;
pub mod show_class_stage_results;
pub mod show_in_game_options;
pub mod show_loading_picture;
pub mod show_overall_rally_results;
pub mod show_overall_stage_results;
pub mod shuffle_drivers;
pub mod simple_select_players_loop;
pub mod single_rally;
pub mod single_rally_four;
pub mod single_rally_split;
pub mod single_rally_three;
pub mod single_rally_two;
pub mod slideshow_loop;
pub mod sort_arcade_level_scores;
pub mod sort_arcade_stage_times;
pub mod sort_standings_b_by_figure;
pub mod sort_standings_b_by_points;
pub mod sort_standings_b_by_score;
pub mod sort_standings_by_time;
pub mod sort_standings_by_total;
pub mod speech_volume_down;
pub mod speech_volume_up;
pub mod stage_choice;
pub mod stage_range_caption;
pub mod stage_result_loop;
pub mod stage_rows;
pub mod stage_view_steps;
pub mod standings;
pub mod standings_a_from_all;
pub mod standings_b_from_all;
pub mod standings_from_drivers;
pub mod start_a8_four;
pub mod start_a8_one;
pub mod start_a8_split;
pub mod start_a8_three;
pub mod start_a8_two;
pub mod start_championship_four;
pub mod start_championship_one;
pub mod start_championship_split;
pub mod start_championship_three;
pub mod start_championship_two;
pub mod start_event;
pub mod start_rally_loop;
pub mod step_carousel;
pub mod store_player_names;
pub mod style_stage_item_a;
pub mod style_stage_item_b;
pub mod switch_stage_group;
pub mod take_arcade_stage_best;
pub mod take_player1_units;
pub mod take_shown_car;
pub mod text_entry;
pub mod time_trial_best_times_loop;
pub mod time_trial_four;
pub mod time_trial_one;
pub mod time_trial_qualifies;
pub mod time_trial_results_loop;
pub mod time_trial_select_players_loop;
pub mod time_trial_select_stage_loop;
pub mod time_trial_service;
pub mod time_trial_split;
pub mod time_trial_stage_details_loop;
pub mod time_trial_three;
pub mod time_trial_two;
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
pub mod walk_section_records;
pub mod welcome_loop;

pub mod list_clear_marks;
pub mod list_item_unusable;
pub mod list_step;
pub mod list_take_key;

pub mod replay_slot_loop;

pub static HOOKS: &[&dyn GameHook] = &[
    &lobby_items::settings::HOOK,
    &lobby_items::link_2::HOOK,
    &lobby_items::link_1::HOOK,
    &add_player_times::HOOK,
    &award_overall_points::HOOK,
    &award_stage_points::HOOK,
    &beat_target_time::HOOK,
    &best_class_scores::HOOK,
    &best_overall_scores::HOOK,
    &calc_cpu_times::HOOK,
    &championship_finish::HOOK,
    &championship_standings::HOOK,
    &class_championship_pass::HOOK,
    &class_rally_results_pass::HOOK,
    &class_stage_results_pass::HOOK,
    &clear_players_288::HOOK,
    &clear_record_entries::HOOK,
    &clear_result_slots::HOOK,
    &clear_score_marks::HOOK,
    &clear_standings_a::HOOK,
    &clear_standings_b::HOOK,
    &copy_standing_entry::HOOK,
    &copy_standings_summary::HOOK,
    &driver_stats::HOOK,
    &enter_best_score::HOOK,
    &enter_best_score_4::HOOK,
    &enter_record_entry::HOOK,
    &fade_frame::HOOK,
    &fade_in::HOOK,
    &fade_out::HOOK,
    &find_cpu_team::HOOK,
    &find_section_player::HOOK,
    &first_page_stage::HOOK,
    &first_player_a::HOOK,
    &first_player_b::HOOK,
    &first_player_unplaced::HOOK,
    &init_standings_a::HOOK,
    &init_standings_b::HOOK,
    &inspection_faults::HOOK,
    &inspection_pass::HOOK,
    &inspection_screen::HOOK,
    &inspection_verdict::HOOK,
    &keep_record_best::HOOK,
    &next_record_kind::HOOK,
    &number_all_a::HOOK,
    &number_all_b::HOOK,
    &number_standings_a::HOOK,
    &number_standings_b::HOOK,
    &picture_frame::HOOK,
    &player_entered::HOOK,
    &player_to_standing_b::HOOK,
    &rally_results::HOOK,
    &rank_best_score::HOOK,
    &rank_best_score_4::HOOK,
    &rank_record_entry::HOOK,
    &record_entries_pass::HOOK,
    &reload_front_end::HOOK,
    &reset_cars_750724::HOOK,
    &reset_driver_condition::HOOK,
    &reset_drivers::HOOK,
    &reset_player_times::HOOK,
    &reset_player_totals::HOOK,
    &reset_single_event::HOOK,
    &run_event_races::HOOK,
    &scale_stage_times::HOOK,
    &sealed_player_time::HOOK,
    &section_finish::HOOK,
    &service_pass::HOOK,
    &set_player_entry_kind::HOOK,
    &show_championship_standings::HOOK,
    &show_class_championship_standings::HOOK,
    &show_class_rally_results::HOOK,
    &show_class_stage_results::HOOK,
    &show_overall_rally_results::HOOK,
    &show_overall_stage_results::HOOK,
    &shuffle_drivers::HOOK,
    &sort_standings_b_by_figure::HOOK,
    &sort_standings_b_by_points::HOOK,
    &sort_standings_b_by_score::HOOK,
    &sort_standings_by_time::HOOK,
    &sort_standings_by_total::HOOK,
    &stage_range_caption::HOOK,
    &standings_a_from_all::HOOK,
    &standings_b_from_all::HOOK,
    &standings_from_drivers::HOOK,
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
    &walk_section_records::HOOK,
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
    &clear_players_284::HOOK,
    &take_player1_units::HOOK,
    &take_shown_car::HOOK,
    &hide_in_game_options::HOOK,
    &show_in_game_options::HOOK,
    &clear_event_state::HOOK,
    &begin_event::HOOK,
    &single_rally::HOOK,
    &draw_stage_map::HOOK,
    &find_player_car::HOOK,
    &next_choice_car::HOOK,
    &previous_choice_car::HOOK,
    &clear_choice_marks::HOOK,
    &switch_stage_group::HOOK,
    &next_stage_sub::HOOK,
    &previous_stage_sub::HOOK,
    &next_stage_number::HOOK,
    &previous_stage_number::HOOK,
    &ask_to_leave::HOOK,
    &replay_item_a::HOOK,
    &replay_item_b::HOOK,
    &mark_stage_choice::HOOK,
    &next_gearing::HOOK,
    &previous_gearing::HOOK,
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
    &list_step::HOOK,
    &list_clear_marks::HOOK,
    &list_item_unusable::HOOK,
    &list_take_key::HOOK,
    &replay_slot_loop::HOOK,
    &ready_players::HOOK,
    &cheat_car_list::HOOK,
    &time_trial_qualifies::HOOK,
    &clear_time_trial_marks::HOOK,
    &keep_stage_best::HOOK,
    &rank_time_trial_best::HOOK,
    &enter_time_trial_best::HOOK,
    &time_trial_service::HOOK,
    &championship_welcome::HOOK,
    &a8_welcome::HOOK,
    &run_championship::HOOK,
    &run_a8_championship::HOOK,
    &start_championship_one::HOOK,
    &start_championship_split::HOOK,
    &start_championship_two::HOOK,
    &start_championship_three::HOOK,
    &start_championship_four::HOOK,
    &start_a8_one::HOOK,
    &start_a8_split::HOOK,
    &start_a8_two::HOOK,
    &start_a8_three::HOOK,
    &start_a8_four::HOOK,
    &single_rally_split::HOOK,
    &single_rally_two::HOOK,
    &single_rally_three::HOOK,
    &single_rally_four::HOOK,
    &time_trial_one::HOOK,
    &time_trial_split::HOOK,
    &time_trial_two::HOOK,
    &time_trial_three::HOOK,
    &time_trial_four::HOOK,
    &apply_controller::HOOK,
    &release_controller::HOOK,
    &next_controller::HOOK,
    &previous_controller::HOOK,
    &lower_controller_sensitivity::HOOK,
    &raise_controller_sensitivity::HOOK,
    &raise_dead_zone_a::HOOK,
    &raise_dead_zone_b::HOOK,
    &music_volume_down::HOOK,
    &music_volume_up::HOOK,
    &speech_volume_down::HOOK,
    &speech_volume_up::HOOK,
    &menu_sounds_a_volume_down::HOOK,
    &menu_sounds_a_volume_up::HOOK,
    &menu_sounds_b_volume_down::HOOK,
    &menu_sounds_b_volume_up::HOOK,
    &next_screen_mode::HOOK,
    &previous_texture_setting::HOOK,
    &next_texture_setting::HOOK,
    &next_colour_a::HOOK,
    &previous_colour_a::HOOK,
    &next_colour_b::HOOK,
    &previous_colour_b::HOOK,
    &apply_audio_choices::HOOK,
    &flip_gearing::HOOK,
    &rank_stage_times::HOOK,
    &rank_totals::HOOK,
    &score_four_players::HOOK,
    &score_two_players::HOOK,
    &keep_arcade_stage_best::HOOK,
    &check_arcade_level_score::HOOK,
    &check_arcade_stage_time::HOOK,
    &enter_arcade_stage_time::HOOK,
    &sort_arcade_stage_times::HOOK,
    &enter_arcade_level_score::HOOK,
    &sort_arcade_level_scores::HOOK,
    &score_arcade_stage::HOOK,
    &clear_arcade_score_marks::HOOK,
    &clear_arcade_time_marks::HOOK,
    &take_arcade_stage_best::HOOK,
    &add_shown_car::HOOK,
    &remove_shown_car::HOOK,
    &arcade_level_1::HOOK,
    &arcade_level_2::HOOK,
    &arcade_level_3::HOOK,
    &arcade_level_4::HOOK,
    &arcade_level_5::HOOK,
    &arcade_event::HOOK,
    &arcade_level_end::HOOK,
    &arcade_finale::HOOK,
    &count_stage_pages::HOOK,
    &play_stage_speech::HOOK,
    &play_rally_speech::HOOK,
    &load_welcome_picture::HOOK,
    &previous_stage_page::HOOK,
    &next_stage_page::HOOK,
    &style_stage_item_a::HOOK,
    &style_stage_item_b::HOOK,
    &check_cheat_name::HOOK,
    &previous_stage_rows::HOOK,
    &next_stage_rows::HOOK,
    &previous_stage_rows_of_90::HOOK,
    &next_stage_rows_of_90::HOOK,
    &replay_item_c::HOOK,
    &inspection_frame::HOOK,
    &apply_screen_mode::HOOK,
    &show_car_specs::HOOK,
    &find_class_time::HOOK,
];
