use crate::framework::GameHook;

pub mod append_directory;
pub mod append_file_name;
pub mod close_stream;
pub mod copy_file;
pub mod directories;
pub mod drive_free_space_kb;
pub mod drive_type;
pub mod file_exists;
pub mod file_size;
pub mod handle;
pub mod is_cd_drive;
pub mod numbered_path;
pub mod open_stream;
pub mod read_file;
pub mod read_first_match;
pub mod read_level_file;
pub mod read_packed_file;
pub mod read_stream_block;
pub mod read_stream_byte;
pub mod read_whole_file;
pub mod resolve_path;
pub mod stream;
pub mod swap_missing_extension;
pub mod write_file;

pub mod write_stream_block;
pub mod write_stream_byte;

pub static HOOKS: &[&dyn GameHook] = &[
    &resolve_path::HOOK,
    &numbered_path::HOOK,
    &read_file::HOOK,
    &read_first_match::HOOK,
    &read_whole_file::HOOK,
    &drive_type::HOOK,
    &write_file::HOOK,
    &file_exists::HOOK,
    &file_size::HOOK,
    &swap_missing_extension::HOOK,
    &open_stream::HOOK,
    &close_stream::HOOK,
    &read_stream_byte::HOOK,
    &read_stream_block::HOOK,
    &read_packed_file::HOOK,
    &read_level_file::HOOK,
    &is_cd_drive::HOOK,
    &append_directory::HOOK,
    &append_file_name::HOOK,
    &copy_file::HOOK,
    &drive_free_space_kb::HOOK,
    &write_stream_byte::HOOK,
    &write_stream_block::HOOK,
];
