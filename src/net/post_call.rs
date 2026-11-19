//! With `receiver.posting` 1, hands the given Rust function to the receiving thread (its event is
//! signalled, so it takes the call at once rather than at the end of its wait), and waits (pumping
//! the window's messages every few milliseconds) until it has answered or the game is closing;
//! returns its result. DirectPlay calls that may show a dialog
//! (modem, serial link) are made on the receiving thread this way. Without posting it just
//! calls `code`.

use crate::net::owner::Net;
use crate::net::store::NetShared;
use crate::race::viewer_screen_store::NetTexts;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

/// A piece of code posted to the receiving thread: it gets the network's state and the
/// lines's texts.
pub type PostedCall = fn(&Net, &NetTexts) -> u32;

/// How often the poster looks for the answer, pumping the window's messages in between.
const POLL_MS: u32 = 5;

/// The call posted, while `posted_call` is 1.
static POSTED: Mutex<Option<PostedCall>> = Mutex::new(None);
pub fn post_call(o: crate::owners::Owners<'static>, code: PostedCall) -> u32 {
    let net = o.net;
    if net.vars.receiver.posting.get() == 1 {
        *POSTED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(code);
        net.vars.receiver.posted_call.set(1);
        crate::platform::sync::set_event(net.vars.event.get());
        loop {
            let answered = net
                .vars
                .receiver
                .posted_answered
                .atomic()
                .swap(0, Ordering::SeqCst);
            if answered != 0 {
                break;
            }
            crate::clock::sleep::sleep(POLL_MS);
            crate::platform::pump_messages_now(o);
            if o.system.session.flags.quitting.is_set() {
                break;
            }
        }
    }
    net.vars.receiver.posted_result.get()
}

/// The call posted, taken by the receiving thread (none waiting: `None`).
pub fn take_posted(vars: &NetShared) -> Option<PostedCall> {
    if vars.receiver.posted_call.atomic().swap(0, Ordering::SeqCst) == 0 {
        return None;
    }
    POSTED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}
