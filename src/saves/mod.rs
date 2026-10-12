//! Saved games.

use crate::framework::GameHook;

pub mod allocate_best_times;
pub mod default_best_scores;
pub mod default_championship_times;
pub mod default_checkpoint_times;
pub mod default_class_times;
pub mod default_stage_times;
pub mod default_stage_times_alt;
pub mod default_times;
pub mod drivers;
pub mod free_best_times;
pub mod gather_savegame;
pub mod list_car_numbers;
pub mod load_best_times;
pub mod load_options;
pub mod load_savegame;
pub mod read_replay_slot;
pub mod read_savegame_slot;
pub mod rebuild_car_lists;
pub mod save_best_times;
pub mod save_game_data;
pub mod save_options;
pub mod write_savegame_slot;

pub static HOOKS: &[&dyn GameHook] = &[
    &gather_savegame::HOOK,
    &write_savegame_slot::HOOK,
    &read_savegame_slot::HOOK,
    &load_savegame::HOOK,
    &read_replay_slot::HOOK,
    &rebuild_car_lists::HOOK,
    &load_options::HOOK,
    &allocate_best_times::HOOK,
    &load_best_times::HOOK,
    &list_car_numbers::HOOK,
    &default_best_scores::HOOK,
    &default_class_times::HOOK,
    &default_championship_times::HOOK,
    &default_checkpoint_times::HOOK,
    &default_stage_times::HOOK,
    &default_stage_times_alt::HOOK,
    &save_game_data::HOOK,
    &save_options::HOOK,
    &save_best_times::HOOK,
    &free_best_times::HOOK,
];
