//! The network's receiving thread ("NETRXTDB"): until the game is quitting, waits up to a
//! second for the event; when signalled and a network session is on, takes the messages
//! (`net::take_messages`); a call posted by `net::post_call` is made and its result kept. Ends
//! with "NETRXTDB:QUIT".

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::net::owner::Net;
use crate::system::owner::System;

const START_TEXT: &std::ffi::CStr = c"NETRXTDB:START"; // "NETRXTDB:START"
const QUIT_TEXT: &std::ffi::CStr = c"NETRXTDB:QUIT"; // "NETRXTDB:QUIT"
/// The wait for the event, in milliseconds.
const WAIT: u32 = 1000;

/// Runs to the `ExitThread`, which does not come back.
pub fn receive_thread(net: &Net, gfx: &'static Gfx, system: &System) -> Result<(), Refused> {
    let lines = &crate::race::viewer_screen_store::NET_TEXTS;
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
                        crate::net::take_messages::take_messages(net, gfx, lines)?;
                    }
                }
                vars.receiving.set(0);
                if vars.posting.get() == 1
                    && let Some(call) = crate::net::post_call::take_posted(&net.vars)
                {
                    vars.posted_result.set(call(net, lines));
                    vars.posted_answered.set(1);
                }
                if system.session.flags.quitting.is_set() {
                    break;
                }
            }
        }
    }
    vars.receive_done.set(1);
    crate::logfile::log_write::log_nul(&QUIT_TEXT, true, 2);
    running.set(0);
    // The routine's end ends its thread.
    Ok(())
}
