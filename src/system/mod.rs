//! The program's outer shell and its small services: the main loop that chooses between the
//! installer and the game (`main_loop`), the command line, the waiting that pumps the window's
//! events, and what the game wants to know about the machine it runs on. Its state is the
//! `System` owner (`owner.rs`): the session's flags (quitting, window active), the picture clock,
//! the machine's facts. The time itself (`tick_count`, `sleep`) is `crate::clock`.

pub(crate) mod checked_reads;
pub(crate) mod detect_processor;
pub(crate) mod keep_start_time;
pub(crate) mod load_game_data;
pub(crate) mod main_loop;
pub(crate) mod owner;
pub(crate) mod pause;
pub(crate) mod read_command_line;
pub(crate) mod read_windows_version;
pub(crate) mod restore_window;
mod start_texts;
pub(crate) mod wait_or_key;
