//! The network's receiving thread ("NETRXTDB"): until the game is quitting, waits up to a
//! second for the event; when signalled and a network session is on, takes the messages
//! (`net::take_messages`); a call posted by `net::post_call` is made and its result kept. Ends
//! with "NETRXTDB:QUIT".

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::system::owner::System;
use rc2k_core::threading::Waited;
use std::sync::atomic::Ordering;

const START_TEXT: &std::ffi::CStr = c"NETRXTDB:START";
const QUIT_TEXT: &std::ffi::CStr = c"NETRXTDB:QUIT";
/// The wait for the event, in milliseconds.
const WAIT: u32 = 1000;

/// Runs to the thread's end.
pub fn receive_thread(net: &Net, system: &System) -> Result<(), Refused> {
    let lines = &crate::race::viewer_screen_store::NET_TEXTS;
    let vars = &net.vars;
    let running = &crate::threading::thread_slot::SlotThread::NetReceive
        .slot()
        .running;
    if !running.swap(true, Ordering::SeqCst) {
        crate::logfile::log_write::log_nul(&START_TEXT, true, 2);
        if vars.receive_done.get() == 0 {
            loop {
                // A wait that fails counts as the time running out.
                let waited = Waited::of(crate::platform::sync::wait(vars.event.get(), WAIT));
                if waited == Waited::Signalled {
                    vars.receiver.receiving.set(1);
                    if net.race_slot.is_on() {
                        match crate::net::take_messages::take_messages(
                            net,
                            lines,
                            &system.session.picture_clock,
                        ) {
                            // What a sender's data makes the take refuse is dropped, with a
                            // notice; the session goes on.
                            Err(Refused::BadInput(why)) => {
                                crate::net::bad_peer::dropped_message(u32::MAX, why.0);
                            }
                            other => other?,
                        }
                    }
                }
                vars.receiver.receiving.set(0);
                if vars.receiver.posting.get() == 1
                    && let Some(call) = crate::net::post_call::take_posted(&net.vars)
                {
                    vars.receiver.posted_result.set(call(net, lines));
                    vars.receiver.posted_answered.set(1);
                }
                if system.session.flags.quitting.is_set() {
                    break;
                }
            }
        }
    }
    vars.receive_done.set(1);
    crate::logfile::log_write::log_nul(&QUIT_TEXT, true, 2);
    running.store(false, Ordering::SeqCst);
    // The routine's end ends its thread.
    Ok(())
}
