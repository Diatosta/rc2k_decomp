//! `net::unlock_received` (0x418481): the received messages' mutex (`g_netMutex`) given
//! back: `system::release_mutex` (0x4110B9: `ReleaseMutex(eax)`, the flags of
//! `cmp result, 1`) inside `pusha` .. `popa`.
//!
//! In:  nothing
//! Out: every register kept; the flags `release_mutex` left (zero: released; no caller
//!      reads them)
//! Six callers, those of `net::lock_received`. The release through `part` (in the game
//! `ReleaseMutex` from Rust, `net::game_part`). Refuses nothing. Shadow mode runs the
//! original alone. Self-test: as `net::lock_received`'s.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::net::message_vars::{MessageVars, get};
use crate::net::{Part, RELEASE_MUTEX, dispatch_net, net_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::unlock_received", rva(0x418481), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version on the variables at `v`.
pub fn unlock_received_with(r: &mut Regs, v: &MessageVars, part: Part) -> Result<(), Refused> {
    let mut c = Regs {
        eax: get(v.mutex),
        ..*r
    };
    part(RELEASE_MUTEX, &mut c)?;
    r.eflags = c.eflags;
    Ok(())
}

/// The Rust version.
pub fn unlock_received(r: &mut Regs, part: Part) -> Result<(), Refused> {
    unlock_received_with(r, &MessageVars::game(), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, unlock_received);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, unlock_received)
}

fn self_test() -> (usize, usize) {
    // pusha; mov eax, g_netMutex
    crate::net::lock_received::test_wrapper(
        &HOOK,
        0x418481,
        &[0x60, 0xA1, 0x6C, 0xAB, 0x52, 0x00],
        RELEASE_MUTEX,
        unlock_received_with,
    )
}
