//! With `posting` 1, hands the given Rust function to the receiving thread, which calls it,
//! and waits (sleeping a tenth of a second and pumping the window's messages) until it has
//! answered or the game is closing; returns its result. DirectPlay calls that may show a dialog
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

/// The call posted, while `posted_call` is 1.
static POSTED: Mutex<Option<PostedCall>> = Mutex::new(None);
pub fn post_call(o: crate::owners::Owners<'static>, code: PostedCall) -> u32 {
    let net = o.net;
    if net.vars.posting.get() == 1 {
        *POSTED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(code);
        net.vars.posted_call.set(1);
        loop {
            let answered = net.vars.posted_answered.atomic().swap(0, Ordering::SeqCst);
            if answered != 0 {
                break;
            }
            crate::system::sleep::sleep(100);
            crate::platform::pump_messages_now(o);
            if o.system.session.flags.quitting.is_set() {
                break;
            }
        }
    }
    net.vars.posted_result.get()
}

/// The call posted, taken by the receiving thread (none waiting: `None`).
pub fn take_posted(vars: &NetShared) -> Option<PostedCall> {
    if vars.posted_call.atomic().swap(0, Ordering::SeqCst) == 0 {
        return None;
    }
    POSTED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}
