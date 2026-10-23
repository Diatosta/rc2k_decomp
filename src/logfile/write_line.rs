//! `logfile::write_line` (0x40132C): the tail shared by the game's seven log functions. Formats
//! one message into the line buffer and appends it to `var\logfile.txt`.
//!
//! In:  eax = message, dl = newline flag, dh = level - 1; `g_logLock` held by the caller
//! Out, logged:  eax = bytes written (or the path pointer if open or seek failed),
//!               ebx = line buffer start, ecx = line length, edx = dl
//! Out, skipped: ecx = level, edx = dl; eax and ebx unchanged
//! The lock is released on both paths. Entered by `jmp`, so its `retn` returns to the log
//! function's caller.
//!
//! The original writes the real file, so there is no shadow comparison: in shadow mode the
//! original runs alone, and the self-test is the only check of the Rust version.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_ptr, game_read, game_write, rva};
use std::ffi::{CStr, c_char};
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicU32, Ordering};
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{CreateFileA, SetFilePointer, WriteFile};
use windows_sys::Win32::System::Threading::Sleep;

// The game enters this tail holding `g_logLock`, which self-tests that log also take: if the
// thunk waited for self-tests here, the two would wait for each other. So this hook (and the
// hooks it reaches) never waits; the lock is what keeps them apart.
// ---- game data ----

pub const LOG_LEVEL_RVA: usize = rva(0x4E0098); // g_logLevel
const LOG_LOCK_RVA: usize = rva(0x4E009C); // g_logLock
pub const LOG_INDENT_RVA: usize = rva(0x4E00A0); // g_logIndent
pub const LOG_LINE_START_RVA: usize = rva(0x4E00A4); // g_logAtLineStart: 1 after a CRLF-terminated line
const LAST_FILE_PATH_RVA: usize = rva(0x4F52B0); // g_lastFilePath
pub const LINE_BUF_RVA: usize = rva(0x93E080);
// File mode table: 16-byte stride, the log uses mode 4 (offset 0x40).
const MODE_ACCESS_RVA: usize = rva(0x4E06EA) + 0x40;
const MODE_DISPOSITION_RVA: usize = rva(0x4E06EE) + 0x40;
const MODE_SHARE_RVA: usize = rva(0x4E06F2) + 0x40;

static LOG_PATH: &[u8] = b"var\\logfile.txt\0"; // starts with "var\": resolve_path returns it unchanged

pub fn lock() -> &'static AtomicU32 {
    unsafe { &*game_ptr::<AtomicU32>(LOG_LOCK_RVA) }
}

// ---- Rust version ----

/// The bytes the original builds in the line buffer, and the new line-start flag.
/// Converted: `rc2k_core::files::log::format_line`.
pub fn format_line(msg: &[u8], newline: bool, indent: i32, line_start: u32) -> (Vec<u8>, u32) {
    rc2k_core::files::log::format_line(msg, newline, indent, line_start)
}

/// `files::append_to_file` (0x402C9C), whose only caller is this tail.
/// Some(bytes written) if the write was reached; None if open or seek failed.
unsafe fn append_to_file(path: &'static [u8], data: &[u8]) -> Option<u32> {
    unsafe {
        // open_by_mode stores resolve_path's result
        game_write(LAST_FILE_PATH_RVA, path.as_ptr() as u32);
        let h = CreateFileA(
            path.as_ptr(),
            game_read(MODE_ACCESS_RVA),
            game_read(MODE_SHARE_RVA),
            std::ptr::null(),
            game_read(MODE_DISPOSITION_RVA),
            0x80,
            0 as _,
        );
        if h == INVALID_HANDLE_VALUE {
            return None; // the original does not close: the path stays set
        }
        let (mut written, mut result) = (0u32, None);
        if SetFilePointer(h, 0, std::ptr::null_mut(), 2) != u32::MAX {
            // 2 = FILE_END
            WriteFile(
                h,
                data.as_ptr().cast(),
                data.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            );
            result = Some(written);
        }
        CloseHandle(h);
        game_write(LAST_FILE_PATH_RVA, 0u32);
        result
    }
}

