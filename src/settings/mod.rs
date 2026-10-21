//! The game's settings files (`var\ral.cfg` and the others of its kind): lines of
//! `key=value` or `key value`, read whole into a scratch block and searched key by key.

use crate::framework::{GameHook, rva};

pub mod apply_feature_switches;
pub mod copy_value;
pub mod find_key;
pub mod parse_number;
pub mod read_driver_names;
pub mod read_settings;
pub mod read_text;

pub static HOOKS: &[&dyn GameHook] = &[
    &read_text::HOOK,
    &find_key::HOOK,
    &copy_value::HOOK,
    &parse_number::HOOK,
    &read_settings::HOOK,
    &apply_feature_switches::HOOK,
    &read_driver_names::HOOK,
];

/// The block a settings file is read into: 0x10000 bytes, up to `files::numbered_path`'s
/// buffer. (The PNG reader and `files::read_first_match` use its start too.)
pub const TEXT_RVA: usize = rva(0x93E8B0);
pub const TEXT_ROOM: u32 = rc2k_core::files::settings::TEXT_ROOM;
