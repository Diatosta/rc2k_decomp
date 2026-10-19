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

use windows_sys::Win32::Storage::FileSystem::{CreateDirectoryA, DeleteFileA, RemoveDirectoryA};
use windows_sys::Win32::System::Environment::SetCurrentDirectoryA;

pub const SET_CURRENT_DIRECTORY: usize = 0x403105;
pub const MAKE_DIRECTORY: usize = 0x403113;
pub const REMOVE_DIRECTORY: usize = 0x403123;
pub const DELETE_FILE: usize = 0x402E8C;

pub unsafe fn set_current_directory(path: *const u8) -> u32 {
    unsafe { SetCurrentDirectoryA(path) as u32 }
}

pub unsafe fn make_directory(path: *const u8) -> u32 {
    unsafe { CreateDirectoryA(path, std::ptr::null()) as u32 }
}

pub unsafe fn remove_directory(path: *const u8) -> u32 {
    unsafe { RemoveDirectoryA(path) as u32 }
}

pub unsafe fn delete_file(path: *const u8) -> u32 {
    unsafe { DeleteFileA(path) as u32 }
}

/// The one of the four at `place` (its IDA address): the BOOL it gives.
pub unsafe fn by_place(place: usize, path: *const u8) -> u32 {
    unsafe {
        match place {
            SET_CURRENT_DIRECTORY => set_current_directory(path),
            MAKE_DIRECTORY => make_directory(path),
            REMOVE_DIRECTORY => remove_directory(path),
            _ => delete_file(path),
        }
    }
}
