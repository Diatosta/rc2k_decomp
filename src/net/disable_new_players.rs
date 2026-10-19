//! `net::disable_new_players` (0x418D8A): on the host (state bit 0x10 of `byte_52AD64`), "NET:
//! Disable new players", and with DirectPlay's object there the session's description
//! (`dword_5AB2AC`) given DPSESSION_NEWPLAYERSDISABLED and DPSESSION_JOINDISABLED (0x21) and
//! set (`SetSessionDesc`, flags 0); its result logged.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if not the host, no object or it
//!      failed, else the flags of `test eax, eax`
//! Caller: `net::close_if_host`. Its calls through `part`; `SetSessionDesc` from Rust
//! (`net::dplay`). Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins and whose DirectPlay is a test object, against the Rust version: not the host,
//! no object, failing, done.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{SET_SESSION_DESC, call};
use crate::net::lobby::{
    DPLAY, DPLAY_METHOD, FLAGS, LOG_HEX_L2_NL, LOG_L2, OWN, Part, SESSION_DESC, Saved, TestObject,
    a, cmp_flags, com_answer, dispatch_lobby, dword, g, gb, log_text, p, pb, region, run_both,
    set_sign, test_eax,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::disable_new_players", rva(0x418D8A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXT: usize = 0x5C2F26; // "NET: Disable new players"
const CLOSED: u32 = 0x21; // DPSESSION_NEWPLAYERSDISABLED | DPSESSION_JOINDISABLED

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn disable_new_players(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    if gb(OWN) & 0x10 == 0 {
        // The flags of `test byte_52AD64, 10h`, and the carry.
        r.set_zero(true);
        set_sign(&mut r, 0);
        r.set_carry(true);
        return Ok(left(&r));
    }
    log_text(LOG_L2, a(TEXT), &r, part)?;
    cmp_flags(&mut r, g(DPLAY), 0);
    if g(DPLAY) == 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    p(SESSION_DESC + 4, g(SESSION_DESC + 4) | CLOSED);
    r.eax = unsafe { call(DPLAY, SET_SESSION_DESC, &[a(SESSION_DESC), 0]) };
    let mut l = r;
    part(LOG_HEX_L2_NL, &mut l)?;
    test_eax(&mut r);
    if r.eax != 0 {
        r.set_carry(true);
    }
    Ok(left(&r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, disable_new_players);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; test byte_52AD64, 10h
    let copy = match crate::net::lobby::copy(
        0x418D8A,
        0x49,
        &[0x60, 0xF6, 0x05, 0x64, 0xAD, 0x52, 0x00, 0x10],
        &[0x11, 0x3A],
        &[LOG_L2, LOG_HEX_L2_NL],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    let all = [dword(DPLAY), region(OWN, 1), region(SESSION_DESC, 8)];
    let saved = Saved::new(&all);
    for (what, state, object, result) in [
        ("not the host", 0x02u8, true, 0u32),
        ("no object", 0x12, false, 0),
        ("failing", 0x10, true, 0x8877_0005),
        ("done", 0x13, true, 0),
    ] {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &all,
            FLAGS,
            &[],
            &|| {
                p(DPLAY, if object { dp } else { 0 });
                pb(OWN, state);
                p(SESSION_DESC + 4, 0x44);
                com_answer(Box::new(move |place, _| {
                    if place == DPLAY_METHOD | SET_SESSION_DESC {
                        result
                    } else {
                        0
                    }
                }));
            },
            &disable_new_players,
        );
    }
    drop(saved);
    drop(dplay);
    t.finish()
}
