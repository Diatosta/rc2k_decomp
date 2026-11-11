//! The game's folder primitives as Rust functions (as `handle.rs`'s): each calls the Win32
//! function on the path and gives back whether it worked (the original returned the BOOL, with
//! carry set when it failed and zero set when it is 1):
//! - `files::set_current_directory`: `SetCurrentDirectoryA`;
//! - `files::make_directory`: `CreateDirectoryA` (no security attributes);
//! - `files::remove_directory`: `RemoveDirectoryA`, the uninstaller's (`install::uninstall`);
//! - `files::delete_file`: `DeleteFileA`.

// `std::fs` / `std::env` stand in for the Win32 calls (`SetCurrentDirectoryA`,
// `CreateDirectoryA`, `RemoveDirectoryA`, `DeleteFileA`): the same cases fail (a folder
// already there, one not empty, a read-only file), the answer 1 or 0 as the BOOL was.

/// `SetCurrentDirectoryA`: whether it worked.
pub fn set_current_directory(path: &[u8]) -> bool {
    std::env::set_current_dir(crate::files::host_path(path)).is_ok()
}

pub fn make_directory(path: &[u8]) -> bool {
    std::fs::create_dir(crate::files::host_path(path)).is_ok()
}

pub fn remove_directory(path: &[u8]) -> bool {
    std::fs::remove_dir(crate::files::host_path(path)).is_ok()
}

pub fn delete_file(path: &[u8]) -> bool {
    std::fs::remove_file(crate::files::host_path(path)).is_ok()
}
