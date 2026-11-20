//! What the game keeps between sessions: the options, the best times and scores, the saved
//! championships, and the lists of saved games and replays the menus show. Read at start-up;
//! written when the player leaves, saves a game or beats a time. Its state is the `Saves` owner
//! (`owner.rs`).

pub(crate) mod allocate_best_times;
pub(crate) mod best_times;
mod best_times_store;
pub(crate) mod default_best_scores;
pub(crate) mod default_championship_times;
pub(crate) mod default_checkpoint_times;
pub(crate) mod default_class_times;
pub(crate) mod default_stage_times;
pub(crate) mod default_stage_times_alt;
pub(crate) mod default_times;
pub(crate) mod delete_all;
pub(crate) mod drivers;
pub(crate) mod free_best_times;
pub(crate) mod gather_savegame;
mod keep_player_settings;
pub(crate) mod list_car_numbers;
pub(crate) mod load_best_times;
mod load_options;
pub(crate) mod load_savegame;
pub(crate) mod options_files;
pub(crate) mod options_state;
pub(crate) mod owner;
mod preview_store;
pub(crate) mod read_replay_slot;
pub(crate) mod read_savegame_slot;
mod rebuild_car_lists;
pub(crate) mod save_best_times;
pub(crate) mod save_game_data;
mod save_options;
pub(crate) mod write_savegame_slot;
