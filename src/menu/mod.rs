//! The menu system's state.

use crate::framework::GameHook;

pub mod add_text_length;
pub mod advanced_audio_loop;
pub mod audio_setup_loop;
pub mod backspace_savegame_name;
pub mod backup_text;
pub mod begin_replay_name;
pub mod begin_savegame_name;
pub mod best_class_scores_loop;
pub mod best_overall_scores_loop;
pub mod best_times_loop;
pub mod championship_standings_loop;
pub mod class_championship_standings_loop;
pub mod class_rally_results_loop;
pub mod class_stage_results_loop;
pub mod clear_text;
pub mod edit_replay_name;
pub mod edit_savegame_name;
pub mod go_back;
pub mod handle_keys;
pub mod handle_yes_no;
pub mod inspection_area_loop;
pub mod item_at;
pub mod leave_question_loop;
pub mod move_highlight_down;
pub mod move_highlight_left;
pub mod move_highlight_right;
pub mod move_highlight_up;
pub mod name_lists;
pub mod navigation;
pub mod new_simulation_loop;
pub mod overall_rally_results_loop;
pub mod overall_stage_results_loop;
pub mod rally_stages_loop;
pub mod restore_text;
pub mod run_draw_list;
pub mod run_item_action;
pub mod run_list;
pub mod screen_47814c_loop;
pub mod screen_4783f6_loop;
pub mod screen_4786aa_loop;
pub mod screen_478954_loop;
pub mod screen_478c08_loop;
pub mod screen_478eb2_loop;
pub mod screen_479166_loop;
pub mod screen_479410_loop;
pub mod screen_483163_loop;
pub mod screen_48558d_loop;
pub mod screen_488f4c_loop;
pub mod screen_loop;
pub mod select_players_loop;
pub mod text_entry;
pub mod time_trial_stage_details_loop;
pub mod update_blink;
pub mod update_frame_clock;
pub mod view_best_scores_loop;
pub mod view_best_times_loop;
pub mod view_courses_loop;
pub mod visual_options_loop;
pub mod visual_preferences_loop;

pub static HOOKS: &[&dyn GameHook] = &[
    &go_back::HOOK,
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
    &screen_48558d_loop::HOOK,
    &screen_488f4c_loop::HOOK,
    &best_times_loop::HOOK,
    &class_championship_standings_loop::HOOK,
    &best_class_scores_loop::HOOK,
    &best_overall_scores_loop::HOOK,
    &audio_setup_loop::HOOK,
    &visual_preferences_loop::HOOK,
    &select_players_loop::HOOK,
    &view_best_times_loop::HOOK,
    &view_courses_loop::HOOK,
    &view_best_scores_loop::HOOK,
    &new_simulation_loop::HOOK,
];
