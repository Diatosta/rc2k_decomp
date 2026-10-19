//! `net::send_chat` (0x4193C8): this player's chat message added to the chat
//! (`net::add_chat_line`, under the received messages' lock) and sent to everyone
//! (`g_netSendTo`, DirectPlay's `Send` through `net::dplay_send`) as '"' and the text, no NUL.
//!
//! In:  eax = the player, ebx = the message
//! Out: every register kept (`pusha` .. `popa`); the flags of `test eax, eax` on the send's
//!      eax (no caller reads them)
//! Caller: `net::send_chat_regs`. Its calls through `part` (`net::lobby_calls`: the Rust
//! versions, the send through the game). Shadow mode runs the original alone (it sends).
//! Self-test: a copy whose calls go to stand-ins (`str_append` done), against the Rust
//! version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    LOCK_RECEIVED, Part, SEND, SEND_BUFFER, SEND_TO, STR_APPEND, Saved, UNLOCK_RECEIVED, a, answer,
    dispatch_lobby, dword, fill, g, p, region, run_both, test_eax,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::send_chat", rva(0x4193C8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const ADD_CHAT_LINE: u32 = 0x419428; // net::add_chat_line

/// The Rust version: the registers it leaves (the entry's).
pub fn send_chat(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(LOCK_RECEIVED, &mut r)?;
    part(ADD_CHAT_LINE, &mut r)?;
    part(UNLOCK_RECEIVED, &mut r)?;
    r.eax = r.ebx;
    let buffer = a(SEND_BUFFER);
    unsafe { (buffer as *mut u8).write(b'"') };
    r.ebx = buffer + 1;
    part(STR_APPEND, &mut r)?;
    r.ecx = r.ebx.wrapping_sub(buffer);
    r.ebx = buffer;
    r.eax = g(SEND_TO);
    r.edx = 0;
    part(SEND, &mut r)?;
    test_eax(&mut r);
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, send_chat);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call net::lock_received
    let copy = match crate::net::lobby::copy(
        0x4193C8,
        0x37,
        &[0x60, 0xE8, 0xA6, 0xF0, 0xFF, 0xFF],
        &[0x1, 0x6, 0xB, 0x1D, 0x2E],
        &[
            LOCK_RECEIVED,
            ADD_CHAT_LINE,
            UNLOCK_RECEIVED,
            STR_APPEND,
            SEND,
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [region(SEND_BUFFER, 0x40), dword(SEND_TO)];
    let saved = Saved::new(&all);
    for message in [&b"\0"[..], b"hello there\0"] {
        run_both(
            &mut t,
            &format!("{:?}", String::from_utf8_lossy(message)),
            copy.addr(),
            Regs {
                eax: 2,
                ebx: message.as_ptr() as u32,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                fill(SEND_BUFFER, 0x40, 12);
                p(SEND_TO, 0x0000_7777);
                answer(crate::net::add_chat_line::chat_answers());
            },
            &send_chat,
        );
    }
    drop(saved);
    t.finish()
}
