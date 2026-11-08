//! The game's file primitives as Rust functions: `files::open_by_mode` (0x402DE8),
//! `files::read` (0x402E3D), `files::write` (0x402E57), `files::seek` (0x402E75),
//! `files::handle_size` (0x402EA9), `files::close` (0x402E23), and `files::file_time` /
//! `files::set_file_time` (a file's time read and set, for `files::copy_file`). Each was a few
//! instructions around one Win32 call; since phase 6 (2026-10-24) they go through
//! `std::fs::File` on the game's handle (`with_file`). Since phase 11 a handle is a key in
//! a table of `File`s (no Win32 handle): `close` takes the file out (whether it was there).
//! `read_file` and `write_file` have them written out inline; the buffered streams use these.

use crate::errors::BadInput;
use crate::files::resolve_path::resolve;
use crate::files::state::set_last_file_path;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// `CreateFileA`'s access bits and dispositions, as the mode table holds them.
const GENERIC_READ: u32 = 0x8000_0000;
const GENERIC_WRITE: u32 = 0x4000_0000;
const CREATE_NEW: u32 = 1;
const CREATE_ALWAYS: u32 = 2;
const OPEN_ALWAYS: u32 = 4;

/// The open files by the game's handle: 0x6000_0000 up in fours (never 0 or
/// `NO_HANDLE`). Each under its own lock, so threads' files do not wait on each other.
static FILES: Mutex<Option<HashMap<u32, Arc<Mutex<std::fs::File>>>>> = Mutex::new(None);
static NEXT_HANDLE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x6000_0000);

fn files() -> std::sync::MutexGuard<'static, Option<HashMap<u32, Arc<Mutex<std::fs::File>>>>> {
    FILES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

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
/// (the path's bytes read as Windows-1252, `files::host_path`); the handle is the file's
/// key in `FILES`, kept by the game until `close`. The sharing (others kept out while it is
/// open) is Windows' own and kept there. One difference: std makes "create or replace" an
/// open-or-create and a truncation, so a hidden or system file is replaced where
/// `CREATE_ALWAYS` with normal attributes would fail. `path` without its NUL.
pub fn open_by_mode(path: &[u8], mode: u32) -> Result<Option<u32>, BadInput> {
    if mode >= MODE_COUNT {
        return Err(BadInput("a file mode past the table"));
    }
    let resolved = resolve(path)?;
    set_last_file_path(Some(&resolved));
    let [access, disposition, share, _] = MODES[mode as usize];
    let mut options = std::fs::OpenOptions::new();
    options
        .read(access & GENERIC_READ != 0)
        .write(access & GENERIC_WRITE != 0);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
        options.share_mode(share).attributes(FILE_ATTRIBUTE_NORMAL);
    }
    #[cfg(not(windows))]
    let _ = share;
    match disposition {
        CREATE_NEW => options.create_new(true),
        CREATE_ALWAYS => options.create(true).truncate(true),
        OPEN_ALWAYS => options.create(true),
        _ => &mut options, // OPEN_EXISTING
    };
    #[cfg(feature = "remote")]
    crate::remote::note_open(&resolved);
    let Ok(file) = options.open(crate::files::host_path(&resolved)) else {
        return Ok(None);
    };
    let handle = NEXT_HANDLE.fetch_add(4, std::sync::atomic::Ordering::Relaxed);
    files()
        .get_or_insert_default()
        .insert(handle, Arc::new(Mutex::new(file)));
    Ok(Some(handle))
}

/// The game's handle as its `File` for one call; a handle that is none fails the call.
fn with_file<R>(
    file: u32,
    f: impl FnOnce(&mut std::fs::File) -> std::io::Result<R>,
) -> std::io::Result<R> {
    let open = files().get_or_insert_default().get(&file).cloned();
    let Some(open) = open else {
        return Err(std::io::ErrorKind::NotFound.into());
    };
    let mut file = open
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut file)
}

/// `read_slice` of `N` bytes into a typed buffer: (the bytes read, whether the read
/// succeeded).
pub fn read_into<const N: usize>(file: u32, into: &crate::session::Bytes<N>) -> (u32, bool) {
    let mut bytes = vec![0u8; N];
    let (got, ok) = read_slice(file, &mut bytes);
    for (k, &b) in bytes[..got as usize].iter().enumerate() {
        into.set(k, b);
    }
    (got, ok)
}

/// (the bytes read, whether the read succeeded): one `ReadFile` into a Rust buffer. A
/// short read is not a failure.
pub fn read_slice(file: u32, into: &mut [u8]) -> (u32, bool) {
    match with_file(file, |f| f.read(into)) {
        Ok(n) => (n as u32, true),
        Err(_) => (0, false),
    }
}

