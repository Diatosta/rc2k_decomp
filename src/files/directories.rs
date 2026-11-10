//! The game's folder primitives as Rust functions (as `handle.rs`'s): each is
//! `push ecx; push edx; push <path>; call <Win32>; cmp eax, 1; pop edx; pop ecx; retn`, so
//! eax the BOOL, carry set when it failed, zero set when it is 1, everything else kept:
//! - `files::set_current_directory` (0x403105, edx the path): `SetCurrentDirectoryA`;
//! - `files::make_directory` (0x403113, edx): `CreateDirectoryA` (no security attributes);
//! - 0x403123 (proposed `files::remove_directory`, edx): `RemoveDirectoryA`, the
//!   uninstaller's (`install::uninstall`);
//! - `files::delete_file` (0x402E8C, eax): `DeleteFileA`.
//!
//! The paths are used as they are (not through `files::resolve_path`).

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
