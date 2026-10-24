//! `net::receive_thread` (0x4183C3): the network's receiving thread ("NETRXTDB"), the
//! routine of the third slot `threading::start_threads` starts (`stru_518714`), from its
//! start to `ExitThread`.
//!
//! In:  nothing (the thread starter's registers; none is read)
//! Out: does not return (`ExitThread(0)`)
//! Its slot's byte +0xC (running) 1 already: straight to the end. Else running;
//! "NETRXTDB:START" (log level 2); `dword_52A4B0` (done) not 0: the end. Else round and
//! round: `threading::wait_for_object` on the event `dword_52AB68` (`net::set_up`'s) for
//! 1000 ms; signalled (zero clear): `dword_52A918` (receiving) 1 and, with `dword_52A910`
//! (a network session) 1, `net::take_messages`; then `dword_52A918` 0; with
//! `dword_52ACE4` 1 the call posted in `dword_52ACE8` (taken with an `xchg`) is made and
//! its eax kept in `dword_52ACEC`, `dword_52ACF0` 1 (`net::post_call` posts them and
//! waits; what it posts are Rust versions, `net::lobby_calls`); until `g_quitting` is 1.
//! The end: done 1, "NETRXTDB:QUIT", not running, `ExitThread(0)`.
//! The original hands the starter's registers on from call to call; the calls' Rust
//! versions read none of them (`take_messages` and the posted calls take no arguments).
//! Run by `threading::start_slot_thread` (its slot `stru_518714`, `routes.rs`).

use crate::errors::Refused;
use crate::logfile::log_write::log_write;
use crate::net::EVENT;
use crate::net::lobby::{
    POSTED, POSTED_ANSWERED, POSTED_RESULT, POSTING, RECEIVE_DONE, SESSION, a, g, p,
};
use crate::session::FLAGS;
use std::sync::atomic::{AtomicU32, Ordering};

const START_TEXT: usize = 0x4DACD2; // "NETRXTDB:START"
const QUIT_TEXT: usize = 0x4DACE1; // "NETRXTDB:QUIT"
/// dword_52A918: the thread is busy with the messages.
const RECEIVING: usize = 0x52A918;
/// The wait for the event, in milliseconds.
const WAIT: u32 = 1000;

/// Runs to the `ExitThread`, which does not come back.
pub fn receive_thread() -> Result<(), Refused> {
    let get = |x: usize| unsafe { (a(x) as *const u32).read_volatile() };
    let running =
        (crate::threading::thread_slot::slot(crate::threading::thread_slot::NET) + 0xC) as *mut u8;
    unsafe {
        if running.read_volatile() != 1 {
            running.write_volatile(1);
            log_write(a(START_TEXT) as *const u8, true, 2);
            if get(RECEIVE_DONE) == 0 {
                loop {
                    let (_, zero) =
                        crate::threading::wait_for_object::wait_for_object(g(EVENT), WAIT).flags();
                    if !zero {
                        p(RECEIVING, 1);
                        if get(SESSION) == 1 {
                            crate::net::take_messages::take_messages()?;
                        }
                    }
                    p(RECEIVING, 0);
                    if get(POSTING) == 1 {
                        let posted =
                            AtomicU32::from_ptr(a(POSTED) as *mut u32).swap(0, Ordering::SeqCst);
                        if posted != 0 {
                            // Always a Rust version's address (`net::lobby_calls`).
                            let call = std::mem::transmute::<usize, extern "C" fn() -> u32>(
                                posted as usize,
                            );
                            p(POSTED_RESULT, call());
                            p(POSTED_ANSWERED, 1);
                        }
                    }
                    if FLAGS.quitting.get() == 1 {
                        break;
                    }
                }
            }
        }
        p(RECEIVE_DONE, 1);
        log_write(a(QUIT_TEXT) as *const u8, true, 2);
        running.write_volatile(0);
        windows_sys::Win32::System::Threading::ExitThread(0)
    }
}
