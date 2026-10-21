//! `net::list_players` (0x419192): this machine's record sent to the others as its player
//! data (`SetPlayerData`, `byte_52AD64`, 0x10 bytes; its result not looked at); unless the
//! race is on (`dword_52AD84` 1), the players listed afresh (`g_netPlayers` 0, `EnumPlayers`
//! with `net::player_found`); each listed player's record read into its block (+0x24,
//! `GetPlayerData`, the size in `dword_52AD74`, 0x10); the blocks bubble-sorted by their
//! DirectPlay ids (+0x20, unsigned); this machine's player found among them by its id
//! (`g_netPlayerId`) into `g_netPlayer`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if there is no object, the
//!      listing failed or this machine's player is not among them, else the flags of
//!      `test eax, eax` on its id
//! The search looks at block 0 even with no player listed. More than eight players
//! (possible only with the race on) would sort past the blocks: refused. Caller:
//! `net::list_players_regs`, `net::open_lobby`. DirectPlay's calls from Rust (`net::dplay`),
//! the callback the game's address (its hook). Shadow mode runs the original alone.
//! Self-test: a copy whose DirectPlay is a test object (its `EnumPlayers` listing players),
//! against the Rust version: no object, failing, racing, found, not found.

use crate::errors::{BadInput, Refused};
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{ENUM_PLAYERS, GET_PLAYER_DATA, SET_PLAYER_DATA, call};
use crate::net::lobby::{
    BLOCK, DPLAY, DPLAY_METHOD, FLAGS, OWN, PLAYER, PLAYER_ID, PLAYERS, Part, RACING, Saved,
    TestObject, a, block, cmp_flags, com_answer, dispatch_lobby, dword, g, game, p, run_both,
    test_eax,
};
use crate::net::sort_players::swap_blocks;
use crate::net::types::NetPlayerBlock;
use rc2k_core::net::players;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::list_players", rva(0x419192), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const PLAYER_FOUND: u32 = 0x41929E; // net::player_found
const DATA_SIZE: usize = 0x52AD74; // dword_52AD74

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn list_players(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    cmp_flags(&mut r, g(DPLAY), 0);
    if g(DPLAY) == 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    unsafe {
        r.eax = call(DPLAY, SET_PLAYER_DATA, &[g(PLAYER_ID), a(OWN), 0x10, 0]);
        if g(RACING) != 1 {
            p(PLAYERS, 0);
            r.eax = call(DPLAY, ENUM_PLAYERS, &[0, game(PLAYER_FOUND), 0, 0]);
            test_eax(&mut r);
            if r.eax != 0 {
                r.set_carry(true);
                return Ok(left(&r));
            }
        }
        let players = g(PLAYERS);
        if players > players::MOST {
            return Err(BadInput("more than eight players").into());
        }
        for n in 0..players {
            p(DATA_SIZE, 0x10);
            let at = block(n);
            r.eax = call(
                DPLAY,
                GET_PLAYER_DATA,
                &[
                    get(at + field!(NetPlayerBlock, id)),
                    at + field!(NetPlayerBlock, record),
                    a(DATA_SIZE),
                    0,
                ],
            );
        }
        // Converted: `rc2k_core::net::players` (`bubble` by the ids, `find_own`).
        let id = |n: u32| get(block(n) + field!(NetPlayerBlock, id));
        players::bubble(players, |k| id(k) > id(k + 1), |k| swap_blocks(block(k)));
    }
    r.eax = g(PLAYER_ID);
    match players::find_own(r.eax, g(PLAYERS), |n| {
        get(block(n) + field!(NetPlayerBlock, id))
    }) {
        Ok(n) => {
            p(PLAYER, n);
            test_eax(&mut r);
        }
        Err(n) => {
            cmp_flags(&mut r, n, g(PLAYERS));
            r.set_carry(true);
        }
    }
    Ok(left(&r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, list_players);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; cmp g_dplay, 0
    let copy = match crate::net::lobby::copy(
        0x419192,
        0x10C,
        &[0x60, 0x83, 0x3D, 0x88, 0xB2, 0x5A, 0x00, 0x00],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    let mut all = vec![
        dword(DPLAY),
        dword(PLAYERS),
        dword(PLAYER),
        dword(PLAYER_ID),
        dword(RACING),
        dword(DATA_SIZE),
    ];
    all.push((block(0), 8 * BLOCK as usize));
    let saved = Saved::new(&all);
    // (what, an object, racing, EnumPlayers' result, the players it lists, this one's id)
    let cases: [(&str, bool, u32, u32, u32, u32); 7] = [
        ("no object", false, 0, 0, 3, 0x30),
        ("failing", true, 0, 0x8877_0005, 3, 0x30),
        ("three, found", true, 0, 0, 3, 0x10),
        ("three, not found", true, 0, 0, 3, 0x99),
        ("none listed", true, 0, 0, 0, 0x99),
        ("eight", true, 0, 0, 8, 0x70),
        ("racing, two", true, 1, 0, 2, 0x20),
    ];
    // The ids the listing gives, out of order.
    let ids = [0x50u32, 0x10, 0x30, 0x80, 0x20, 0x70, 0x60, 0x40];
    for (what, object, racing, result, listed, own) in cases {
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
                p(RACING, racing);
                p(PLAYER, 9);
                p(PLAYER_ID, own);
                p(PLAYERS, if racing == 1 { listed } else { 5 });
                for n in 0..8u32 {
                    unsafe {
                        for k in 0..0x40u32 {
                            ((block(n) + 4 * k) as *mut u32).write(n << 8 | k);
                        }
                        ((block(n) + field!(NetPlayerBlock, id)) as *mut u32)
                            .write(ids[n as usize]);
                    }
                }
                com_answer(Box::new(move |place, args| {
                    if place == DPLAY_METHOD | ENUM_PLAYERS {
                        p(PLAYERS, listed);
                        result
                    } else if place == DPLAY_METHOD | GET_PLAYER_DATA {
                        unsafe { (args[2] as *mut u32).write_unaligned(args[1] ^ 0x5A5A) };
                        0
                    } else {
                        0
                    }
                }));
            },
            &list_players,
        );
    }
    drop(saved);
    drop(dplay);
    t.finish()
}
