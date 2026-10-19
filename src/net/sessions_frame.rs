//! `net::sessions_frame` (0x41790E): the frame of `net::choose_session`, as
//! `net::connections_frame`'s: the title string 0x150 (0x152 with no session listed), the
//! sessions' names (`net::session_entry`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Called by `screens::run_frame` for `net::choose_session`. Its calls through `part`.
//! Shadow mode runs the original alone (it draws). Self-test: as `net::connections_frame`'s.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::net::connections_frame::{list_frame, list_frame_test};
use crate::net::lobby::{Part, SESSIONS, dispatch_lobby, g};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::sessions_frame", rva(0x41790E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SESSION_ENTRY: u32 = 0x418553; // net::session_entry

/// The Rust version: the registers it leaves (the entry's).
pub fn sessions_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let title = if g(SESSIONS) != 0 { 0x150 } else { 0x152 };
    list_frame(entry, title, SESSIONS, SESSION_ENTRY, 0x51E744, part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, sessions_frame);
}

fn self_test() -> (usize, usize) {
    // pusha; call gfx::lock_surface
    list_frame_test(
        &HOOK,
        0x41790E,
        0xA8,
        &[0x60, 0xE8, 0x99, 0xDF, 0xFE, 0xFF],
        &[0x1, 0xC, 0x39, 0x7E, 0x90, 0x9C, 0xA1],
        SESSIONS,
        SESSION_ENTRY,
        sessions_frame,
    )
}
