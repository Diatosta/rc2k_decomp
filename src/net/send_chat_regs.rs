//! `net::send_chat_regs` (0x41819B): `net::send_chat` inside `pusha` .. `popa`.
//!
//! In:  eax = the player, ebx = the message
//! Out: every register kept; the flags `net::send_chat` leaves (no caller reads them)
//! Caller: `net::chat_key`. Its call through `part`. Shadow mode runs the original alone.
//! Self-test: a copy whose call goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{Part, answer, dispatch_lobby, run_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::send_chat_regs", rva(0x41819B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SEND_CHAT: u32 = 0x4193C8; // net::send_chat

/// The Rust version: the registers it leaves (the entry's).
pub fn send_chat_regs(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(SEND_CHAT, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, send_chat_regs);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call sub_4193C8
    let copy = match crate::net::lobby::copy(
        0x41819B,
        0x8,
        &[0x60, 0xE8, 0x27, 0x12, 0x00, 0x00],
        &[0x1],
        &[SEND_CHAT],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    run_both(
        &mut t,
        "a message",
        copy.addr(),
        Regs {
            eax: 1,
            ebx: 0x0052_A4C8,
            ..Regs::SENTINEL
        },
        &[],
        crate::framework::CARRY_FLAG | crate::framework::ZERO_FLAG,
        &[],
        &|| {
            answer(Box::new(|_, r| {
                r.eax = 0x99;
                r.ecx = 0x98;
                r.set_zero(true);
                r.set_carry(true);
            }))
        },
        &send_chat_regs,
    );
    t.finish()
}
