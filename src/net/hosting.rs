//! `net::hosting` (0x4176D5): "hosting" shown (string 0x151, as `net::connecting` shows
//! its), then the session made (`net::host_session`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags `net::host_session` leaves
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
    RawHook::new("net::hosting", rva(0x4176D5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const HOST_SESSION: u32 = 0x418B13; // net::host_session

/// The Rust version: the registers it leaves (the entry's, the call's flags).
pub fn hosting(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    status(&mut r, 0x151, part)?;
    part(HOST_SESSION, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, hosting);
}

fn self_test() -> (usize, usize) {
    // pusha; xor eax, eax; call screens::run_frame
    status_test(
        &HOOK,
        0x4176D5,
        0x49,
        &[0x60, 0x33, 0xC0, 0xE8, 0x94, 0xE3, 0xFE, 0xFF],
        &[0x3, 0x8, 0xF, 0x29, 0x2E, 0x33, 0x3D, 0x42],
        HOST_SESSION,
        hosting,
    )
}
