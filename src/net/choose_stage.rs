//! `net::choose_stage` (0x418347): the lobby's stage item (the host's): the stage screen
//! (`menu::select_stage_loop`, the Escape flag `dword_755A3C` 0 first); unless it was left
//! with Escape, this player's record given the stage (+4 from `dword_75ACD4`) and its
//! direction (+5 from `dword_75ACF0`); then the background again (`net::show_background`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::lobby_loop`. Its calls through `part` (the stage screen through its hook).
//! The screen runs inside it: ungated. Shadow mode runs the original alone. Self-test: a copy
//! whose calls go to stand-ins, against the Rust version, left with Escape or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::choose_car::ESCAPED;
use crate::net::fill_own_player::{DIRECTION, STAGE};
use crate::net::lobby::{
    OWN, Part, SELECT_STAGE, Saved, answer, dispatch_lobby, dword, g, p, pb, region, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::choose_stage", rva(0x418347), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const SHOW_BACKGROUND: u32 = 0x4173E7; // net::show_background

/// The Rust version: the registers it leaves (the entry's).
pub fn choose_stage(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(ESCAPED, 0);
    part(SELECT_STAGE, &mut r)?;
    if g(ESCAPED) != 1 {
        r.eax = g(STAGE);
        pb(OWN + 4, r.eax as u8);
        r.eax = g(DIRECTION);
        pb(OWN + 5, r.eax as u8);
    }
    part(SHOW_BACKGROUND, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, choose_stage);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov dword_755A3C, 0
    let copy = match crate::net::lobby::copy(
        0x418347,
        0x34,
        &[
            0x60, 0xC7, 0x05, 0x3C, 0x5A, 0x75, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        &[0xB, 0x2D],
        &[SELECT_STAGE, SHOW_BACKGROUND],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [
        dword(ESCAPED),
        region(OWN, 8),
        dword(STAGE),
        dword(DIRECTION),
    ];
    let saved = Saved::new(&all);
    for escaped in [0u32, 1] {
        run_both(
            &mut t,
            &format!("Escape {escaped}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            0,
            &[],
            &|| {
                p(STAGE, 0x1234);
                p(DIRECTION, 0x101);
                answer(Box::new(move |place, _| {
                    if place == SELECT_STAGE {
                        p(ESCAPED, escaped);
                    }
                }));
            },
            &choose_stage,
        );
    }
    drop(saved);
    t.finish()
}
