//! `net::chat_key` (0x4180E4): a key for the chat message being typed (`net::edit_key`); when
//! that ends the editing and the message is not empty, it is sent (`net::send_chat_regs`,
//! this machine's player) and the input emptied.
//!
//! In:  al = the key
//! Out: as `net::edit_key` leaves them while editing goes on; after it eax = the player,
//!      ebx = the input; no `pusha`
//! Caller: `net::lobby_loop`. Its calls through `part`. Shadow mode runs the original alone
//! (it sends). Self-test: a copy whose calls go to stand-ins, against the Rust version:
//! editing on, ended with a message, ended empty.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva};
use crate::net::lobby::{
    CHAT_INPUT, PLAYER, Part, Saved, a, answer, dispatch_lobby, dword, g, gb, p, pb, region,
    run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::chat_key", rva(0x4180E4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const EDIT_KEY: u32 = 0x416E43; // net::edit_key
const SEND_CHAT: u32 = 0x41819B; // net::send_chat_regs

/// The Rust version: the registers it leaves.
pub fn chat_key(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(EDIT_KEY, &mut r)?;
    if r.eflags & ZERO_FLAG == 0 {
        return Ok(r);
    }
    r.eax = g(PLAYER);
    r.ebx = a(CHAT_INPUT);
    if gb(CHAT_INPUT) != 0 {
        part(SEND_CHAT, &mut r)?;
        pb(CHAT_INPUT, 0);
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, chat_key);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // call sub_416E43
    let copy = match crate::net::lobby::copy(
        0x4180E4,
        0x24,
        &[0xE8, 0x5A, 0xED, 0xFF, 0xFF],
        &[0x0, 0x17],
        &[EDIT_KEY, SEND_CHAT],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [dword(PLAYER), region(CHAT_INPUT, 4)];
    let saved = Saved::new(&all);
    for (editing, first) in [(true, b'h'), (false, b'h'), (false, 0)] {
        run_both(
            &mut t,
            &format!("editing {editing}, the input's first byte {first:#x}"),
            copy.addr(),
            Regs {
                eax: 0x1C,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                p(PLAYER, 2);
                pb(CHAT_INPUT, first);
                answer(Box::new(move |place, r| {
                    if place == EDIT_KEY {
                        r.set_zero(!editing);
                    }
                }));
            },
            &chat_key,
        );
    }
    drop(saved);
    t.finish()
}
