use crate::framework::GameHook;

mod append_string_by_id;
mod digit_step;
mod expand_entry;
mod hex_append;
mod str_append;
mod str_append_indexed;
mod string_table;
mod utils;

pub static HOOKS: &[&dyn GameHook] = &[
    &str_append::HOOK,
    &hex_append::HOOK,
    &str_append_indexed::HOOK,
    &append_string_by_id::HOOK,
    &expand_entry::HOOK,
    &digit_step::HOOK,
];
