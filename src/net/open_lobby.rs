//! `net::open_lobby` (0x418E9B): "NET: Create dplobby", DirectPlay's lobby object made
//! (`CoCreateInstance` of CLSID_DirectPlayLobby `stru_5AB340`, IID_IDirectPlayLobby2A
//! `stru_5AB320`) into `g_dplayLobby`; then, for a game a lobby launched
//! (`g_lobbyLaunched` 1): its connection settings read (`GetConnectionSettings` twice: the
//! size, which must come back DPERR_BUFFERTOOSMALL, into `dword_5AB298`; the settings into
//! memory of the network's list, `dword_5AB294`), the session's flags made 0x44 and the
//! settings put back (`SetConnectionSettings`), the lobby's `Connect` into the object
//! whose address `dword_5AB284` holds, `g_dplay` asked for IDirectPlay4A
//! (`QueryInterface`), the object at `dword_5AB284` released and the variable 0; the
//! player's name from the settings (`strings::copy_printable` to `byte_52AB70`), the player
//! set up (`net::fill_own_player`) and made (`net::create_player`); then the session's group
//! made (`net::create_group`, as the host: state bit 0x10) when the settings say to create
//! the session (flag 2), else joined (`net::join_group`); the players listed
//! (`net::list_players`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if something failed, else the
//!      flags of `test eax, eax`
//! `Connect` is given `dword_5AB284`'s value as where to write, and that is 0: it can only
//! fail, so a lobby's launch never gets further (section 16). Past it the original would
//! read through `g_dplay` and the object at `dword_5AB284`: refused while they are null.
//! Caller: `net::find_connections`. Its calls through `part`; DirectPlay's methods from
//! Rust (`net::dplay`). Shadow mode runs the original alone. Self-test: a copy whose calls
//! go to stand-ins and whose objects are test objects, against the Rust version, on every
//! way out.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::create_dplay::{IID_DIRECTPLAY4A, create};
use crate::net::dplay::{
    CONNECT, GET_CONNECTION_SETTINGS, QUERY_INTERFACE, RELEASE, SET_CONNECTION_SETTINGS, call,
    method,
};
use crate::net::lobby::{
    ALLOCATE, CO_CREATE, COPY_PRINTABLE, DPLAY, DPLAY_2, DPLAY_METHOD, FLAGS, LOBBY,
    LOBBY_LAUNCHED, LOBBY_METHOD, LOG_L2_NL, NET_MEMORY, OWN, PLAYER_NAME, Part, SETTINGS,
    SETTINGS_SIZE, Saved, TestObject, a, answer, com_answer, dispatch_lobby, dword, g, gb,
    log_text, p, pb, region, run_both, test_eax,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::open_lobby", rva(0x418E9B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXT: usize = 0x5C2F5A; // "NET: Create dplobby"
pub const CLSID_DIRECTPLAY_LOBBY: usize = 0x5AB340; // stru_5AB340
pub const IID_DIRECTPLAY_LOBBY2A: usize = 0x5AB320; // stru_5AB320
const BUFFER_TOO_SMALL: u32 = 0x8877_001E; // DPERR_BUFFERTOOSMALL
const FILL_OWN_PLAYER: u32 = 0x4181A3; // net::fill_own_player
const CREATE_PLAYER: u32 = 0x4190FB; // net::create_player
const CREATE_GROUP: u32 = 0x4192EA; // net::create_group
const JOIN_GROUP: u32 = 0x419354; // net::join_group
const LIST_PLAYERS: u32 = 0x419192; // net::list_players

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn open_lobby(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let done = |r: &Regs| {
        Ok(Regs {
            eflags: r.eflags,
            ..*entry
        })
    };
    let fail = |r: &mut Regs| {
        r.set_carry(true);
        Ok(Regs {
            eflags: r.eflags,
            ..*entry
        })
    };
    log_text(LOG_L2_NL, a(TEXT), &r, part)?;
    create(
        &mut r,
        a(CLSID_DIRECTPLAY_LOBBY),
        a(IID_DIRECTPLAY_LOBBY2A),
        a(LOBBY),
        part,
    )?;
    test_eax(&mut r);
    if r.eax != 0 {
        return fail(&mut r);
    }
    if g(LOBBY_LAUNCHED) != 1 {
        test_eax(&mut r);
        return done(&r);
    }
    unsafe {
        p(SETTINGS_SIZE, 0);
        r.eax = call(LOBBY, GET_CONNECTION_SETTINGS, &[0, 0, a(SETTINGS_SIZE)]);
        let v = r.eax;
        crate::net::lobby::cmp_flags(&mut r, v, BUFFER_TOO_SMALL);
        if r.eax != BUFFER_TOO_SMALL {
            return fail(&mut r);
        }
        r.eax = g(SETTINGS_SIZE);
        r.ebx = a(NET_MEMORY);
        part(ALLOCATE, &mut r)?;
        if r.carry() {
            return fail(&mut r);
        }
        p(SETTINGS, r.eax);
        r.eax = call(
            LOBBY,
            GET_CONNECTION_SETTINGS,
            &[0, g(SETTINGS), a(SETTINGS_SIZE)],
        );
        test_eax(&mut r);
        if r.eax != 0 {
            return fail(&mut r);
        }
        r.ebx = get(g(SETTINGS) + 8);
        ((r.ebx + 4) as *mut u32).write_unaligned(0x44);
        r.eax = call(LOBBY, SET_CONNECTION_SETTINGS, &[0, 0, g(SETTINGS)]);
        test_eax(&mut r);
        if r.eax != 0 {
            return fail(&mut r);
        }
        r.eax = call(LOBBY, CONNECT, &[0, g(DPLAY_2), 0]);
        test_eax(&mut r);
        if r.eax != 0 {
            return fail(&mut r);
        }
        if g(DPLAY) == 0 || g(DPLAY_2) == 0 {
            return Err(BadInput("a lobby's connection with no DirectPlay object").into());
        }
        r.ebx = call(DPLAY, QUERY_INTERFACE, &[a(IID_DIRECTPLAY4A), a(DPLAY)]);
        r.eax = method(g(DPLAY_2), RELEASE, &[]);
        p(DPLAY_2, 0);
        test_ebx(&mut r);
        if r.ebx != 0 {
            return fail(&mut r);
        }
        r.eax = get(get(g(SETTINGS) + 0xC) + 8);
        let mut c = Regs {
            ecx: 0x1F,
            ebx: a(PLAYER_NAME),
            ..r
        };
        part(COPY_PRINTABLE, &mut c)?;
        part(FILL_OWN_PLAYER, &mut r)?;
        part(CREATE_PLAYER, &mut r)?;
        if r.carry() {
            return fail(&mut r);
        }
        r.eax = g(SETTINGS); // mov eax, dword_5AB294; test dword ptr [eax+4], 2
        if get(r.eax + 4) & 2 != 0 {
            pb(OWN, gb(OWN) | 0x10);
            part(CREATE_GROUP, &mut r)?;
        } else {
            part(JOIN_GROUP, &mut r)?;
        }
        if r.carry() {
            return fail(&mut r);
        }
        part(LIST_PLAYERS, &mut r)?;
        test_eax(&mut r);
        done(&r)
    }
}

/// The flags of `test ebx, ebx`.
fn test_ebx(r: &mut Regs) {
    r.set_carry(false);
    r.set_zero(r.ebx == 0);
    crate::net::lobby::set_sign(r, r.ebx as i32);
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, open_lobby);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [
        LOG_L2_NL,
        ALLOCATE,
        COPY_PRINTABLE,
        FILL_OWN_PLAYER,
        CREATE_PLAYER,
        CREATE_GROUP,
        JOIN_GROUP,
        LIST_PLAYERS,
    ];
    // pusha; pusha; lea eax, aNetCreateDplob
    let copy = match crate::net::lobby::copy(
        0x418E9B,
        0x174,
        &[0x60, 0x60, 0x8D, 0x05, 0x5A, 0x2F, 0x5C, 0x00],
        &[0x8, 0x21, 0x6F, 0x131, 0x137, 0x13C, 0x158, 0x161, 0x168],
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let dplay = TestObject::dplay();
    let lobby = TestObject::lobby();
    let older = TestObject::dplay();
    let (dp, lo, old) = (dplay.addr(), lobby.addr(), older.addr());
    // The connection's settings: +4 the flags, +8 the session's description, +0xC the
    // player's name (a DPNAME, +8 the short name).
    let mut memory = vec![0u32; 0x40];
    let m = memory.as_mut_ptr() as u32;
    let all = [
        dword(DPLAY),
        dword(LOBBY),
        dword(DPLAY_2),
        dword(LOBBY_LAUNCHED),
        dword(SETTINGS),
        dword(SETTINGS_SIZE),
        region(OWN, 1),
        (m, 0x100),
    ];
    let saved = Saved::new(&all);
    // (what, launched, CoCreateInstance, first settings, allocation fails, Connect, QI,
    // create the session, the player made fails)
    type Case = (&'static str, u32, u32, u32, bool, u32, u32, bool, bool);
    let cases: [Case; 10] = [
        (
            "no lobby object",
            1,
            0x8004_0154,
            0,
            false,
            0,
            0,
            false,
            false,
        ),
        ("not launched", 0, 0, 0, false, 0, 0, false, false),
        ("the size not asked", 1, 0, 0, false, 0, 0, false, false),
        (
            "no memory",
            1,
            0,
            BUFFER_TOO_SMALL,
            true,
            0,
            0,
            false,
            false,
        ),
        (
            "Connect fails",
            1,
            0,
            BUFFER_TOO_SMALL,
            false,
            0x8877_0005,
            0,
            false,
            false,
        ),
        (
            "QueryInterface fails",
            1,
            0,
            BUFFER_TOO_SMALL,
            false,
            0,
            0x8000_4002,
            false,
            false,
        ),
        ("joined", 1, 0, BUFFER_TOO_SMALL, false, 0, 0, false, false),
        ("hosted", 1, 0, BUFFER_TOO_SMALL, false, 0, 0, true, false),
        (
            "the player not made",
            1,
            0,
            BUFFER_TOO_SMALL,
            false,
            0,
            0,
            true,
            true,
        ),
        ("launched 2", 2, 0, 0, false, 0, 0, false, false),
    ];
    for (what, launched, made, first, no_memory, connect, query, host, no_player) in cases {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &all,
            FLAGS,
            &[],
            &|| unsafe {
                p(DPLAY, dp);
                p(LOBBY, 0);
                p(DPLAY_2, old);
                p(LOBBY_LAUNCHED, launched);
                p(SETTINGS, 0x1111);
                p(SETTINGS_SIZE, 0x2222);
                pb(OWN, 0x02);
                std::ptr::write_bytes(m as *mut u8, 0x5C, 0x100);
                let wm = m + 0x40;
                (wm as *mut [u32; 4]).write([
                    0x50,
                    if host { 0x2 } else { 0x1 },
                    m + 0x80,
                    m + 0xC0,
                ]);
                ((m + 0xC0 + 8) as *mut u32).write(0x0055_AA00);
                let mut gets = 0;
                com_answer(Box::new(move |place, args| match place {
                    CO_CREATE => {
                        (args[4] as *mut u32).write_unaligned(if made == 0 { lo } else { 0 });
                        made
                    }
                    x if x == LOBBY_METHOD | GET_CONNECTION_SETTINGS => {
                        gets += 1;
                        if gets == 1 {
                            (args[3] as *mut u32).write_unaligned(0x50);
                            first
                        } else {
                            0
                        }
                    }
                    x if x == LOBBY_METHOD | CONNECT => connect,
                    x if x == DPLAY_METHOD | QUERY_INTERFACE => query,
                    _ => 0,
                }));
                answer(Box::new(move |place, r| match place {
                    ALLOCATE => {
                        r.eax = wm;
                        r.set_carry(no_memory);
                    }
                    CREATE_PLAYER => r.set_carry(no_player),
                    _ => {}
                }));
            },
            &open_lobby,
        );
    }
    drop(saved);
    let _ = (&mut memory, dplay, lobby, older);
    t.finish()
}
