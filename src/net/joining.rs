//! `net::joining` (0x4179B6): "joining" shown (string 0x153, as `net::connecting` shows
//! its), then session eax joined (`net::join_session`).
//!
//! In:  eax = the session's number
//! Out: every register kept (`pusha` .. `popa`); the flags `net::join_session` leaves
//!      (carry: it failed)
//! Caller: `net::start_link`. Its calls through `part` (the frames through the game).
//! Shadow mode runs the original alone. Self-test: as `net::connecting`'s.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::net::connecting::{status, status_test};
use crate::net::lobby::{Part, dispatch_lobby};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::joining", rva(0x4179B6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const JOIN_SESSION: u32 = 0x418C6D; // net::join_session

/// The Rust version: the registers it leaves (the entry's, the call's flags).
pub fn joining(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.esi = r.eax;
    status(&mut r, 0x153, part)?;
    r.eax = r.esi;
    part(JOIN_SESSION, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, joining);
}

fn self_test() -> (usize, usize) {
    // pusha; mov esi, eax; xor eax, eax
    status_test(
        &HOOK,
        0x4179B6,
        0x4D,
        &[0x60, 0x8B, 0xF0, 0x33, 0xC0],
        &[0x5, 0xA, 0x11, 0x2B, 0x30, 0x35, 0x3F, 0x46],
        JOIN_SESSION,
        joining,
    )
}
