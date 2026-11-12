//! `net::receive_thread` (0x4183C3): the network's receiving thread ("NETRXTDB"), the
//! routine of the third slot `threading::start_threads` starts, from its start to
//! `ExitThread`.
//!
//! Its slot's byte +0xC (running) 1 already: straight to the end. Else running;
//! "NETRXTDB:START" (log level 2); `receive_done` not 0: the end. Else round and round:
//! `threading::wait_for_object` on the event (`event`, `net::set_up`'s) for 1000 ms;
//! signalled (zero clear): `receiving` 1 and, with `session` 1 (a network session),
//! `net::take_messages`; then `receiving` 0; with `posting` 1 the call posted in
//! `posted_call` (taken with an `xchg`) is made and its result kept in `posted_result`,
//! `posted_answered` 1 (`net::post_call` posts them and waits; what it posts are Rust
//! functions, `net::lobby_calls`); until `g_quitting` is 1.
//! The end: done 1, "NETRXTDB:QUIT", not running, `ExitThread(0)`.
//! The original hands the starter's registers on from call to call; the calls' Rust
//! versions read none of them (`take_messages` and the posted calls take no arguments).
//! Run by `threading::start_slot_thread` (the net slot, `routes.rs`).

use crate::errors::Refused;
use crate::net::owner::net;
use crate::system::owner::system;

const START_TEXT: &std::ffi::CStr = c"NETRXTDB:START"; // "NETRXTDB:START"
const QUIT_TEXT: &std::ffi::CStr = c"NETRXTDB:QUIT"; // "NETRXTDB:QUIT"
/// The wait for the event, in milliseconds.
const WAIT: u32 = 1000;

/// Runs to the `ExitThread`, which does not come back.
pub fn receive_thread() -> Result<(), Refused> {
    let net = net();
    let vars = &net.vars;
    let running =
        &crate::threading::thread_slot::slot_ref(crate::threading::thread_slot::NET)?.flag;
    if running.get() != 1 {
        running.set(1);
        crate::logfile::log_write::log_nul(&START_TEXT, true, 2);
        if vars.receive_done.get() == 0 {
            loop {
                let (_, zero) =
                    crate::threading::wait_for_object::wait_for_object(vars.event.get(), WAIT)
                        .flags();
                if !zero {
                    vars.receiving.set(1);
                    if vars.session.get() == 1 {
                        crate::net::take_messages::take_messages(
                            net,
                            crate::race::owner::race().viewer_panel(),
                        )?;
                    }
                }
                vars.receiving.set(0);
                if vars.posting.get() == 1
                    && let Some(call) = crate::net::post_call::take_posted(&net.vars)
                {
                    vars.posted_result.set(call());
                    vars.posted_answered.set(1);
                }
                if system().session.flags.quitting.is_set() {
                    break;
                }
            }
        }
    }
    vars.receive_done.set(1);
    crate::logfile::log_write::log_nul(&QUIT_TEXT, true, 2);
    running.set(0);
    // The routine's end ends its thread (the original's `ExitThread`).
    Ok(())
}
