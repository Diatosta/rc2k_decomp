//! `net::lobby_button` (0x41802F): one of the lobby's buttons, right to left: drawn
//! (`net::draw_button`, flags 1, x edi, y ebp), dim (flag 2) while this player is ready (state & 1)
//! for all but item 3, and for item 0 after a race (`dword_52AD80` 1); edi then
//! moves left past it (its width `dword_5AB4A0` + 0x44, and 8 more). (The code after its
//! `ret`, 0x41806E .. 0x418082, is reached by nothing.)
//!
//! In:  eax = the string, esi = the item, edi = x, ebp = y
//! Out: ebx = the flags, ecx = the width + 0x44, edx = y, edi moved; the rest kept; no
//!      `pusha`
//! Caller: `net::lobby_frame` (four times). Its call through `part`. Shadow mode runs the
//! original alone (it draws). Self-test: a copy whose call goes to a stand-in, against the
//! Rust version, on each item, ready or not, after a race or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    BACK_FROM_RACE, OWN, Part, Saved, dispatch_lobby, dword, g, gb, p, pb, region, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::lobby_button", rva(0x41802F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DRAW_BUTTON: u32 = 0x416CE0; // net::draw_button
const WIDTH: usize = 0x5AB4A0; // dword_5AB4A0

/// The Rust version: the registers it leaves.
pub fn lobby_button(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.ebx = 1;
    r.ecx = r.edi;
    r.edx = r.ebp;
    if gb(OWN) & 1 != 0 && r.esi != 3 {
        r.ebx |= 2;
    }
    if g(BACK_FROM_RACE) == 1 && r.esi == 0 {
        r.ebx |= 2;
    }
    part(DRAW_BUTTON, &mut r)?;
    r.ecx = g(WIDTH).wrapping_add(0x44);
    r.edi = r.edi.wrapping_sub(r.ecx).wrapping_sub(8);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, lobby_button);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov ebx, 1
    let copy = match crate::net::lobby::copy(
        0x41802F,
        0x3F,
        &[0xBB, 0x01, 0x00, 0x00, 0x00],
        &[0x2B],
        &[DRAW_BUTTON],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [region(OWN, 1), dword(BACK_FROM_RACE), dword(WIDTH)];
    let saved = Saved::new(&all);
    for item in 0..4u32 {
        for (state, back) in [(0u8, 0u32), (1, 0), (0, 1), (3, 1)] {
            run_both(
                &mut t,
                &format!("item {item}, state {state}, after a race {back}"),
                copy.addr(),
                Regs {
                    eax: 0x15D,
                    esi: item,
                    edi: 0x310,
                    ebp: 0x223,
                    ..Regs::SENTINEL
                },
                &all,
                0,
                &[],
                &|| {
                    pb(OWN, state);
                    p(BACK_FROM_RACE, back);
                    p(WIDTH, 0x50);
                },
                &lobby_button,
            );
        }
    }
    drop(saved);
    t.finish()
}
