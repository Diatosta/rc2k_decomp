//! The shutdown thread's routine, started by `threading::request_quit`: "SHUTDOWN:START"
//! logged, the input, audio and network receive threads waited for (all of them, three seconds
//! at most) and their handles closed, "SHUTDOWN:wfmo rc=" and the wait's result (8 hex digits)
//! logged, then "SHUTDOWN:DONE". A fatal error on this thread enters it again. The quit switch
//! ends the game's own loop; nothing is sent to the window.

use crate::logfile::write_line::log_text;
use crate::threading::thread_slot::SlotThread;
use std::sync::atomic::Ordering;

/// Ends the thread.
pub fn shutdown_thread() -> Result<(), crate::errors::Refused> {
    log_text(b"SHUTDOWN:START", true, 2);
    let running = &SlotThread::Shutdown.slot().running;
    running.store(true, Ordering::SeqCst);
    let rc = wait_for_threads();
    log_text(b"SHUTDOWN:wfmo rc=", false, 2);
    crate::logfile::log_hex8::log_hex8(rc, true, 2);
    running.store(false, Ordering::SeqCst);
    log_text(b"SHUTDOWN:DONE", true, 2);
    // The routine's end ends its thread.
    Ok(())
}

/// The three worker threads waited for, three seconds at most, and their handles closed; the
/// wait's result.
fn wait_for_threads() -> u32 {
    use rc2k_core::threading::{WAIT_MS, wait_order};
    let handles = SlotThread::WORKERS.map(|thread| thread.slot().handle.load(Ordering::SeqCst));
    let array = wait_order(handles);
    let rc = crate::platform::sync::wait_all_threads(&array, WAIT_MS);
    for handle in array {
        crate::platform::sync::close(handle);
    }
    rc
}
