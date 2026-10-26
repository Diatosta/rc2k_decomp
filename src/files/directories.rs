//! The game's folder primitives as Rust functions, not hooked (as `handle.rs`'s): each is
//! `push ecx; push edx; push <path>; call <Win32>; cmp eax, 1; pop edx; pop ecx; retn`, so
//! eax the BOOL, carry set when it failed, zero set when it is 1, everything else kept:
//! - `files::set_current_directory` (0x403105, edx the path): `SetCurrentDirectoryA`;
//! - `files::make_directory` (0x403113, edx): `CreateDirectoryA` (no security attributes);
//! - `sub_403123` (proposed `files::remove_directory`, edx): `RemoveDirectoryA`, the
//!   uninstaller's (`install::uninstall`);
//! - `files::delete_file` (0x402E8C, eax): `DeleteFileA`.
//!
//! The paths are used as they are (not through `files::resolve_path`).

// Phase 6 (2026-10-24): `std::fs` / `std::env` in place of the Win32 calls (`SetCurrentDirectoryA`,
// `CreateDirectoryA`, `RemoveDirectoryA`, `DeleteFileA`): the same cases fail (a folder
// already there, one not empty, a read-only file), the answer 1 or 0 as the BOOL was.
use crate::errors::BadInput;
use crate::files::ansi_path;
use crate::pointers::GamePtr;

fn path(path: *const u8) -> Result<std::path::PathBuf, BadInput> {
    ansi_path(GamePtr::new(path as u32))
}

pub fn set_current_directory(at: *const u8) -> Result<u32, BadInput> {
    Ok(std::env::set_current_dir(path(at)?).is_ok() as u32)
}

pub fn make_directory(at: *const u8) -> Result<u32, BadInput> {
    Ok(std::fs::create_dir(path(at)?).is_ok() as u32)
}

pub fn remove_directory(at: *const u8) -> Result<u32, BadInput> {
    Ok(std::fs::remove_dir(path(at)?).is_ok() as u32)
}

pub fn delete_file(at: *const u8) -> Result<u32, BadInput> {
    Ok(std::fs::remove_file(path(at)?).is_ok() as u32)
}
