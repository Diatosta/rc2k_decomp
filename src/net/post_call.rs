//! `net::post_call` (0x4184F8): with `g_netPosting` 1, the code at eax handed to the
//! receiving thread (`g_netPostedCall`), which calls it and sets `g_netPostedResult` and
//! `g_netPostedAnswered`; meanwhile a tenth of a second's sleep (`system::sleep`) and the
//! window's messages (`system::pump_messages`) until it has answered or the game is
//! closing; then eax = `g_netPostedResult` (also without posting).
//!
//! In:  `code` (eax: the code to call; here a Rust function, handed over beside the cell,
//!      which holds 1 while it waits: phase 11, 2026-10-26)
//! Out: the posted call's result (eax). The original leaves the rest as the sleep and the
//!      messages leave them, and the flags of its last test: its callers read eax only
//!      (`lq -s 0x4184F8`).
//! DirectPlay's calls that may show a dialog (a modem's, a serial link's) are made on the
//! receiving thread this way (`net::build_address` sets `g_netPosting`). Callers:
//! `net::find_sessions`, `net::host_session`, `net::join_session`, each posting its own
//! piece of code (0x418A43, 0x418C5A, 0x418D77: Rust versions of them). It waits on
//! another thread.

use std::sync::Mutex;
use std::sync::atomic::Ordering;

/// The call posted, while `g_netPostedCall` is 1.
static POSTED: Mutex<Option<fn() -> u32>> = Mutex::new(None);

/// The Rust version.
pub fn post_call(code: fn() -> u32) -> u32 {
    if crate::net::owner::net().vars.posting.get() == 1 {
        *POSTED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(code);
        crate::net::owner::net().vars.posted_call.set(1);
        loop {
            let answered = crate::net::owner::net()
                .vars
                .posted_answered
                .atomic()
                .swap(0, Ordering::SeqCst);
            if answered != 0 {
                break;
            }
            crate::system::sleep::sleep(100);
            crate::platform::pump_messages_now();
            if crate::system::owner::system().session.flags.quitting.get() == 1 {
                break;
            }
        }
    }
    crate::net::owner::net().vars.posted_result.get()
}

/// The call posted, taken by the receiving thread (none waiting: `None`).
pub fn take_posted() -> Option<fn() -> u32> {
    if crate::net::owner::net()
        .vars
        .posted_call
        .atomic()
        .swap(0, Ordering::SeqCst)
        == 0
    {
        return None;
    }
    POSTED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}
