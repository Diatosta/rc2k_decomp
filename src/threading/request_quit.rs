//! `threading::request_quit` (0x410BF9): asks the game to shut down: sets `flags.quitting` to 1
//! and, if it was not 1 already, starts the thread of slot 3, whose routine
//! at 0x410C17 logs "SHUTDOWN:START", waits for the other threads and posts a close message
//! to the window.
//!
//! Two callers: the fatal-error reporter and `system::main_loop`.
//! It cannot be run twice or undone.

use crate::errors::FatalError;
use crate::framework::log;
use crate::system::owner::{System, system};
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

/// `request_quit_in` for the fatal reporter, which runs on any thread with no owners in hand.
pub fn request_quit() -> Result<(), FatalError> {
    request_quit_in(system())
}

/// `request_quit` on the system the caller holds.
pub fn request_quit_in(system: &System) -> Result<(), FatalError> {
    let requested = system.session.flags.quitting.atomic();
    let before = requested.load(Ordering::SeqCst);
    let result = request_quit_with(requested, &mut start_slot_thread);
    if result.is_err() {
        // Left as it was, so that the original, run in its place, tries the thread too.
        requested.store(before, Ordering::SeqCst);
    }
    result
}

/// `request_quit`, saying in the log that it ran.
pub fn request_quit_logged(system: &System) -> Result<(), FatalError> {
    // It happens once, while the game is closing: say in the log that it ran.
    let was = u32::from(system.session.flags.quitting.is_set());
    let result = request_quit_in(system);
    log(&format!(
        "threading::request_quit: ran (quitting was {was}): {result:?}"
    ));
    result
}
