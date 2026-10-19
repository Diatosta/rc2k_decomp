//! `net::chat_line_locked` (0x4193FF): `net::add_chat_line` with the received messages' lock
//! held (`net::lock_received` .. `net::unlock_received`): a chat message another player sent.
//!
//! In:  eax = the player, ebx = the message
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: the receiving thread's message handling (0x41959x, not ported). Its calls through
//! `part`. Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{LOCK_RECEIVED, Part, UNLOCK_RECEIVED, dispatch_lobby, run_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::chat_line_locked", rva(0x4193FF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const ADD_CHAT_LINE: u32 = 0x419428; // net::add_chat_line

/// The Rust version: the registers it leaves (the entry's).
pub fn chat_line_locked(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(LOCK_RECEIVED, &mut r)?;
    part(ADD_CHAT_LINE, &mut r)?;
    part(UNLOCK_RECEIVED, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, chat_line_locked);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call net::lock_received
    let copy = match crate::net::lobby::copy(
        0x4193FF,
        0x12,
        &[0x60, 0xE8, 0x6F, 0xF0, 0xFF, 0xFF],
        &[0x1, 0x6, 0xB],
        &[LOCK_RECEIVED, ADD_CHAT_LINE, UNLOCK_RECEIVED],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    run_both(
        &mut t,
        "a message",
        copy.addr(),
        Regs {
            eax: 3,
            ebx: 0x0012_3456,
            ..Regs::SENTINEL
        },
        &[],
        0,
        &[],
        &|| {},
        &chat_line_locked,
    );
    t.finish()
}
