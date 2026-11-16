//! The program's outer shell and its small services: the main loop that chooses between the
//! installer and the game (`main_loop`), the command line, the clocks, sleeping and waiting, and
//! what the game wants to know about the machine it runs on. Its state is the `System` owner
//! (`owner.rs`): the session's flags (quitting, window active), the clocks, the machine's facts.

pub mod checked_reads;
pub mod keep_start_time;
pub mod load_game_data;
pub mod main_loop;
pub mod owner;
pub mod packed_local_time;
pub mod pause;
pub mod read_command_line;
pub mod read_windows_version;
pub mod restore_window;
pub mod sleep;
pub mod start_texts;
pub mod test_processor;
pub mod tick_count;
pub mod wait_or_key;