/// `read` into dwords (a typed store's, their bytes in order); what the file does not
/// reach is kept.
pub fn read_cells(file: u32, cells: &[crate::session::Dword]) -> (u32, bool) {
    let mut bytes: Vec<u8> = cells.iter().flat_map(|c| c.get().to_le_bytes()).collect();
    let got = read_slice(file, &mut bytes);
    for (c, b) in cells.iter().zip(bytes.as_chunks::<4>().0) {
        c.set(u32::from_le_bytes(*b));
    }
    got
}

/// `write` from dwords (a typed store's, their bytes in order).
pub fn write_cells(file: u32, cells: &[crate::session::Dword]) -> (u32, bool) {
    let bytes: Vec<u8> = cells.iter().flat_map(|c| c.get().to_le_bytes()).collect();
    write_slice(file, &bytes)
}

/// `read_to` into a store's field from its first byte (no address between).
pub fn read_into_store(
    file: u32,
    to: &dyn crate::pointers::Store,
    count: u32,
) -> Result<(u32, bool), BadInput> {
    let mut bytes = vec![0u8; count as usize];
    let (got, ok) = read_slice(file, &mut bytes);
    to.store(0, &bytes[..got as usize])?;
    Ok((got, ok))
}

/// `write_from` the first `count` bytes of a store's field.
pub fn write_from_store(
    file: u32,
    from: &dyn crate::pointers::Store,
    count: u32,
) -> Result<(u32, bool), BadInput> {
    let mut bytes = vec![0u8; count as usize];
    from.load(0, &mut bytes)?;
    Ok(write_slice(file, &bytes))
}

/// (the bytes written, whether all of them were): one `WriteFile`, from a Rust buffer.
pub fn write_slice(file: u32, from: &[u8]) -> (u32, bool) {
    let count = from.len() as u32;
    match with_file(file, |f| f.write(from)) {
        Ok(n) => (n as u32, n as u32 >= count),
        Err(_) => (0, false),
    }
}

/// `files::seek` (0x402E75): the new position, or None if the seek failed, as
/// `SetFilePointer` with no high dword answers: a negative distance from the start, a
/// position before the start, one past 32 bits and one of 0xFFFFFFFF (its failure value)
/// fail. Methods: 0 from the start, 1 from where it is, 2 from the end.
pub fn seek(file: u32, distance: i32, method: u32) -> Option<u32> {
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
pub fn handle_size(file: u32) -> Option<u32> {
    let size = with_file(file, |f| f.metadata()).ok()?.len() as u32;
    (size != u32::MAX).then_some(size)
}

/// `files::file_time`: the file's last write time as DOS date << 16 | time, or None.
pub fn file_time(file: u32) -> Option<u32> {
    let modified = with_file(file, |f| f.metadata().and_then(|m| m.modified())).ok()?;
    rc2k_core::files::dos_time::from_ticks(ticks_of(modified)?)
}

/// `files::set_file_time`: a DOS date << 16 | time put on the file as its last access and write
/// time; whether it worked (not for fields that are not a time).
pub fn set_file_time(file: u32, dos: u32) -> bool {
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

/// Whether the handle was closed. `g_lastFilePath` is none afterwards.
pub fn close(file: u32) -> bool {
    set_last_file_path(None);
    files().get_or_insert_default().remove(&file).is_some()
}

/// The game's file primitives as the core's file system: paths are the game's (a path's
/// resolving refuses, `g_lastFilePath` set as the original sets it), handles `FILES`'. A
/// path is its bytes without a NUL.
pub struct GameFiles<'a>(std::marker::PhantomData<&'a [u8]>);

impl GameFiles<'_> {
    pub fn new() -> Self {
        GameFiles(std::marker::PhantomData)
    }
}

impl<'a> rc2k_core::files::io::Files for GameFiles<'a> {
    type Path = &'a [u8];
    type Handle = u32;

    fn check(&mut self, path: &'a [u8]) -> Result<(), rc2k_core::Invalid> {
        resolve(path)
            .map(|_| ())
            .map_err(|e| rc2k_core::Invalid(e.0))
    }
    fn open(
        &mut self,
        path: &'a [u8],
        mode: rc2k_core::files::io::Mode,
    ) -> Result<Option<u32>, rc2k_core::Invalid> {
        open_by_mode(path, mode as u32).map_err(|e| rc2k_core::Invalid(e.0))
    }
    fn read(&mut self, file: u32, into: &mut [u8]) -> (u32, bool) {
        read_slice(file, into)
    }
    fn write(&mut self, file: u32, from: &[u8]) -> (u32, bool) {
        write_slice(file, from)
    }
    fn seek(&mut self, file: u32, distance: i32, from: rc2k_core::files::io::From) -> Option<u32> {
        seek(file, distance, from as u32)
    }
    fn size(&mut self, file: u32) -> Option<u32> {
        handle_size(file)
    }
    fn close(&mut self, file: u32) -> bool {
        close(file)
    }
    fn time(&mut self, file: u32) -> Option<u32> {
        file_time(file)
    }
    fn set_time(&mut self, file: u32, dos: u32) -> bool {
        set_file_time(file, dos)
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
