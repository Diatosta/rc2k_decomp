//! The routine of thread slot 3,
//! started by `threading::request_quit`: "SHUTDOWN:START" logged, the slot's +0xC 1 (in
//! its work), the other threads waited for (`threading::wait_for_threads`), "SHUTDOWN:wfmo
//! rc=" and the wait's result (8 hex digits) logged, the window (`hWnd`, if any) asked to
//! close, +0xC 0, "SHUTDOWN:DONE" logged and the thread ended. It runs as the slot's thread
//! (`start_slot_thread`), and a fatal error on this thread enters it again. The window is SDL's
//! and nothing is posted; the quit switch ends its loop.

use crate::logfile::write_line::log_text;

/// Ends the thread.
pub fn shutdown_thread() -> Result<(), crate::errors::Refused> {
    log_text(b"SHUTDOWN:START", true, 2);
    let flag =
        &crate::threading::thread_slot::slot_ref(crate::threading::thread_slot::SHUTDOWN)?.flag;
    flag.set(1);
    let rc = crate::threading::wait_for_threads::wait_for_threads()?;
    log_text(b"SHUTDOWN:wfmo rc=", false, 2);
    crate::logfile::log_hex8::log_hex8(rc, true, 2);
    flag.set(0);
    log_text(b"SHUTDOWN:DONE", true, 2);
    // The routine's end ends its thread.
    Ok(())
}
