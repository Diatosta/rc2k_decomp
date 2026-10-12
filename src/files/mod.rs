use crate::framework::GameHook;

pub mod close_stream;
pub mod drive_type;
pub mod file_exists;
pub mod file_size;
pub mod handle;
pub mod numbered_path;
pub mod open_stream;
pub mod read_file;
pub mod read_first_match;
pub mod read_stream_block;
pub mod read_stream_byte;
pub mod resolve_path;
pub mod stream;
pub mod swap_missing_extension;
pub mod write_file;

pub static HOOKS: &[&dyn GameHook] = &[
    &resolve_path::HOOK,
    &numbered_path::HOOK,
    &read_file::HOOK,
    &read_first_match::HOOK,
    &drive_type::HOOK,
    &write_file::HOOK,
    &file_exists::HOOK,
    &file_size::HOOK,
    &swap_missing_extension::HOOK,
    &open_stream::HOOK,
    &close_stream::HOOK,
    &read_stream_byte::HOOK,
    &read_stream_block::HOOK,
];
