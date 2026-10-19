//! `net::race_wait` (0x4171FE): one turn of waiting for the race to start: this player's
//! state made "waiting" (bit 1 off, bit 2 on), its time -1; once a second the players listed
//! (`net::list_players_regs`); on the host, with every player waiting (state bit 2, those
//! with 0x80 left out), the sync of `net::lobby_loop` with 'P' messages, and when all have
//! answered `dword_52AD7C` 1.
//!
//! In:  nothing
//! Out: eax = `dword_52AD7C` (1: go); the rest kept (`pusha` .. `popa`); carry set with no
//!      players on the host (or a failed listing, which never comes), else clear
//! Caller: `net::lets_rally`, once a frame. Its calls through `part`. Shadow mode runs the
//! original alone (it sends). Self-test: a copy whose calls go to stand-ins, against the
//! Rust version: not the host, the host waiting for one, all answered, no players.

use crate::errors::Refused;
use crate::framework::{CARRY_FLAG, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    CLOCK_A, OWN, OWN_TIME, PLAYERS, Part, RACE_OVER, SEND, SYNC_CLOCK, SYNC_FIRST, SYNCED, Saved,
    TICK, a, answer, dispatch_lobby, dword, g, gb, p, pb, run_both,
};
use crate::net::lobby_loop::{all_states, lobby_regions, set_players, states_start, sync};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::race_wait", rva(0x4171FE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LIST_PLAYERS: u32 = 0x417A03; // net::list_players_regs

/// The Rust version: the registers it leaves.
pub fn race_wait(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs, carry: bool| {
        let mut l = Regs {
            eflags: r.eflags,
            ..*entry
        };
        l.set_carry(carry);
        l.eax = g(RACE_OVER);
        l
    };
    pb(OWN, gb(OWN) & 0xFE | 2);
    p(OWN_TIME, u32::MAX);
    part(TICK, &mut r)?;
    r.ebx = r.eax;
    r.eax = r.eax.wrapping_sub(g(CLOCK_A));
    if (r.eax.wrapping_sub(1000) as i32) >= 0 {
        p(CLOCK_A, r.ebx);
        part(LIST_PLAYERS, &mut r)?;
        if r.carry() {
            return Ok(left(&r, true));
        }
    }
    if gb(OWN) & 0x10 != 0 {
        states_start(&mut r, 2);
        if g(PLAYERS) == 0 {
            return Ok(left(&r, true));
        }
        if all_states(&mut r, 2, g(PLAYERS)) == 2 && sync(&mut r, b'P', part)? {
            p(RACE_OVER, 1);
        }
    }
    Ok(left(&r, false))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, race_wait);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; and byte_52AD64, 0FEh
    let copy = match crate::net::lobby::copy(
        0x4171FE,
        0x130,
        &[0x60, 0x80, 0x25, 0x64, 0xAD, 0x52, 0x00, 0xFE],
        &[0x19, 0x33, 0xA5, 0xED],
        &[TICK, LIST_PLAYERS, SEND],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut all = lobby_regions();
    all.push(dword(SYNC_CLOCK));
    all.push(dword(SYNC_FIRST));
    all.push(dword(RACE_OVER));
    let saved = Saved::new(&all);
    // (what, own state, the players' states, the answers, the clock's step, the first sync)
    type Case = (&'static str, u8, Vec<u8>, u8, u32, u32);
    let cases: Vec<Case> = vec![
        ("not the host", 0x01, vec![0x02, 0x02], 0, 2000, 0),
        (
            "the host, waiting for one",
            0x11,
            vec![0x12, 0x02, 0x82],
            0,
            2000,
            1,
        ),
        ("the host, all answered", 0x11, vec![0x12, 0x02], 1, 2000, 0),
        (
            "the host, one not waiting",
            0x11,
            vec![0x12, 0x01],
            1,
            500,
            0,
        ),
        ("the host, no players", 0x10, vec![], 0, 10, 0),
    ];
    for (what, own, states, answers, step, first) in cases {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &all,
            CARRY_FLAG,
            &[],
            &|| {
                pb(OWN, own);
                p(PLAYERS, states.len() as u32);
                set_players(&states);
                p(RACE_OVER, 0);
                p(SYNC_FIRST, first);
                p(CLOCK_A, 0x1000);
                p(SYNC_CLOCK, 0x1000);
                unsafe { std::ptr::write_bytes(a(SYNCED) as *mut u8, answers, 8) };
                answer(Box::new(move |place, r| {
                    if place == TICK {
                        r.eax = 0x1000 + step;
                    }
                }));
            },
            &race_wait,
        );
    }
    drop(saved);
    t.finish()
}
