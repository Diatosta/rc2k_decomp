use crate::framework::GameHook;

pub mod append_clock;
pub mod append_distance_2dp;
pub mod append_fraction;
pub mod append_percent;
pub mod append_speed;
pub mod append_string_by_id;
pub mod append_temperature;
pub mod append_time;
pub mod append_u32;
pub mod bin_append;
pub mod expand_entry;
pub mod format_u32;
pub mod hex_append;
pub mod str_append;
pub mod str_append_indexed;
pub mod string_table;
pub mod text_from_ids;
pub mod units;
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
    &append_u32::HOOK,
    &append_percent::HOOK,
    &append_fraction::HOOK,
    &append_clock::HOOK,
    &append_time::HOOK,
    &append_distance_2dp::HOOK,
    &append_speed::HOOK,
    &text_from_ids::HOOK,
];
