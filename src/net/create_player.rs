//! `net::create_player` (0x4190FB): "NET: Create player" and the player's name
//! (`byte_52AB70`) logged; a DPNAME made (`dword_5AB29C`: zeroed, its size 0x10, the short
//! name) and this machine's DirectPlay player made with it (`CreatePlayer` into
//! `g_netPlayerId`, the receiving thread's event `g_netEvent`, the player's record
//! `byte_52AD64` as its data, 0x10 bytes, flags 0).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if it failed, else the flags of
//!      `test eax, eax`
//! Callers: `net::host_session`, `net::join_session`, `net::open_lobby`. Its calls through
//! `part`; `CreatePlayer` from Rust (`net::dplay`). Shadow mode runs the original alone.
//! Self-test: a copy whose calls go to stand-ins and whose DirectPlay is a test object,
//! against the Rust version: made, failing.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{CREATE_PLAYER, call};
use crate::net::lobby::{
    DP_NAME, DPLAY, DPLAY_METHOD, FLAGS, LOG_L2, LOG_L2_NL, OWN, PLAYER_ID, PLAYER_NAME, Part,
    Saved, TestObject, ZERO_MEMORY, a, answer, com_answer, dispatch_lobby, dword, fill, g,
    log_text, p, region, run_both, test_eax,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::create_player", rva(0x4190FB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXT: usize = 0x5C2F6E; // "NET: Create player"
const EVENT: usize = 0x52AB68; // g_netEvent

/// A DPNAME at `dword_5AB29C` made: zeroed, its size, the short name.
pub fn dp_name(r: &Regs, short_name: u32, part: Part) -> Result<(), Refused> {
    let mut c = Regs {
        ebx: a(DP_NAME),
        ecx: 0x10,
        ..*r
    };
    part(ZERO_MEMORY, &mut c)?;
    p(DP_NAME, 0x10);
    p(DP_NAME + 8, short_name);
    Ok(())
}

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn create_player(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    log_text(LOG_L2, a(TEXT), &r, part)?;
    log_text(LOG_L2_NL, a(PLAYER_NAME), &r, part)?;
    dp_name(&r, a(PLAYER_NAME), part)?;
    r.eax = unsafe {
        call(
            DPLAY,
            CREATE_PLAYER,
            &[a(PLAYER_ID), a(DP_NAME), g(EVENT), a(OWN), 0x10, 0],
        )
    };
    test_eax(&mut r);
    if r.eax != 0 {
        r.set_carry(true);
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, create_player);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; pusha; lea eax, aNetCreatePlaye
    let copy = match crate::net::lobby::copy(
        0x4190FB,
        0x70,
        &[0x60, 0x60, 0x8D, 0x05, 0x6E, 0x2F, 0x5C, 0x00],
        &[0x8, 0x15, 0x28],
        &[LOG_L2, LOG_L2_NL, ZERO_MEMORY],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    let all = [dword(DPLAY), region(DP_NAME, 0x10)];
    let saved = Saved::new(&all);
    for result in [0u32, 0x8877_0005] {
        run_both(
            &mut t,
            &format!("CreatePlayer gives {result:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            FLAGS,
            &[],
            &|| {
                p(DPLAY, dp);
                fill(DP_NAME, 0x10, 91);
                answer(Box::new(|_, _| {}));
                com_answer(Box::new(move |place, _| {
                    if place == DPLAY_METHOD | CREATE_PLAYER {
                        result
                    } else {
                        0
                    }
                }));
            },
            &create_player,
        );
    }
    drop(saved);
    drop(dplay);
    t.finish()
}
