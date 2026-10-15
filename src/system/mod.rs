//! Operating system services other than files.

use crate::framework::GameHook;

pub mod check_developer_machine;
pub mod find_entry;
pub mod keep_start_time;
pub mod load_game_data;
pub mod main_loop;
pub mod next_argument;
pub mod packed_local_time;
pub mod read_command_line;
pub mod restore_window;
pub mod sleep;
pub mod tick_count;
pub mod zero_memory;

pub static HOOKS: &[&dyn GameHook] = &[
    &keep_start_time::HOOK,
    &packed_local_time::HOOK,
    &tick_count::HOOK,
    &sleep::HOOK,
    &zero_memory::HOOK,
    &find_entry::HOOK,
    &next_argument::HOOK,
    &read_command_line::HOOK,
    &check_developer_machine::HOOK,
    &load_game_data::HOOK,
    &main_loop::HOOK,
];
