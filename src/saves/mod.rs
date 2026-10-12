//! Saved games.

use crate::framework::GameHook;

pub mod allocate_best_times;
pub mod gather_savegame;
pub mod list_car_numbers;
pub mod load_best_times;
pub mod load_options;
pub mod load_savegame;
pub mod read_replay_slot;
pub mod read_savegame_slot;
pub mod rebuild_car_lists;
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
];
