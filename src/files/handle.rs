//! The game's five file primitives as Rust functions: `files::open_by_mode` (0x402DE8),
//! `files::read` (0x402E3D), `files::write` (0x402E57), `files::seek` (0x402E75),
//! `files::close` (0x402E23). They are
//! not hooked (section 10 of CLAUDE.md): each is a few instructions around one Win32 call.
//! `read_file` and `write_file` have them written out inline; the buffered streams use these.

use crate::errors::BadInput;
use crate::files::read_file::LAST_FILE_PATH_RVA;
use crate::files::resolve_path::resolve_path_in_game;
use crate::framework::{game_read, game_write, rva};
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileA, GetFileSize, ReadFile, SetFilePointer, WriteFile,
};

/// The file mode table: 16 bytes per mode, +0 access, +4 disposition, +8 share.
const MODES_RVA: usize = rva(0x4E06EA);
/// 0 read an existing file, 1 create or replace, 2 create only, 3 read and write an existing
/// file, 4 open or create for writing.
pub const MODE_COUNT: u32 = 5;

pub const NO_HANDLE: u32 = u32::MAX;

/// The handle, or None if the file could not be opened. `g_lastFilePath` is the resolved
/// path afterwards, either way.
/// Refuses what `resolve_path` refuses and a mode the table does not have.
pub unsafe fn open_by_mode(path: *const u8, mode: u32) -> Result<Option<u32>, BadInput> {
    if mode >= MODE_COUNT {
        return Err(BadInput("a file mode past the table"));
    }
    unsafe {
        let resolved = resolve_path_in_game(path)?;
        game_write(LAST_FILE_PATH_RVA, resolved as u32);
        let entry = MODES_RVA + mode as usize * 16;
        let file = CreateFileA(
            resolved,
            game_read(entry),
            game_read(entry + 8),
            std::ptr::null(),
            game_read(entry + 4),
            0x80,
            0 as _,
        ) as u32;
        Ok((file != NO_HANDLE).then_some(file))
    }
}

/// (the bytes read, whether `ReadFile` succeeded). A short read is not a failure.
pub unsafe fn read(file: u32, buffer: *mut u8, count: u32) -> (u32, bool) {
    let mut n = 0u32;
    let ok = unsafe {
        ReadFile(
            file as _,
            buffer.cast(),
            count,
            &mut n,
            std::ptr::null_mut(),
        )
    } != 0;
    (n, ok)
}

/// (the bytes written, whether all of them were).
pub unsafe fn write(file: u32, buffer: *const u8, count: u32) -> (u32, bool) {
    let mut n = 0u32;
    let ok = unsafe {
        WriteFile(
            file as _,
            buffer.cast(),
            count,
            &mut n,
            std::ptr::null_mut(),
        )
    } != 0;
    (n, ok && n >= count)
}

/// `files::seek` (0x402E75): the new position, or None if `SetFilePointer` failed. Methods:
/// 0 from the start, 1 from where it is, 2 from the end.
pub unsafe fn seek(file: u32, distance: i32, method: u32) -> Option<u32> {
    let position = unsafe { SetFilePointer(file as _, distance, std::ptr::null_mut(), method) };
    (position != u32::MAX).then_some(position)
}

/// `files::handle_size` (0x402EA9): the file's size, or None if `GetFileSize` failed (which
/// a file of 0xFFFFFFFF bytes also looks like).
pub unsafe fn handle_size(file: u32) -> Option<u32> {
    let size = unsafe { GetFileSize(file as _, std::ptr::null_mut()) };
    (size != u32::MAX).then_some(size)
}

/// Whether the handle was closed. `g_lastFilePath` is 0 afterwards.
pub unsafe fn close(file: u32) -> bool {
    unsafe {
        game_write(LAST_FILE_PATH_RVA, 0u32);
        CloseHandle(file as _) != 0
    }
}
