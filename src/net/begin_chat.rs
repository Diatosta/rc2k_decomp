//! `net::begin_chat` (0x4180C5): Enter on the lobby's chat item starts a message: the input
//! (`byte_52A4C8`) emptied and edited (`net::begin_edit`, the item `g_listChoice`, width
//! 0x30C, measured with `dword_51E424`).
//!
//! In:  nothing
//! Out: eax = the item, ebx = the input, ecx = 0x30C, edx = `dword_51E424`'s address; no
//!      `pusha`
//! Caller: `net::lobby_loop`. Its call through `part`. Shadow mode runs the original alone.
//! Self-test: a copy whose call goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    CHAT_INPUT, CHOICE, Part, Saved, a, dispatch_lobby, dword, g, p, pb, region, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::begin_chat", rva(0x4180C5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const BEGIN_EDIT: u32 = 0x416E1A; // net::begin_edit
const CHAT_FONT: usize = 0x51E424; // dword_51E424

/// The Rust version: the registers it leaves.
pub fn begin_chat(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = g(CHOICE);
    r.ebx = a(CHAT_INPUT);
    pb(CHAT_INPUT, 0);
    r.ecx = 0x30C;
    r.edx = a(CHAT_FONT);
    part(BEGIN_EDIT, &mut r)?;
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, begin_chat);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, g_listChoice
    let copy = match crate::net::lobby::copy(
        0x4180C5,
        0x1F,
        &[0xA1, 0x07, 0xD4, 0x51, 0x00],
        &[0x19],
        &[BEGIN_EDIT],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [dword(CHOICE), region(CHAT_INPUT, 4)];
    let saved = Saved::new(&all);
    run_both(
        &mut t,
        "the chat item",
        copy.addr(),
        Regs::SENTINEL,
        &all,
        0,
        &[],
        &|| {
            p(CHOICE, 3);
            pb(CHAT_INPUT, b'x');
        },
        &begin_chat,
    );
    drop(saved);
    t.finish()
}
