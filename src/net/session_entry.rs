//! `net::session_entry` (0x418553): the record of session eax (0x68 bytes each in the table
//! `dword_52ACF8`, `dword_52ACF4` of them): +0 its name (0x40 bytes), +0x40 its instance
//! GUID, +0x50 the most players, +0x54 the players in it, +0x58 its password GUID (a
//! reading of what `net::session_found` copies from the session's description).
//!
//! In:  eax = the session's number
//! Out: eax = its record; the flags of the `add` (no caller reads them)
//! A number not below the count (unsigned) is fatal error 0x80A61030, no table fatal error
//! 0x80A61031. Callers: `net::choose_session`, `net::sessions_frame`, `net::session_found`,
//! `net::join_session`. Shadow mode: both run, eax compared. Self-test: as
//! `net::connection_entry`'s.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::net::lobby::{NO_SESSION, NO_SESSIONS, Part, SESSION_TABLE, SESSIONS, session};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::session_entry", rva(0x418553), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers it leaves.
pub fn session_entry(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::net::lists::record` (in `net::lobby::record_of`).
    Ok(Regs {
        eax: session(entry.eax)?,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_regs(&HOOK, r, call_original, &[], 0, &|r| {
        session_entry(r, &mut crate::net::lobby::game_part)
    });
}

fn self_test() -> (usize, usize) {
    crate::net::connection_entry::entry_test(
        &HOOK,
        0x418553,
        [0x3B, 0x05, 0xF4, 0xAC, 0x52, 0x00],
        SESSIONS,
        SESSION_TABLE,
        [NO_SESSION, NO_SESSIONS],
        session_entry,
    )
}
