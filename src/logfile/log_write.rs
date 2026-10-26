//! `logfile::log_write` (0x4013CB): the entry of the string log function: checks that logging
//! is on, takes the log's lock and goes on into `write_line`.
//!
//! In:  eax = text, dl = 1 for a line break after it, dh = level - 1
//! Out: as `write_line` leaves the registers if it got that far. Nothing changes if
//!      `g_logEnabled` is 0 or less. If `g_logLevel` is 0, only eax changes, to the code
//!      address 0x4013D9: the original's lock function then returns straight to the caller
//!      of this one, with its own return address popped into eax.
//! The four entries `log_write_l1`, `_l1_nl`, `_l2`, `_l2_nl` (0x401427 ..) set edx and jump
//! here, so they go through this hook too, as do the four `log_newline`/`log_space` helpers.
//! The lock is `g_logLock`, taken with an `xchg` and `Sleep(1)` between tries
//! (`log_lock_acquire`, 0x40130A, inlined here); `write_line` releases it.
//! It writes the real log file, so shadow mode runs the original alone. The thunk does not
//! wait for self-tests and the self-test runs with the game running, like `write_line`'s.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::logfile::write_line::log_text;

// ---- game data ----

// ---- Rust version ----

/// For Rust callers holding a pointer to a NUL-terminated text (the game's): logs it at
/// `level` (1 or 2), with a line break after it if `newline` (`write_line::log_text`).
///
/// # Safety
/// `text` points to a NUL-terminated text.
pub unsafe fn log_write(text: *const u8, newline: bool, level: u8) {
    let text = unsafe { std::ffi::CStr::from_ptr(text.cast()) };
    log_text(text.to_bytes(), newline, level);
}

/// The NUL-terminated text at `at` (the game's memory, checked: `files::cstring`) logged
/// as `log_write`; a text outside its region ends the program (`fatal::raise::end`).
pub fn log_at(at: u32, newline: bool, level: u8) {
    match crate::files::cstring(crate::pointers::GamePtr::new(at), "a text to log") {
        Ok(text) => log_text(&text, newline, level),
        Err(e) => crate::fatal::raise::end("logfile::log_write", e),
    }
}

/// `logfile::log_newline_l1` (0x4013DE): an empty line at level 1 (the empty text is the
/// end of the log's path literal); every register kept (`pusha` .. `popa`).
pub fn log_newline_l1() {
    log_text(b"", true, 1);
}
