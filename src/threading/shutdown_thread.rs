//! `threading::shutdown_thread` (0x410C17): the routine of thread slot 3 (`stru_518824`),
//! started by `threading::request_quit`: "SHUTDOWN:START" logged, the slot's +0xC 1 (in
//! its work), the other threads waited for (`threading::wait_for_threads`), "SHUTDOWN:wfmo
//! rc=" and the wait's result (8 hex digits) logged, the window (`hWnd`, if any) asked to
//! close (`PostMessageA` WM_CLOSE), +0xC 0, "SHUTDOWN:DONE" logged and the thread ended
//! (`ExitThread(0)`).
//!
//! In:  nothing (a thread's start; `esi` is set to the slot)
//! Out: does not return
//!
//! No callers: `CreateThread`'s (`start_slot_thread`, through `routes.rs`: the slot holds
//! the routine's address), and a fatal error on this thread enters it again
//! (`fatal::report`'s `jmp [esi]`). In the standalone the window's handle is 0 and nothing
//! is posted. The registers above are the original's, for reference.

use crate::logfile::write_line::log_text;
use crate::session::MACHINE;

const WM_CLOSE: u32 = 0x10;

/// The Rust version (see the header): ends the thread.
pub fn shutdown_thread() -> Result<(), crate::errors::Refused> {
    use windows_sys::Win32::System::Threading::ExitThread;
    use windows_sys::Win32::UI::WindowsAndMessaging::PostMessageA;
    log_text(b"SHUTDOWN:START", true, 2);
    let flag =
        &crate::threading::thread_slot::slot_ref(crate::threading::thread_slot::SHUTDOWN)?.flag;
    flag.set(1);
    let rc = crate::threading::wait_for_threads::wait_for_threads()?;
    log_text(b"SHUTDOWN:wfmo rc=", false, 2);
    crate::logfile::log_hex8::log_hex8(rc, true, 2);
    let window: u32 = MACHINE.window.get();
    if window != 0 {
        unsafe { PostMessageA(crate::pointers::host(window) as _, WM_CLOSE, 0, 0) };
    }
    flag.set(0);
    log_text(b"SHUTDOWN:DONE", true, 2);
    unsafe { ExitThread(0) }
}
