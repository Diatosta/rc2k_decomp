//! `net::post_call` (0x4184F8): with `g_netPosting` 1, the code at eax handed to the
//! receiving thread (`g_netPostedCall`), which calls it and sets `g_netPostedResult` and
//! `g_netPostedAnswered`; meanwhile a tenth of a second's sleep (`system::sleep`) and the
//! window's messages (`system::pump_messages`) until it has answered or the game is
//! closing; then eax = `g_netPostedResult` (also without posting).
//!
//! In:  eax = the code to call
//! Out: eax = the posted call's result; the rest as the sleep and the messages leave them;
//!      the flags of the last test (no caller reads them)
//! DirectPlay's calls that may show a dialog (a modem's, a serial link's) are made on the
//! receiving thread this way (`net::build_address` sets `g_netPosting`). Callers:
//! `net::find_sessions`, `net::host_session`, `net::join_session`, each posting its own
//! piece of code (0x418A43, 0x418C5A, 0x418D77: `net::lobby_calls` posts Rust versions of
//! them). It waits on another thread: ungated. Shadow mode runs the original alone. Its
//! calls through `part`. Self-test: a copy whose five variables are the test's (the
//! receiving thread reads them) and whose calls go to stand-ins (one answering after some
//! turns), against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    POSTED, POSTED_ANSWERED, POSTED_RESULT, POSTING, PUMP, Part, QUITTING, SLEEP, a, lobby_entry,
};
use std::sync::atomic::{AtomicU32, Ordering};

/// Where the variables are.
#[derive(Clone, Copy)]
pub struct Vars {
    pub posting: u32,
    pub posted: u32,
    pub result: u32,
    pub answered: u32,
    pub quitting: u32,
}

impl Vars {
    pub fn game() -> Self {
        Self {
            posting: a(POSTING),
            posted: a(POSTED),
            result: a(POSTED_RESULT),
            answered: a(POSTED_ANSWERED),
            quitting: a(QUITTING),
        }
    }
}

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}

/// The Rust version on the variables in `v`.
pub fn post_call_with(entry: &Regs, v: &Vars, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if get(v.posting) == 1 {
        unsafe { (v.posted as *mut u32).write_volatile(r.eax) };
        loop {
            r.eax = 0;
            let answered =
                unsafe { AtomicU32::from_ptr(v.answered as *mut u32) }.swap(0, Ordering::SeqCst);
            if answered != 0 {
                break;
            }
            r.eax = 100;
            part(SLEEP, &mut r)?;
            part(PUMP, &mut r)?;
            if get(v.quitting) == 1 {
                break;
            }
        }
    }
    r.eax = get(v.result);
    Ok(r)
}

/// The Rust version.
pub fn post_call(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    post_call_with(entry, &Vars::game(), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, post_call)
}
