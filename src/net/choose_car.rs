//! `net::choose_car` (0x4182FA): the lobby's car item: `dword_75598C` 0, the car screen
//! (`menu::select_car_loop` on `net::car_list`'s list, from player 1's car, the Escape flag
//! `dword_755A3C` 0 first); unless it was left with Escape, the car taken
//! (`menu::take_shown_car`) and this player's record given it (+1 the car, +2 its colours);
//! then the background again (`net::show_background`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::lobby_loop`. Its calls through `part` (the car screen through its hook).
//! The screen runs inside it: ungated. Shadow mode runs the original alone. Self-test: a copy
//! whose calls go to stand-ins, against the Rust version, left with Escape or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    OWN, Part, SELECT_CAR, Saved, TAKE_SHOWN_CAR, answer, dispatch_lobby, dword, g, p, pb, region,
    run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::choose_car", rva(0x4182FA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const CAR_LIST: u32 = 0x4AAAF7; // net::car_list
const SHOW_BACKGROUND: u32 = 0x4173E7; // net::show_background
const CLEARED: usize = 0x75598C; // dword_75598C
pub const ESCAPED: usize = 0x755A3C; // dword_755A3C: the screen was left with Escape
const CAR: usize = 0x755A54; // g_player1.car_number
const COLOURS: usize = 0x755A7C; // g_player1.field_30

/// The Rust version: the registers it leaves (the entry's).
pub fn choose_car(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(CLEARED, 0);
    part(CAR_LIST, &mut r)?;
    p(ESCAPED, 0);
    r.eax = g(CAR);
    part(SELECT_CAR, &mut r)?;
    if g(ESCAPED) != 1 {
        part(TAKE_SHOWN_CAR, &mut r)?;
        r.eax = g(CAR);
        pb(OWN + 1, r.eax as u8);
        r.eax = g(COLOURS);
        pb(OWN + 2, r.eax as u8);
    }
    part(SHOW_BACKGROUND, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, choose_car);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov dword_75598C, 0
    let copy = match crate::net::lobby::copy(
        0x4182FA,
        0x4D,
        &[
            0x60, 0xC7, 0x05, 0x8C, 0x59, 0x75, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        &[0xB, 0x1F, 0x2D, 0x46],
        &[CAR_LIST, SELECT_CAR, TAKE_SHOWN_CAR, SHOW_BACKGROUND],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [dword(CLEARED), dword(ESCAPED), region(OWN, 4)];
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
                p(CLEARED, 9);
                answer(Box::new(move |place, r| match place {
                    CAR_LIST => r.esi = 0x0075_9E52,
                    SELECT_CAR => p(ESCAPED, escaped),
                    _ => {}
                }));
            },
            &choose_car,
        );
    }
    drop(saved);
    t.finish()
}
