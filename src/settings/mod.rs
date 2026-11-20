//! The game's settings files (`var\ral.cfg` and the others of its kind): lines of
//! `key=value` or `key value`, read whole into a scratch block and searched key by key.

pub(crate) mod parse_number;
pub(crate) mod read_driver_names;
pub(crate) mod read_settings;
pub(crate) mod read_text;

/// The block a settings file is read into: 0x10000 bytes, up to `files::numbered_path`'s
/// buffer. (The PNG reader and `files::read_first_match` use its start too.)
pub const TEXT_ROOM: u32 = rc2k_core::files::settings::TEXT_ROOM;