/// What a logged line leaves behind.
pub struct Logged {
    /// Length of the line in the line buffer.
    pub len: usize,
    /// Bytes written to the file; None if open or seek failed.
    pub written: Option<u32>,
}

/// None if the level check skipped the message. `msg` is only read when it is logged.
/// The caller must hold `g_logLock`; it is released here on both paths.
pub unsafe fn write_line(msg: *const u8, newline: bool, level_m1: u8) -> Option<Logged> {
    let level = level_m1.wrapping_add(1) as u32; // `inc dh` wraps as a byte
    unsafe {
        if game_read::<u32>(LOG_LEVEL_RVA) < level {
            lock().store(0, Ordering::Release);
            return None;
        }
        let msg = CStr::from_ptr(msg as *const c_char).to_bytes();
        let (bytes, new_flag) = format_line(
            msg,
            newline,
            game_read::<u32>(LOG_INDENT_RVA) as i32,
            game_read(LOG_LINE_START_RVA),
        );
        // The original leaves the line in the game's buffer.
        let buf = game_ptr::<u8>(LINE_BUF_RVA);
        copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
        game_write(LOG_LINE_START_RVA, new_flag);
        let written = append_to_file(LOG_PATH, from_raw_parts(buf, bytes.len()));
        lock().store(0, Ordering::Release);
        Some(Logged {
            len: bytes.len(),
            written,
        })
    }
}

/// The log's state as the game keeps it (`rc2k_core::files::log::LogState`).
mod state {
    use crate::bridge::Global;
    pub static ENABLED: Global<i32> = Global::at(0x4E_0094); // g_logEnabled
    pub static LEVEL: Global<u32> = Global::at(0x4E_0098); // g_logLevel
    pub static INDENT: Global<i32> = Global::at(0x4E_00A0); // g_logIndent
    pub static LINE_START: Global<u32> = Global::at(0x4E_00A4); // g_logAtLineStart
}

fn load_state() -> rc2k_core::files::log::LogState {
    rc2k_core::files::log::LogState {
        enabled: state::ENABLED.get(),
        level: state::LEVEL.get(),
        indent: state::INDENT.get(),
        line_start: state::LINE_START.get(),
    }
}

/// Logs `text` at `level` (1 or 2), with a line break after it if `newline`: what the
/// game's log functions do (`log_write` and this tail; converted:
/// `rc2k_core::files::log::log_line`). The lock (`g_logLock`, shared with the game's code
/// while it logs too) is held from the level check to the file's write; the line is left in
/// the game's line buffer, as the original leaves it.
pub fn log_text(text: &[u8], newline: bool, level: u8) {
    use rc2k_core::files::log::{Entry, Logged, entry, log_line};
    let first = load_state();
    if entry(first.enabled, || first.level) != Entry::Written {
        return;
    }
    acquire();
    let mut now = load_state();
    if let Logged::Line(line) = log_line(&mut now, text, newline, level) {
        state::LINE_START.set(now.line_start);
        // SAFETY: the line is at most `LINE_LIMIT` + 2 bytes, the buffer's size.
        unsafe {
            let buf = game_ptr::<u8>(LINE_BUF_RVA);
            copy_nonoverlapping(line.as_ptr(), buf, line.len());
            append_to_file(LOG_PATH, from_raw_parts(buf, line.len()));
        }
    }
    lock().store(0, Ordering::Release);
}

/// `write_line` with the original's registers in and out.
pub unsafe fn write_line_regs(r: &mut Regs) {
    let (newline, level_m1) = (r.edx as u8, (r.edx >> 8) as u8);
    r.edx = newline as u32;
    match unsafe { write_line(r.eax as *const u8, newline != 0, level_m1) } {
        Some(logged) => {
            r.eax = logged.written.unwrap_or(LOG_PATH.as_ptr() as u32);
            r.ebx = game_ptr::<u8>(LINE_BUF_RVA) as u32;
            r.ecx = logged.len as u32;
        }
        None => r.ecx = level_m1.wrapping_add(1) as u32,
    }
}

/// Takes the lock the way `log_lock_acquire` does.
pub fn acquire() {
    while lock().swap(1, Ordering::Acquire) != 0 {
        unsafe { Sleep(1) }
    }
}
