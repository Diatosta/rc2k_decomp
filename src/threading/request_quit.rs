//! Asks the game to shut down: sets `flags.quitting` to 1 and, if it was not 1 already, starts
//! the thread of slot 3, which logs "SHUTDOWN:START", waits for the other threads and posts a
//! close message to the window. Used by the fatal-error reporter and `system::main_loop`; it
//! cannot be run twice or undone.

use crate::errors::FatalError;
use crate::logfile::program_log::log;
use crate::threading::start_slot_thread::start_slot_thread;
use crate::threading::thread_slot::SlotOwners;
use std::sync::atomic::{AtomicU32, Ordering};

/// `requested` is the shutdown switch; `start` starts the thread of the slot of the number
/// it is given.
pub fn request_quit_with(
    requested: &AtomicU32,
    start: &mut dyn FnMut(usize) -> Result<(), FatalError>,
) -> Result<(), FatalError> {
    use rc2k_core::threading::{QUITTING, starts_shutdown};
    if !starts_shutdown(requested.swap(QUITTING, Ordering::SeqCst)) {
        return Ok(());
    }
    start(crate::threading::thread_slot::SHUTDOWN)
}

/// `request_quit` on the system of `owners`, the owners the shutdown thread gets.
pub fn request_quit_in(owners: SlotOwners<'static>) -> Result<(), FatalError> {
    let requested = owners.system.session.flags.quitting.atomic();
    let before = requested.load(Ordering::SeqCst);
    let result = request_quit_with(requested, &mut |slot| start_slot_thread(slot, owners));
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
