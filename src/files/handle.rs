//! The game's file primitives as Rust functions: `files::open_by_mode` (0x402DE8),
//! `files::read` (0x402E3D), `files::write` (0x402E57), `files::seek` (0x402E75),
//! `files::handle_size` (0x402EA9), `files::close` (0x402E23), and `sub_402F1C` /
//! `sub_402F4D` (a file's time read and set, for `files::copy_file`). Each was a few
//! instructions around one Win32 call; since phase 6 (2026-10-24) they go through
//! `std::fs::File` on the game's handle (`with_file`), all but `close`, which keeps
//! `CloseHandle` because the callers read whether it closed (std's drop says nothing).
//! `read_file` and `write_file` have them written out inline; the buffered streams use these.

use crate::errors::BadInput;
use crate::files::resolve_path::resolve_path_in_game;
use crate::files::state::LAST_FILE_PATH;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::{FromRawHandle, IntoRawHandle};
use std::time::{SystemTime, UNIX_EPOCH};
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Storage::FileSystem::{
    CREATE_ALWAYS, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, OPEN_ALWAYS,
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
/// Opened through `std::fs::OpenOptions` with the table's access, sharing and disposition
/// (the path's bytes read as Windows-1252, `files::ansi_path`); the handle is the file's
/// Win32 handle, kept by the game until `close`. One difference: std makes "create or
/// replace" an open-or-create and a truncation, so a hidden or system file is replaced
/// where `CREATE_ALWAYS` with normal attributes would fail.
pub unsafe fn open_by_mode(path: *const u8, mode: u32) -> Result<Option<u32>, BadInput> {
    if mode >= MODE_COUNT {
        return Err(BadInput("a file mode past the table"));
    }
    let resolved = unsafe { resolve_path_in_game(path)? };
    LAST_FILE_PATH.store(resolved as u32, std::sync::atomic::Ordering::Relaxed);
    let [access, disposition, share, _] = MODES[mode as usize];
    let mut options = std::fs::OpenOptions::new();
    // `write` only lets std accept a creating disposition; the access is the table's.
    options
        .access_mode(access)
        .share_mode(share)
        .attributes(FILE_ATTRIBUTE_NORMAL)
        .write(true);
    match disposition {
        CREATE_NEW => options.create_new(true),
        CREATE_ALWAYS => options.create(true).truncate(true),
        OPEN_ALWAYS => options.create(true),
        _ => &mut options, // OPEN_EXISTING
    };
    let file = options.open(unsafe { crate::files::ansi_path(resolved) });
    Ok(file.ok().map(|f| f.into_raw_handle() as u32))
}

/// The game's handle as a `File` for one call, not closed after it.
fn with_file<R>(file: u32, f: impl FnOnce(&mut std::fs::File) -> R) -> R {
    let mut file = std::mem::ManuallyDrop::new(unsafe {
        std::fs::File::from_raw_handle(file as usize as std::os::windows::io::RawHandle)
    });
    f(&mut file)
}

/// (the bytes read, whether the read succeeded): one `ReadFile`. A short read is not a
/// failure.
pub unsafe fn read(file: u32, buffer: *mut u8, count: u32) -> (u32, bool) {
    let into = unsafe { std::slice::from_raw_parts_mut(buffer, count as usize) };
    match with_file(file, |f| f.read(into)) {
        Ok(n) => (n as u32, true),
        Err(_) => (0, false),
    }
}

/// (the bytes written, whether all of them were): one `WriteFile`.
pub unsafe fn write(file: u32, buffer: *const u8, count: u32) -> (u32, bool) {
    let from = unsafe { std::slice::from_raw_parts(buffer, count as usize) };
    match with_file(file, |f| f.write(from)) {
        Ok(n) => (n as u32, n as u32 >= count),
        Err(_) => (0, false),
    }
}

/// `files::seek` (0x402E75): the new position, or None if the seek failed, as
/// `SetFilePointer` with no high dword answers: a negative distance from the start, a
/// position before the start, one past 32 bits and one of 0xFFFFFFFF (its failure value)
/// fail. Methods: 0 from the start, 1 from where it is, 2 from the end.
pub unsafe fn seek(file: u32, distance: i32, method: u32) -> Option<u32> {
    let to = match method {
        0 => SeekFrom::Start(u64::try_from(distance).ok()?),
        1 => SeekFrom::Current(distance.into()),
        2 => SeekFrom::End(distance.into()),
        _ => return None,
    };
    let position = with_file(file, |f| f.seek(to)).ok()?;
    (position < u32::MAX as u64).then_some(position as u32)
}

/// `files::handle_size` (0x402EA9): the file's size (its low dword, as `GetFileSize`
/// with no high dword gives it), or None if it could not be read or is 0xFFFFFFFF (which
/// `GetFileSize`'s failure looks like).
pub unsafe fn handle_size(file: u32) -> Option<u32> {
    let size = with_file(file, |f| f.metadata()).ok()?.len() as u32;
    (size != u32::MAX).then_some(size)
}

/// `sub_402F1C`: the file's last write time as DOS date << 16 | time, or None.
pub unsafe fn file_time(file: u32) -> Option<u32> {
    let modified = with_file(file, |f| f.metadata().and_then(|m| m.modified())).ok()?;
    rc2k_core::files::dos_time::from_ticks(ticks_of(modified)?)
}

/// `sub_402F4D`: a DOS date << 16 | time put on the file as its last access and write
/// time; whether it worked (not for fields that are not a time).
pub unsafe fn set_file_time(file: u32, dos: u32) -> bool {
    let Some(time) = rc2k_core::files::dos_time::to_ticks(dos).and_then(time_of) else {
        return false;
    };
    let times = std::fs::FileTimes::new()
        .set_accessed(time)
        .set_modified(time);
    with_file(file, |f| f.set_times(times)).is_ok()
}

/// Ticks (100 ns) from 1601-01-01 to 1970-01-01.
const TICKS_1970: u64 = 116_444_736_000_000_000;

/// A time as a Windows file time's ticks, None before 1601.
fn ticks_of(time: SystemTime) -> Option<u64> {
    let ticks = |d: std::time::Duration| (d.as_nanos() / 100) as u64;
    match time.duration_since(UNIX_EPOCH) {
        Ok(after) => TICKS_1970.checked_add(ticks(after)),
        Err(before) => TICKS_1970.checked_sub(ticks(before.duration())),
    }
}

fn time_of(ticks: u64) -> Option<SystemTime> {
    let nanos = |t: u64| std::time::Duration::from_nanos(t * 100);
    if ticks >= TICKS_1970 {
        UNIX_EPOCH.checked_add(nanos(ticks - TICKS_1970))
    } else {
        UNIX_EPOCH.checked_sub(nanos(TICKS_1970 - ticks))
    }
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
        unsafe { file_time(file) }
    }
    fn set_time(&mut self, file: u32, dos: u32) -> bool {
        unsafe { set_file_time(file, dos) }
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
