//! The menu system's state.

use crate::framework::GameHook;

pub mod add_text_length;
pub mod backspace_savegame_name;
pub mod backup_text;
pub mod begin_replay_name;
pub mod begin_savegame_name;
pub mod championship_standings_loop;
pub mod class_rally_results_loop;
pub mod class_stage_results_loop;
pub mod clear_text;
pub mod edit_replay_name;
pub mod edit_savegame_name;
pub mod go_back;
pub mod handle_keys;
pub mod handle_yes_no;
pub mod item_at;
pub mod move_highlight_down;
pub mod move_highlight_left;
pub mod move_highlight_right;
pub mod move_highlight_up;
pub mod name_lists;
pub mod navigation;
pub mod overall_rally_results_loop;
pub mod overall_stage_results_loop;
pub mod rally_stages_loop;
pub mod restore_text;
pub mod run_draw_list;
pub mod run_item_action;
pub mod run_list;
pub mod screen_loop;
pub mod text_entry;
pub mod update_blink;
pub mod update_frame_clock;

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
];
