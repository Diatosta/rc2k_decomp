//! `files::copy_file` (0x402C28): a file copied, its time with it: the source opened to
//! read (`files::open_by_mode` 0), the destination created (mode 1), 64 KB at a time
//! through `g_pngInfo` (the PNG reader's buffer) until a read gives 0 bytes, then the
//! source's last write time (`sub_402F1C`: `GetFileTime`, `FileTimeToDosDateTime`) put on
//! the destination as its last access and last write time (`sub_402F4D`:
//! `DosDateTimeToFileTime`, `SetFileTime`; so to two seconds). A failed read, a short or
//! failed write or a failed time ends it; both files are closed (the destination first).
//! `sub_402F1C` and `sub_402F4D` are not hooked (each a Win32 call or two, as
//! `files/handle.rs`'s): `file_time` and `set_file_time` here.
//!
//! In:  eax = the source's path, ebx = the destination's (both through `resolve_path`)
//! Out: every register kept (`pusha` .. `popa`); carry clear if it was all done, set if
//!      not (zero set if done)
//!
//! Five callers: `install::run_after_exit`, `sub_4139E5`, `sub_41EA1F`, `sub_421B42`,
//! `sub_4599AD`. Refuses (before anything) what `open_by_mode` refuses (a resolved path
//! past its buffer).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{game_addr, rva};
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::Storage::FileSystem::{GetFileTime, SetFileTime};
use windows_sys::Win32::System::WindowsProgramming::{
    DosDateTimeToFileTime, FileTimeToDosDateTime,
};

/// g_pngInfo: the 64 KB the copy goes through.
const BUFFER: usize = 0x93E8B0;
const CHUNK: u32 = 0x1_0000;

/// `sub_402F1C`: the file's last write time as DOS date << 16 | time, or None.
pub unsafe fn file_time(file: u32) -> Option<u32> {
    let mut ft = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let (mut date, mut time) = (0u16, 0u16);
    unsafe {
        if GetFileTime(
            file as _,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut ft,
        ) == 0
        {
            return None;
        }
        if FileTimeToDosDateTime(&ft, &mut date, &mut time) == 0 {
            return None;
        }
    }
    Some((date as u32) << 16 | time as u32)
}

/// `sub_402F4D`: a DOS date << 16 | time put on the file as its last access and write
/// time; whether it worked.
pub unsafe fn set_file_time(file: u32, dos: u32) -> bool {
    let mut ft = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    unsafe {
        if DosDateTimeToFileTime((dos >> 16) as u16, dos as u16, &mut ft) == 0 {
            return false;
        }
        SetFileTime(file as _, std::ptr::null(), &ft, &ft) != 0
    }
}

/// The Rust version: whether it was all done.
pub unsafe fn copy_file(from: *const u8, to: *const u8) -> Result<bool, BadInput> {
    // Converted: `rc2k_core::files::io::copy_file`, through the game's 64 KB block.
    unsafe {
        let buffer =
            std::slice::from_raw_parts_mut(game_addr(rva(BUFFER)) as *mut u8, CHUNK as usize);
        Ok(rc2k_core::files::io::copy_file(
            &mut crate::files::handle::GameFiles::new(),
            from,
            to,
            buffer,
        )?)
    }
}
