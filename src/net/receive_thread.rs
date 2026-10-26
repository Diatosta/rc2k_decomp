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
use crate::net::store::NET;
use crate::session::FLAGS;
use std::sync::atomic::Ordering;

const START_TEXT: &std::ffi::CStr = c"NETRXTDB:START"; // "NETRXTDB:START"
const QUIT_TEXT: &std::ffi::CStr = c"NETRXTDB:QUIT"; // "NETRXTDB:QUIT"
/// The wait for the event, in milliseconds.
const WAIT: u32 = 1000;

/// Runs to the `ExitThread`, which does not come back.
pub fn receive_thread() -> Result<(), Refused> {
    let running =
        &crate::threading::thread_slot::slot_ref(crate::threading::thread_slot::NET)?.flag;
    unsafe {
        if running.get() != 1 {
            running.set(1);
            log_write(START_TEXT.as_ptr().cast(), true, 2);
            if NET.receive_done.get() == 0 {
                loop {
                    let (_, zero) =
                        crate::threading::wait_for_object::wait_for_object(NET.event.get(), WAIT)
                            .flags();
                    if !zero {
                        NET.receiving.set(1);
                        if NET.session.get() == 1 {
                            crate::net::take_messages::take_messages()?;
                        }
                    }
                    NET.receiving.set(0);
                    if NET.posting.get() == 1 {
                        let posted = NET.posted_call.atomic().swap(0, Ordering::SeqCst);
                        if posted != 0 {
                            // Always a Rust version's address (`net::lobby_calls`).
                            let call = std::mem::transmute::<usize, extern "C" fn() -> u32>(
                                posted as usize,
                            );
                            NET.posted_result.set(call());
                            NET.posted_answered.set(1);
                        }
                    }
                    if FLAGS.quitting.get() == 1 {
                        break;
                    }
                }
            }
        }
        NET.receive_done.set(1);
        log_write(QUIT_TEXT.as_ptr().cast(), true, 2);
        running.set(0);
        windows_sys::Win32::System::Threading::ExitThread(0)
    }
}
