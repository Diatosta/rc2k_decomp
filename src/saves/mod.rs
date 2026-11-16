//! What the game keeps between sessions: the options, the best times and scores, the saved
//! championships, and the lists of saved games and replays the menus show. Read at start-up;
//! written when the player leaves, saves a game or beats a time. Its state is the `Saves` owner
//! (`owner.rs`).

pub mod allocate_best_times;
pub mod best_times;
pub mod best_times_store;
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
pub mod keep_player_settings;
pub mod list_car_numbers;
pub mod load_best_times;
pub mod load_options;
pub mod load_savegame;
pub mod options_files;
pub mod options_state;
pub mod owner;
pub mod preview_store;
pub mod read_replay_slot;
pub mod read_savegame_slot;
pub mod rebuild_car_lists;
pub mod save_best_times;
pub mod save_game_data;
pub mod save_options;
pub mod write_savegame_slot;
