//! `net::race_over` (0x41732E): this player's race done: `dword_52AD78` and `dword_52AD7C` 0,
//! the race's time (eax) to the record (+8) and added to its total (+0xC), its state
//! "raced" (bits 0 and 1 off, bit 2 on), "after a race" (`dword_52AD80` 1), and the players
//! listed (`net::list_players_regs`: the record sent).
//!
//! In:  eax = the race's time
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::lets_rally`. Its call through `part`. Shadow mode runs the original alone.
//! Self-test: a copy whose call goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    ALL_RACED, BACK_FROM_RACE, OWN, OWN_TIME, OWN_TOTAL, Part, RACE_OVER, Saved, dispatch_lobby,
    dword, g, gb, p, pb, region, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::race_over", rva(0x41732E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LIST_PLAYERS: u32 = 0x417A03; // net::list_players_regs

/// The Rust version: the registers it leaves (the entry's).
pub fn race_over(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(ALL_RACED, 0);
    p(RACE_OVER, 0);
    p(OWN_TIME, r.eax);
    p(OWN_TOTAL, g(OWN_TOTAL).wrapping_add(r.eax));
    // Converted: `rc2k_core::net::players::raced`.
    pb(OWN, rc2k_core::net::players::raced(gb(OWN)));
    p(BACK_FROM_RACE, 1);
    part(LIST_PLAYERS, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, race_over);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov dword_52AD78, 0
    let copy = match crate::net::lobby::copy(
        0x41732E,
        0x3F,
        &[
            0x60, 0xC7, 0x05, 0x78, 0xAD, 0x52, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        &[0x38],
        &[LIST_PLAYERS],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [region(OWN, 0x20)];
    let saved = Saved::new(&all);
    for time in [12345u32, 0xFFFF_FFF0] {
        run_both(
            &mut t,
            &format!("time {time:#x}"),
            copy.addr(),
            Regs {
                eax: time,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                pb(OWN, 0x13);
                p(OWN_TOTAL, 100);
                p(ALL_RACED, 1);
                p(RACE_OVER, 1);
            },
            &race_over,
        );
    }
    drop(saved);
    let _ = dword;
    t.finish()
}
