//! `net::list_players_regs` (0x417A03): `net::list_players`, then `test eax, eax` on the eax it leaves (its own, since it
//! keeps every register), inside `pusha` .. `popa`: the carry it gives is lost, so its
//! callers' `jb` after it are never taken (section 16).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of `test eax, eax` (carry clear)
//! Callers: `net::lets_rally`, `net::race_wait`, `net::race_over`, `net::lobby_loop`. Its call through `part`. Shadow mode runs the original alone. Self-test: a copy whose call goes to a
//! stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{FLAGS, Part, answer, dispatch_lobby, run_both, test_eax};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::list_players_regs", rva(0x417A03), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CALLEE: u32 = 0x419192; // net::list_players

/// The Rust version: the registers it leaves (the entry's, the flags of `test eax, eax`).
pub fn list_players_regs(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(CALLEE, &mut r)?;
    test_eax(&mut r);
    let _ = test_eax;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, list_players_regs);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match crate::net::lobby::copy(
        0x417A03,
        0xA,
        &[0x60, 0xE8, 0x89, 0x17, 0x00, 0x00],
        &[0x1],
        &[CALLEE],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };

    for (eax, carry) in [(0u32, false), (5, true), (0x8000_0000, false)] {
        run_both(
            &mut t,
            &format!("eax {eax:#x}, the call's carry {carry}"),
            copy.addr(),
            Regs {
                eax,
                ..Regs::SENTINEL
            },
            &[],
            FLAGS,
            &[],
            &|| {
                answer(Box::new(move |_, r| {
                    r.eax = 0x77;
                    r.ebx = 0x66;
                    r.set_carry(carry);
                    r.set_zero(!carry);
                }))
            },
            &list_players_regs,
        );
    }

    t.finish()
}
