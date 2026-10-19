//! `net::init_connection` (0x418794): "NET: Init connection index" and the number logged;
//! with DirectPlay's object there, connection eax's address built (`net::build_address`)
//! and DirectPlay set to it (`InitializeConnection`, `dword_52AC40`, flags 0); "NET: Init"
//! and its result logged.
//!
//! In:  eax = the connection's number
//! Out: every register kept (`pusha` .. `popa`); carry set if it failed, else the flags of
//!      `test eax, eax`
//! Caller: `net::connecting`. Its calls through `part`; `InitializeConnection` from Rust
//! (`net::dplay`). Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins and whose DirectPlay is a test object, against the Rust version: no object,
//! no address, failing, done.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{INITIALIZE_CONNECTION, call};
use crate::net::lobby::{
    ADDRESS, DPLAY, DPLAY_METHOD, FLAGS, LOG_HEX_L2_NL, LOG_INT_L2_NL, LOG_L2, Part, Saved,
    TestObject, a, answer, cmp_flags, com_answer, dispatch_lobby, dword, g, log_text, p, run_both,
    test_eax,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::init_connection", rva(0x418794), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXT: usize = 0x5C2E80; // "NET: Init connection index"
const INIT: usize = 0x5C2E9B; // "NET: Init"
const BUILD_ADDRESS: u32 = 0x41882C; // net::build_address

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn init_connection(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    log_text(LOG_L2, a(TEXT), &r, part)?;
    let mut l = r;
    part(LOG_INT_L2_NL, &mut l)?;
    cmp_flags(&mut r, g(DPLAY), 0);
    if g(DPLAY) == 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    part(BUILD_ADDRESS, &mut r)?;
    if r.carry() {
        return Ok(left(&r));
    }
    r.eax = unsafe { call(DPLAY, INITIALIZE_CONNECTION, &[g(ADDRESS), 0]) };
    log_text(LOG_L2, a(INIT), &r, part)?;
    let mut l = r;
    part(LOG_HEX_L2_NL, &mut l)?;
    test_eax(&mut r);
    if r.eax != 0 {
        r.set_carry(true);
    }
    Ok(left(&r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, init_connection);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; pusha; lea eax, aNetInitConnect
    let copy = match crate::net::lobby::copy(
        0x418794,
        0x5A,
        &[0x60, 0x60, 0x8D, 0x05, 0x80, 0x2E, 0x5C, 0x00],
        &[0x8, 0xF, 0x1E, 0x42, 0x49],
        &[LOG_L2, LOG_INT_L2_NL, BUILD_ADDRESS, LOG_HEX_L2_NL],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    let all = [dword(DPLAY), dword(ADDRESS)];
    let saved = Saved::new(&all);
    for (what, object, built, result) in [
        ("no object", false, true, 0u32),
        ("no address", true, false, 0),
        ("failing", true, true, 0x8877_0005),
        ("done", true, true, 0),
    ] {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs {
                eax: 3,
                ..Regs::SENTINEL
            },
            &all,
            FLAGS,
            &[],
            &|| {
                p(DPLAY, if object { dp } else { 0 });
                p(ADDRESS, 0x00AD_D000);
                answer(Box::new(move |place, r| {
                    if place == BUILD_ADDRESS {
                        r.set_carry(!built);
                    }
                }));
                com_answer(Box::new(move |place, _| {
                    if place == DPLAY_METHOD | INITIALIZE_CONNECTION {
                        result
                    } else {
                        0
                    }
                }));
            },
            &init_connection,
        );
    }
    drop(saved);
    drop(dplay);
    t.finish()
}
