//! Operating system services other than files.

pub mod check_developer_machine;
pub mod find_entry;
pub mod find_entry_with_bits;
pub mod find_running_game;
pub mod keep_start_time;
pub mod load_game_data;
pub mod main_loop;
pub mod next_entry;
pub mod packed_local_time;
pub mod read_command_line;
pub mod read_keyboard_type;
pub mod read_volume_serial;
pub mod read_windows_version;
pub mod restore_error_mode;
pub mod restore_window;
pub mod same_guid;
pub mod set_error_mode;
pub mod set_priority_class;
pub mod sleep;
pub mod start_texts;
pub mod test_processor;
pub mod tick_count;
pub mod wait_or_key;
pub mod zero_memory;
