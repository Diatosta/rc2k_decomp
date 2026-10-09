use crate::framework::GameHook;

pub mod append_string_by_id;
pub mod append_temperature;
pub mod bin_append;
pub mod expand_entry;
pub mod format_u32;
pub mod hex_append;
pub mod str_append;
pub mod str_append_indexed;
pub mod string_table;
pub mod utils;

pub static HOOKS: &[&dyn GameHook] = &[
    &str_append::HOOK,
    &hex_append::HOOK,
    &str_append_indexed::HOOK,
    &append_string_by_id::HOOK,
    &expand_entry::HOOK,
    &format_u32::HOOK,
    &append_temperature::HOOK,
    &bin_append::HOOK,
];
