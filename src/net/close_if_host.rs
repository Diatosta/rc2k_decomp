//! `net::close_if_host` (0x41837B): on the host (state bit 0x10 of `byte_52AD64`), no more
//! players let in (`net::disable_new_players`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of the test, or the call's (no
//!      caller reads them)
//! Caller: `net::start_link`, before each race. Its call through `part`. Shadow mode runs
//! the original alone. Self-test: a copy whose call goes to a stand-in, against the Rust
//! version, host or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{OWN, Part, Saved, dispatch_lobby, gb, pb, region, run_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::close_if_host", rva(0x41837B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DISABLE_NEW_PLAYERS: u32 = 0x418D8A; // net::disable_new_players

/// The Rust version: the registers it leaves (the entry's).
pub fn close_if_host(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if gb(OWN) & 0x10 != 0 {
        part(DISABLE_NEW_PLAYERS, &mut r)?;
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, close_if_host);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; test byte_52AD64, 10h
    let copy = match crate::net::lobby::copy(
        0x41837B,
        0x11,
        &[0x60, 0xF6, 0x05, 0x64, 0xAD, 0x52, 0x00, 0x10],
        &[0xA],
        &[DISABLE_NEW_PLAYERS],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [region(OWN, 1)];
    let saved = Saved::new(&all);
    for state in [0u8, 0x10, 0xEF, 0x13] {
        run_both(
            &mut t,
            &format!("state {state:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            0,
            &[],
            &|| pb(OWN, state),
            &close_if_host,
        );
    }
    drop(saved);
    t.finish()
}
