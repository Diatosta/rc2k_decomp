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

use crate::moved::Place;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicU32, Ordering};

// ---- game data ----

/// The log's state, out of the image since 2026-10-24 (phase 5; it was `g_logEnabled`,
/// `g_logLevel`, `g_logLock`, `g_logIndent`, `g_logAtLineStart`, 0x4E0094 .. 0x4E00A7).
/// Atomics: any thread logs. `LEVEL` is set from `var\ral.cfg` (`logfile=`, through
/// `crate::moved::place`); the race's steps change `INDENT` and `ENABLED` directly.
pub mod state {
    use std::sync::atomic::{AtomicI32, AtomicU32};
    /// On while above 0 (signed). 1 in the image.
    pub static ENABLED: AtomicI32 = AtomicI32::new(1);
    /// The highest level logged; 0 logs nothing. 1 in the image.
    pub static LEVEL: AtomicU32 = AtomicU32::new(1);
    /// 1 while a thread writes the log (`acquire`).
    pub static LOCK: AtomicU32 = AtomicU32::new(0);
    /// Spaces before a line.
    pub static INDENT: AtomicI32 = AtomicI32::new(0);
    /// 1 when the next text starts a line.
    pub static LINE_START: AtomicU32 = AtomicU32::new(0);
}

/// `g_logLevel` now.
pub fn level() -> u32 {
    state::LEVEL.load(Ordering::Relaxed)
}

/// Turns the log on or off (`g_logEnabled`).
pub fn set_enabled(value: i32) {
    state::ENABLED.store(value, Ordering::Relaxed)
}

/// The line buffer (0x93E080 .. the text buffer at 0x93E430 in the image): a line is at
/// most `LINE_LIMIT` + 3 bytes.
pub const LINE_BUF_LEN: usize = 0x3B0;
pub static LINE_BUF: Place<[u8; LINE_BUF_LEN]> = Place::new([0; LINE_BUF_LEN]);
/// `dword_4E06EA` .. (`files`' table of file modes, 16 bytes each): mode 4's access,
/// disposition and share, as the image holds them (`tests::modes_are_the_games`).
const MODE_ACCESS: u32 = 0x4000_0000;
const MODE_DISPOSITION: u32 = 4;
const MODE_SHARE: u32 = 0;

/// "var\logfile.txt" (`aVarLogfileTxt`, 0x4E00A8); starts with "var\": `resolve_path`
/// returns it unchanged.
pub static LOG_PATH: &[u8] = b"var\\logfile.txt\0";

pub fn lock() -> &'static AtomicU32 {
    &state::LOCK
}

fn line_buf() -> *mut u8 {
    LINE_BUF.ptr().cast()
}

// ---- Rust version ----

/// `files::append_to_file` (0x402C9C), whose only caller is this tail.
/// Some(bytes written) if the write was reached; None if open or seek failed.
fn append_to_file(path: &'static [u8], data: &[u8]) -> Option<u32> {
    use std::io::{Seek, SeekFrom, Write};
    use std::os::windows::fs::OpenOptionsExt;
    // open_by_mode stores resolve_path's result
    crate::files::state::LAST_FILE_PATH.store(path.as_ptr() as u32, Ordering::Relaxed);
    // Mode 4 (`MODE_*`): write access, no sharing, open or create (std's `create`;
    // `write` only lets std accept it, the access is the table's).
    const _: () = assert!(MODE_DISPOSITION == 4);
    let name = crate::files::ansi_text(&path[..path.len() - 1]);
    let mut file = std::fs::OpenOptions::new()
        .access_mode(MODE_ACCESS)
        .share_mode(MODE_SHARE)
        .attributes(0x80)
        .write(true)
        .create(true)
        .open(name)
        .ok()?; // the original does not close: the path stays set
    let result = match file.seek(SeekFrom::End(0)) {
        // The bytes written; a failed write counts what it wrote, nothing.
        Ok(_) => Some(file.write(data).unwrap_or(0) as u32),
        Err(_) => None,
    };
    drop(file);
    crate::files::state::LAST_FILE_PATH.store(0, Ordering::Relaxed);
    result
}

fn load_state() -> rc2k_core::files::log::LogState {
    rc2k_core::files::log::LogState {
        enabled: state::ENABLED.load(Ordering::Relaxed),
        level: state::LEVEL.load(Ordering::Relaxed),
        indent: state::INDENT.load(Ordering::Relaxed),
        line_start: state::LINE_START.load(Ordering::Relaxed),
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
        state::LINE_START.store(now.line_start, Ordering::Relaxed);
        // SAFETY: the line is at most `LINE_LIMIT` + 3 bytes, within the buffer; the lock
        // is held.
        unsafe {
            let buf = line_buf();
            copy_nonoverlapping(line.as_ptr(), buf, line.len());
            append_to_file(LOG_PATH, from_raw_parts(buf, line.len()));
        }
    }
    lock().store(0, Ordering::Release);
}

/// Takes the lock the way `log_lock_acquire` does.
pub fn acquire() {
    while lock().swap(1, Ordering::Acquire) != 0 {
        crate::system::sleep::sleep(1)
    }
}

#[cfg(test)]
mod tests {
    use crate::moved::image_bytes;
    use std::sync::atomic::Ordering;

    #[test]
    fn modes_are_the_games() {
        let mode = image_bytes(0x4E06EA + 0x40, 12);
        let word = |n: usize| u32::from_le_bytes(mode[n..n + 4].try_into().unwrap());
        assert_eq!(
            [word(0), word(4), word(8)],
            [
                super::MODE_ACCESS,
                super::MODE_DISPOSITION,
                super::MODE_SHARE
            ]
        );
        assert_eq!(image_bytes(0x4E00A8, 0x10), super::LOG_PATH);
    }

    #[test]
    fn state_starts_as_the_games() {
        use super::state::*;
        let d = image_bytes(0x4E0094, 0x14);
        let word = |n: usize| u32::from_le_bytes(d[n..n + 4].try_into().unwrap());
        assert_eq!(word(0) as i32, ENABLED.load(Ordering::Relaxed));
        assert_eq!(word(4), LEVEL.load(Ordering::Relaxed));
        assert_eq!(word(8), LOCK.load(Ordering::Relaxed));
        assert_eq!(word(0xC) as i32, INDENT.load(Ordering::Relaxed));
        assert_eq!(word(0x10), LINE_START.load(Ordering::Relaxed));
    }
}
