//! `net::sort_players` (0x418108): the eight players' blocks of `g_netReceived` copied to
//! `unk_56AFFC` and bubble-sorted there for the lobby's table: by the total (+0x30), then
//! the race's time (+0x2C), then the DirectPlay id (+0x20), unsigned, the smaller first; a
//! player whose time is -1 (no time) goes after one who has a time.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! `g_netPlayers` - 1 passes (signed: none for one player or none). More than eight players
//! would sort past the copy: refused (DirectPlay's listing keeps eight at most). Caller:
//! `net::lobby_frame`. Shadow mode: both run, the copy compared. Self-test: a copy against
//! the Rust version on 0, 1, 2, 5 and 8 players with times, totals and ids that tie.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    BLOCK, PLAYERS, Part, RECEIVED, SORTED, Saved, a, dword, fill, g, movsb, p, region, run_both,
};
use rc2k_core::net::players::{self, Standing};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::sort_players", rva(0x418108), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const ALL: u32 = 8 * BLOCK;

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The block at `x` as the lobby's table sorts it.
fn standing(x: u32) -> Standing {
    Standing {
        total: get(x + 0x30),
        time: get(x + 0x2C),
        id: get(x + 0x20),
    }
}

/// The blocks at `x` and the next swapped, a dword at a time.
pub fn swap_blocks(x: u32) {
    for n in 0..BLOCK / 4 {
        let (one, two) = (x + 4 * n, x + BLOCK + 4 * n);
        let v = get(one);
        unsafe {
            (one as *mut u32).write_unaligned(get(two));
            (two as *mut u32).write_unaligned(v);
        }
    }
}

/// The Rust version: the registers it leaves (the entry's).
pub fn sort_players(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let players = g(PLAYERS);
    if players > players::MOST {
        return Err(BadInput("more than eight players").into());
    }
    movsb(a(SORTED), a(RECEIVED), ALL);
    // Converted: `rc2k_core::net::players` (`bubble`, `goes_after`).
    let x = |k: u32| a(SORTED) + k * BLOCK;
    players::bubble(
        players,
        |k| players::goes_after(standing(x(k)), standing(x(k) + BLOCK)),
        |k| swap_blocks(x(k)),
    );
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_regs(
        &HOOK,
        r,
        call_original,
        &[region(SORTED, ALL as usize)],
        0,
        &|r| sort_players(r, &mut crate::net::lobby::game_part),
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; lea esi, g_netReceived
    let copy = match crate::net::lobby::copy(
        0x418108,
        0x93,
        &[0x60, 0x8D, 0x35, 0x9C, 0xAD, 0x52, 0x00],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [
        region(SORTED, ALL as usize),
        region(RECEIVED, ALL as usize),
        dword(PLAYERS),
    ];
    let saved = Saved::new(&all);
    // (total, time, id) of each player.
    let players: [(u32, u32, u32); 8] = [
        (5, 300, 9),
        (5, u32::MAX, 3),
        (2, 400, 7),
        (5, 300, 4),
        (0, u32::MAX, 1),
        (9, 100, 2),
        (2, 400, 7),
        (0x8000_0000, 1, 0),
    ];
    for count in [0u32, 1, 2, 5, 8] {
        run_both(
            &mut t,
            &format!("{count} players"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            0,
            &[],
            &|| {
                fill(RECEIVED, 0x200, 21);
                fill(SORTED, 0x200, 22);
                for (n, &(total, time, id)) in players.iter().enumerate() {
                    let b = a(RECEIVED) + n as u32 * BLOCK;
                    unsafe {
                        ((b + 0x20) as *mut u32).write_unaligned(id);
                        ((b + 0x2C) as *mut u32).write_unaligned(time);
                        ((b + 0x30) as *mut u32).write_unaligned(total);
                        ((b + 0x100) as *mut u32).write_unaligned(0xABC0 + n as u32);
                    }
                }
                p(PLAYERS, count);
            },
            &sort_players,
        );
    }
    drop(saved);
    t.finish()
}
