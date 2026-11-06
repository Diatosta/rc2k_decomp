//! `threading::request_quit` (0x410BF9): asks the game to shut down: sets `dword_51837C` to 1
//! and, if it was not 1 already, starts the thread of slot 3 (`stru_518824`), whose routine
//! at 0x410C17 logs "SHUTDOWN:START", waits for the other threads and posts a close message
//! to the window.
//!
//! In:  nothing
//! Out: every register preserved
//! Two callers: the fatal-error reporter and `sub_415383`.
//! It cannot be run twice or undone, so shadow mode runs the original alone. The self-test
//! runs a copy of the original whose switch is a test variable and whose call to
//! `start_slot_thread` goes to a stand-in.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::FatalError;
use crate::framework::log;
use crate::threading::start_slot_thread::start_slot_thread;
use std::sync::atomic::{AtomicU32, Ordering};

// ---- game data ----

// ---- Rust version ----

/// `requested` is the shutdown switch; `start` starts the thread of the slot of the number
/// it is given.
pub fn request_quit_with(
    requested: &AtomicU32,
    start: &mut dyn FnMut(usize) -> Result<(), FatalError>,
) -> Result<(), FatalError> {
    // Converted: `rc2k_core::threading::starts_shutdown`.
    use rc2k_core::threading::{QUITTING, starts_shutdown};
    if !starts_shutdown(requested.swap(QUITTING, Ordering::SeqCst)) {
        return Ok(());
    }
    start(crate::threading::thread_slot::SHUTDOWN)
}

pub fn request_quit() -> Result<(), FatalError> {
    let requested = crate::system::owner::system()
        .session
        .flags
        .quitting
        .atomic();
    let before = requested.load(Ordering::SeqCst);
    let result = request_quit_with(requested, &mut start_slot_thread);
    if result.is_err() {
        // Left as it was, so that the original, run in its place, tries the thread too.
        requested.store(before, Ordering::SeqCst);
    }
    result
}

/// `request_quit`, which version ran said in the log.
pub fn request_quit_logged() -> Result<(), FatalError> {
    // It happens once, while the game is closing, when `stats` can no longer be asked: say
    // in the log which version ran.
    let was: u32 = crate::system::owner::system().session.flags.quitting.get();
    let result = request_quit();
    log(&format!(
        "threading::request_quit: the Rust version ran (switch was {was}): {result:?}"
    ));
    result
}

// pusha; mov eax, 1
