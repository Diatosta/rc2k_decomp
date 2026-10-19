//! `net::clear_chat` (0x419411): the lobby's chat (`byte_52A508`) made eight empty lines:
//! seven 0x0A and a NUL.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::start_link` (with the received messages' lock held). Shadow mode: both run,
//! the eight bytes compared. Self-test: a copy against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{CHAT, Part, Saved, a, fill, region, run_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::clear_chat", rva(0x419411), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers it leaves (the entry's).
pub fn clear_chat(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    unsafe { (a(CHAT) as *mut [u8; 8]).write_unaligned(*b"\n\n\n\n\n\n\n\0") };
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_regs(&HOOK, r, call_original, &[region(CHAT, 8)], 0, &|r| {
        clear_chat(r, &mut crate::net::lobby::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov eax, 0Ah
    let copy = match crate::net::lobby::copy(
        0x419411,
        0x17,
        &[0x60, 0xB8, 0x0A, 0x00, 0x00, 0x00],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [region(CHAT, 0x20)];
    let saved = Saved::new(&all);
    run_both(
        &mut t,
        "the chat",
        copy.addr(),
        Regs::SENTINEL,
        &all,
        0,
        &[],
        &|| fill(CHAT, 0x20, 2),
        &clear_chat,
    );
    drop(saved);
    t.finish()
}
