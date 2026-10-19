//! `net::find_connections` (0x41867C): "NET: Find connections"; unless the receiving thread
//! has ended (`g_netReceiveDone` 1): the lobby object made (`net::open_lobby`); unless a
//! lobby launched the game (its connection is made there): DirectPlay's object made
//! (`net::create_dplay`), the lists' memory taken (`net::allocate_lists`), and the
//! connections listed (`EnumConnections` for the game's GUID `dword_5AB300` with
//! `net::connection_found`, flags DPCONNECTION_DIRECTPLAY).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if it failed or nothing was
//!      listed (the flags of `cmp dword_52AC34, 1`); after a lobby's launch the flags of
//!      `cmp g_lobbyLaunched, 1`
//! Caller: `net::find_connections_regs`. Its calls through `part`; `EnumConnections` from
//! Rust (`net::dplay`), the callback the game's address (its hook). Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins and whose DirectPlay is a
//! test object, against the Rust version, on every way out.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{ENUM_CONNECTIONS, call};
use crate::net::lobby::{
    APP_GUID, CONNECTIONS, DPLAY, DPLAY_METHOD, FLAGS, LOBBY_LAUNCHED, LOG_L2_NL, Part,
    RECEIVE_DONE, Saved, TestObject, a, answer, cmp_flags, com_answer, dispatch_lobby, dword, g,
    game, log_text, p, run_both, test_eax,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::find_connections", rva(0x41867C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXT: usize = 0x5C2E5F; // "NET: Find connections"
const OPEN_LOBBY: u32 = 0x418E9B; // net::open_lobby
const CREATE_DPLAY: u32 = 0x41861D; // net::create_dplay
const ALLOCATE_LISTS: u32 = 0x418576; // net::allocate_lists
const CONNECTION_FOUND: u32 = 0x4186E5; // net::connection_found
const DIRECTPLAY_CONNECTIONS: u32 = 1; // DPCONNECTION_DIRECTPLAY

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn find_connections(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    log_text(LOG_L2_NL, a(TEXT), &r, part)?;
    cmp_flags(&mut r, g(RECEIVE_DONE), 1);
    if g(RECEIVE_DONE) == 1 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    part(OPEN_LOBBY, &mut r)?;
    if r.carry() {
        return Ok(left(&r));
    }
    cmp_flags(&mut r, g(LOBBY_LAUNCHED), 1);
    if g(LOBBY_LAUNCHED) == 1 {
        return Ok(left(&r));
    }
    part(CREATE_DPLAY, &mut r)?;
    if r.carry() {
        return Ok(left(&r));
    }
    part(ALLOCATE_LISTS, &mut r)?;
    p(CONNECTIONS, 0);
    r.eax = unsafe {
        call(
            DPLAY,
            ENUM_CONNECTIONS,
            &[
                a(APP_GUID),
                game(CONNECTION_FOUND),
                0,
                DIRECTPLAY_CONNECTIONS,
            ],
        )
    };
    test_eax(&mut r);
    if r.eax != 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    cmp_flags(&mut r, g(CONNECTIONS), 1);
    Ok(left(&r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, find_connections);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; pusha; lea eax, aNetFindConnect
    let copy = match crate::net::lobby::copy(
        0x41867C,
        0x69,
        &[0x60, 0x60, 0x8D, 0x05, 0x5F, 0x2E, 0x5C, 0x00],
        &[0x8, 0x17, 0x27, 0x2E],
        &[LOG_L2_NL, OPEN_LOBBY, CREATE_DPLAY, ALLOCATE_LISTS],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    let all = [
        dword(DPLAY),
        dword(CONNECTIONS),
        dword(RECEIVE_DONE),
        dword(LOBBY_LAUNCHED),
    ];
    let saved = Saved::new(&all);
    // (what, done, launched, the failing call (place), what EnumConnections gives, listed)
    let cases: [(&str, u32, u32, u32, u32, u32); 8] = [
        ("the thread ended", 1, 0, 0, 0, 0),
        ("no lobby object", 0, 0, OPEN_LOBBY, 0, 0),
        ("launched", 0, 1, 0, 0, 0),
        ("no DirectPlay", 0, 0, CREATE_DPLAY, 0, 0),
        ("the listing fails", 0, 0, 0, 0x8877_0005, 2),
        ("none listed", 0, 0, 0, 0, 0),
        ("three listed", 0, 0, 0, 0, 3),
        ("one listed", 0, 0, 0, 0, 1),
    ];
    for (what, done, launched, fails, given, listed) in cases {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &all,
            FLAGS,
            &[],
            &|| {
                p(DPLAY, dp);
                p(CONNECTIONS, 7);
                p(RECEIVE_DONE, done);
                p(LOBBY_LAUNCHED, launched);
                answer(Box::new(move |place, r| r.set_carry(place == fails)));
                com_answer(Box::new(move |place, _| {
                    if place == DPLAY_METHOD | ENUM_CONNECTIONS {
                        p(CONNECTIONS, listed);
                    }
                    given
                }));
            },
            &find_connections,
        );
    }
    drop(saved);
    drop(dplay);
    t.finish()
}
