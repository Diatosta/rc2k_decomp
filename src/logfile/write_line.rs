//! `logfile::write_line` (0x40132C): the tail shared by the game's seven log functions. Formats
//! one message into the line buffer and appends it to `var\logfile.txt`.
//!
//! Takes the message, the newline flag and the level - 1; `g_logLock` held by the caller.
//! Logged: the bytes written, or the path if the open or seek failed. Skipped (level too
//! high): nothing written.
//! The lock is released on both paths. Entered by `jmp`, so its `retn` returns to the log
//! function's caller.

use std::sync::atomic::{AtomicU32, Ordering};

// ---- game data ----

/// The log's state (`g_logEnabled`,
/// `g_logLevel`, `g_logLock`, `g_logIndent`, `g_logAtLineStart`, 0x4E0094 .. 0x4E00A7).
/// Atomics: any thread logs. `LEVEL` is set from `var\ral.cfg` (`logfile=`); the race's
/// steps change `INDENT` and `ENABLED` directly.
pub mod state {
    use crate::session::Dword;
    /// On while above 0 (signed). 1 in the image.
    pub static ENABLED: Dword = Dword::new(1);
    /// The highest level logged; 0 logs nothing. 1 in the image.
    pub static LEVEL: Dword = Dword::new(1);
    /// 1 while a thread writes the log (`acquire`).
    pub static LOCK: Dword = Dword::new(0);
    /// Spaces before a line.
    pub static INDENT: Dword = Dword::new(0);
    /// 1 when the next text starts a line.
    pub static LINE_START: Dword = Dword::new(0);
}

/// `g_logLevel` now.
pub fn level() -> u32 {
    state::LEVEL.get()
}

/// Turns the log on or off (`g_logEnabled`).
pub fn set_enabled(value: i32) {
    state::ENABLED.set(value as u32);
}

/// `dword_4E06EA` .. (`files`' table of file modes, 16 bytes each): mode 4's access,
/// disposition and share.
const MODE_ACCESS: u32 = 0x4000_0000;
const MODE_DISPOSITION: u32 = 4;
#[cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Windows' own sharing; std has no share mode elsewhere"
    )
)]
const MODE_SHARE: u32 = 0;

/// "var\logfile.txt" (`aVarLogfileTxt`, 0x4E00A8); starts with "var\": `resolve_path`
/// returns it unchanged.
pub static LOG_PATH: &[u8] = b"var\\logfile.txt\0";

pub fn lock() -> &'static AtomicU32 {
    state::LOCK.atomic()
}

// ---- Rust version ----

/// `files::append_to_file` (0x402C9C), whose only caller is this tail.
/// Some(bytes written) if the write was reached; None if open or seek failed.
fn append_to_file(path: &'static [u8], data: &[u8]) -> Option<u32> {
    use std::io::{Seek, SeekFrom, Write};
    // open_by_mode stores resolve_path's result
    crate::files::state::set_last_file_path(Some(&path[..path.len() - 1]));
    // Mode 4 (`MODE_*`): write access, no sharing, open or create (std's `create`;
    // `write` only lets std accept it, the access is the table's).
    const _: () = assert!(MODE_DISPOSITION == 4);
    const _: () = assert!(MODE_ACCESS == 0x4000_0000); // GENERIC_WRITE
    let name = crate::files::host_path(&path[..path.len() - 1]);
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(MODE_SHARE).attributes(0x80);
    }
    let mut file = options.open(name).ok()?; // the original does not close: the path stays set
    let result = match file.seek(SeekFrom::End(0)) {
        // The bytes written; a failed write counts what it wrote, nothing.
        Ok(_) => Some(file.write(data).unwrap_or(0) as u32),
        Err(_) => None,
    };
    drop(file);
    crate::files::state::set_last_file_path(None);
    result
}

fn load_state() -> rc2k_core::files::log::LogState {
    rc2k_core::files::log::LogState {
        enabled: (state::ENABLED.get() as i32),
        level: state::LEVEL.get(),
        indent: (state::INDENT.get() as i32),
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
        append_to_file(LOG_PATH, &line);
    }
    lock().store(0, Ordering::Release);
}

/// Takes the lock the way `log_lock_acquire` does.
pub fn acquire() {
    while lock().swap(1, Ordering::Acquire) != 0 {
        crate::system::sleep::sleep(1);
    }
}
