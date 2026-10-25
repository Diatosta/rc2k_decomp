//! `net::post_call` (0x4184F8): with `g_netPosting` 1, the code at eax handed to the
//! receiving thread (`g_netPostedCall`), which calls it and sets `g_netPostedResult` and
//! `g_netPostedAnswered`; meanwhile a tenth of a second's sleep (`system::sleep`) and the
//! window's messages (`system::pump_messages`) until it has answered or the game is
//! closing; then eax = `g_netPostedResult` (also without posting).
//!
//! In:  `code` (eax: the code to call; here a Rust version's address, `net::lobby_calls`)
//! Out: the posted call's result (eax). The original leaves the rest as the sleep and the
//!      messages leave them, and the flags of its last test: its callers read eax only
//!      (`lq -s 0x4184F8`).
//! DirectPlay's calls that may show a dialog (a modem's, a serial link's) are made on the
//! receiving thread this way (`net::build_address` sets `g_netPosting`). Callers:
//! `net::find_sessions`, `net::host_session`, `net::join_session`, each posting its own
//! piece of code (0x418A43, 0x418C5A, 0x418D77: Rust versions of them). It waits on
//! another thread.

use crate::net::store::NET;
use crate::session::FLAGS;
use std::sync::atomic::{AtomicU32, Ordering};

/// The Rust version.
pub fn post_call(code: u32) -> u32 {
    if NET.posting.get() == 1 {
        unsafe { ((NET.posted_call.addr() as u32) as *mut u32).write_volatile(code) };
        loop {
            let answered =
                unsafe { AtomicU32::from_ptr((NET.posted_answered.addr() as u32) as *mut u32) }
                    .swap(0, Ordering::SeqCst);
            if answered != 0 {
                break;
            }
            crate::system::sleep::sleep(100);
            crate::platform::pump_messages_now();
            if FLAGS.quitting.get() == 1 {
                break;
            }
        }
    }
    unsafe { ((NET.posted_result.addr() as u32) as *const u32).read_volatile() }
}
