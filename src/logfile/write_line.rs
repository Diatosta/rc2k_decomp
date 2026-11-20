//! The tail shared by the game's seven log functions. Formats
//! one message into the line buffer and appends it to `var\logfile.txt` (nothing when its level
//! is too high). The lock is released on every path.

/// The log's state: enabled, level, indent, at-line-start. Atomics: any thread logs.
/// `LEVEL` is set from `var\ral.cfg` (`logfile=`); the race's steps change `INDENT` and
/// `ENABLED` directly.
pub mod state {
    use crate::memory::session::Dword;
    /// On while above 0 (signed).
    pub static ENABLED: Dword = Dword::new(1);
    /// The highest level logged; 0 logs nothing.
    pub static LEVEL: Dword = Dword::new(1);
    /// Spaces before a line.
    pub static INDENT: Dword = Dword::new(0);
    /// 1 when the next text starts a line.
    pub static LINE_START: Dword = Dword::new(0);
}

pub fn level() -> u32 {
    state::LEVEL.get()
}

pub fn set_enabled(value: i32) {
    state::ENABLED.set(value as u32);
}

/// Windows opens the log with no sharing.
#[cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Windows' own sharing; std has no share mode elsewhere"
    )
)]
const MODE_SHARE: u32 = 0;

/// "var\logfile.txt"; starts with "var\": `resolve_path` returns it unchanged.
pub static LOG_PATH: &[u8] = b"var\\logfile.txt\0";

/// Held from a line's level check to its write: one thread's line at a time.
static LINE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The log file kept open for appending between writes, where the system lets another process
/// read a file that is open for writing (everywhere but Windows, whose open file is not shared).
/// Opening and closing it for every fragment of every line is most of what a logged stage load
/// costs on a slow card.
#[cfg(not(windows))]
static OPEN_LOG: std::sync::Mutex<Option<std::fs::File>> = std::sync::Mutex::new(None);

/// Lets go of the kept log file: the next write opens the path again (the log was deleted).
pub(super) fn close_log() {
    #[cfg(not(windows))]
    {
        *OPEN_LOG
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }
}

/// Appends `data` to the log file; None if it cannot be opened or sought to its end, else the
/// bytes written.
fn append_to_file(path: &'static [u8], data: &[u8]) -> Option<u32> {
    use std::io::Write;
    crate::files::state::set_last_file_path(Some(&path[..path.len() - 1]));
    #[cfg(not(windows))]
    let result = {
        let mut kept = OPEN_LOG
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if kept.is_none() {
            let name = crate::files::host_path(&path[..path.len() - 1]);
            // Appending: every write goes to the file's end.
            *kept = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(name)
                .ok();
        }
        // The bytes written; a failed write counts what it wrote, nothing.
        kept.as_mut()
            .map(|file| file.write(data).unwrap_or(0) as u32)
    };
    #[cfg(windows)]
    let result = {
        let name = crate::files::host_path(&path[..path.len() - 1]);
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        crate::platform::os::share_mode(&mut options, MODE_SHARE);
        let mut file = options.open(name).ok()?; // not closed here: the path stays set
        match std::io::Seek::seek(&mut file, std::io::SeekFrom::End(0)) {
            // The bytes written; a failed write counts what it wrote, nothing.
            Ok(_) => Some(file.write(data).unwrap_or(0) as u32),
            Err(_) => None,
        }
    };
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

/// Logs `text` at `level` (1 or 2), with a line break after it if `newline` (converted:
/// `rc2k_core::files::log::log_line`). The lock is held from the level check to the file's
/// write.
pub fn log_text(text: &[u8], newline: bool, level: u8) {
    use rc2k_core::files::log::{Entry, Logged, entry, log_line};
    let first = load_state();
    if entry(first.enabled, || first.level) != Entry::Written {
        return;
    }
    let _one_line = LINE_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut now = load_state();
    if let Logged::Line(line) = log_line(&mut now, text, newline, level) {
        state::LINE_START.set(now.line_start);
        append_to_file(LOG_PATH, &line);
    }
}
