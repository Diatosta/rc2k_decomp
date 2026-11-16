//! Asks the game to shut down: sets `flags.quitting` to 1 and, if it was not 1 already, starts the
//! shutdown thread, which waits for the other threads. It cannot be run twice or undone.

use crate::errors::FatalError;
use crate::logfile::program_log::log;
use crate::threading::thread_slot::{SlotOwners, SlotThread};
use std::sync::atomic::{AtomicU32, Ordering};

/// `requested` is the shutdown switch; `start` starts the shutdown thread.
pub fn request_quit_with(
    requested: &AtomicU32,
    start: &mut dyn FnMut() -> Result<(), FatalError>,
) -> Result<(), FatalError> {
    use rc2k_core::threading::{QUITTING, starts_shutdown};
    if !starts_shutdown(requested.swap(QUITTING, Ordering::SeqCst)) {
        return Ok(());
    }
    start()
}

/// `request_quit` on the system of `owners`, the owners the shutdown thread gets.
pub fn request_quit_in(owners: SlotOwners<'static>) -> Result<(), FatalError> {
    let requested = owners.system.session.flags.quitting.atomic();
    let before = requested.load(Ordering::SeqCst);
    let result = request_quit_with(requested, &mut || SlotThread::Shutdown.start(owners));
    if result.is_err() {
        // Left as it was, so that a later request tries the thread again.
        requested.store(before, Ordering::SeqCst);
    }
    result
}

/// `request_quit`, saying in the log that it ran.
pub fn request_quit_logged(owners: SlotOwners<'static>) -> Result<(), FatalError> {
    // It happens once, while the game is closing: say in the log that it ran.
    let was = u32::from(owners.system.session.flags.quitting.is_set());
    let result = request_quit_in(owners);
    log(&format!(
        "threading::request_quit: ran (quitting was {was}): {result:?}"
    ));
    result
}
