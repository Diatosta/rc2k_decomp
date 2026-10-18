use crate::framework::GameHook;

pub mod append_axis_name;
pub mod append_clock;
pub mod append_clock_padded;
pub mod append_distance_2dp;
pub mod append_fraction;
pub mod append_length;
pub mod append_percent;
pub mod append_power;
pub mod append_speed;
pub mod append_string_by_id;
pub mod append_temperature;
pub mod append_time;
pub mod append_torque;
pub mod append_u32;
pub mod append_weight;
pub mod bin_append;
pub mod car_units;
pub mod compile_lexicon;
pub mod copy_printable;
pub mod expand_entry;
pub mod format_seal_code;
pub mod format_u32;
pub mod hex_append;
pub mod lexicon;
pub mod load_language;
pub mod same_text;
pub mod str_append;
pub mod str_append_indexed;
pub mod string_table;
pub mod text_from_ids;
pub mod text_length;
pub mod units;
pub mod utils;

pub static HOOKS: &[&dyn GameHook] = &[
    &text_length::HOOK,
    &compile_lexicon::HOOK,
    &load_language::HOOK,
    &same_text::HOOK,
    &copy_printable::HOOK,
    &str_append::HOOK,
    &hex_append::HOOK,
    &format_seal_code::HOOK,
    &append_axis_name::HOOK,
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
    &append_clock_padded::HOOK,
    &append_time::HOOK,
    &append_distance_2dp::HOOK,
    &append_speed::HOOK,
    &text_from_ids::HOOK,
    &append_length::HOOK,
    &append_power::HOOK,
    &append_torque::HOOK,
    &append_weight::HOOK,
];
