//! The game's five file primitives as Rust functions: `files::open_by_mode` (0x402DE8),
//! `files::read` (0x402E3D), `files::write` (0x402E57), `files::seek` (0x402E75),
//! `files::close` (0x402E23). They are
//! not hooked (section 10 of CLAUDE.md): each is a few instructions around one Win32 call.
//! `read_file` and `write_file` have them written out inline; the buffered streams use these.

use crate::errors::BadInput;
use crate::files::resolve_path::resolve_path_in_game;
use crate::files::state::LAST_FILE_PATH;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileA, GetFileSize, ReadFile, SetFilePointer, WriteFile,
};

/// The file mode table (0x4E06EA in the image, 16 bytes per mode: +0 access, +4 disposition,
/// +8 share, +12 0): `tests::modes_are_the_games`.
const MODES: [[u32; 4]; MODE_COUNT as usize] = [
    [0x8000_0000, 3, 1, 0],
    [0x4000_0000, 2, 0, 0],
    [0x4000_0000, 1, 0, 0],
    [0xC000_0000, 3, 0, 0],
    [0x4000_0000, 4, 0, 0],
];
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
        LAST_FILE_PATH.store(resolved as u32, std::sync::atomic::Ordering::Relaxed);
        let entry = MODES[mode as usize];
        let file = CreateFileA(
            resolved,
            entry[0],
            entry[2],
            std::ptr::null(),
            entry[1],
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
        LAST_FILE_PATH.store(0, std::sync::atomic::Ordering::Relaxed);
        CloseHandle(file as _) != 0
    }
}

/// The game's file primitives as the core's file system: paths are the game's (a path's
/// resolving refuses, `g_lastFilePath` set as the original sets it), handles Win32's.
pub struct GameFiles(());

impl GameFiles {
    /// # Safety
    /// Every path handed to it must be a NUL-ended text in game memory, which stays there
    /// (`g_lastFilePath` may keep pointing at it or at what resolving made of it).
    pub unsafe fn new() -> Self {
        GameFiles(())
    }
}

impl rc2k_core::files::io::Files for GameFiles {
    type Path = *const u8;
    type Handle = u32;

    #[allow(clippy::not_unsafe_ptr_arg_deref)] // `GameFiles::new`'s contract covers paths
    fn check(&mut self, path: *const u8) -> Result<(), rc2k_core::Invalid> {
        unsafe { resolve_path_in_game(path) }
            .map(|_| ())
            .map_err(|e| rc2k_core::Invalid(e.0))
    }
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // `GameFiles::new`'s contract covers paths
    fn open(
        &mut self,
        path: *const u8,
        mode: rc2k_core::files::io::Mode,
    ) -> Result<Option<u32>, rc2k_core::Invalid> {
        unsafe { open_by_mode(path, mode as u32) }.map_err(|e| rc2k_core::Invalid(e.0))
    }
    fn read(&mut self, file: u32, into: &mut [u8]) -> (u32, bool) {
        unsafe { read(file, into.as_mut_ptr(), into.len() as u32) }
    }
    fn write(&mut self, file: u32, from: &[u8]) -> (u32, bool) {
        unsafe { write(file, from.as_ptr(), from.len() as u32) }
    }
    fn seek(&mut self, file: u32, distance: i32, from: rc2k_core::files::io::From) -> Option<u32> {
        unsafe { seek(file, distance, from as u32) }
    }
    fn size(&mut self, file: u32) -> Option<u32> {
        unsafe { handle_size(file) }
    }
    fn close(&mut self, file: u32) -> bool {
        unsafe { close(file) }
    }
    fn time(&mut self, file: u32) -> Option<u32> {
        unsafe { crate::files::copy_file::file_time(file) }
    }
    fn set_time(&mut self, file: u32, dos: u32) -> bool {
        unsafe { crate::files::copy_file::set_file_time(file, dos) }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn modes_are_the_games() {
        let bytes: Vec<u8> = super::MODES
            .iter()
            .flatten()
            .flat_map(|w| w.to_le_bytes())
            .collect();
        assert_eq!(bytes, crate::moved::image_bytes(0x4E06EA, bytes.len()));
    }
}
